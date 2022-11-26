use super::js_value_match::{Match};
use crate::serve::request_state::RequestState;

use crate::serve::js_value::{JsValue, MemoryBlock};
use wasmtime::{
    Engine,
    Store,
    Func,
    Caller,
    Instance,
    Extern,
    Module, Memory,
};
use wasmtime::AsContextMut;
use tokio::sync::mpsc::{UnboundedSender};

#[derive(Debug)]
pub enum Message {
    TimeoutAndSendResponse,
    DomUpdate(String),
    Panic(Option<String>),
}


fn get_memory(caller: &mut Caller<'_, RequestState>) -> Option<Memory> {
    let mem = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => {
            log::error!("can't read the memory");
            return None;
        }
    };

    Some(mem)
}

fn get_data(caller: &mut Caller<'_, RequestState>, ptr: u32, offset: u32) -> Option<String> {
    let Some(mem) = get_memory(caller) else {
        return None;
    };

    let context = caller.as_context_mut();
    let buff = mem.data(&context);
    
    let ptr = ptr as usize;
    let offset = offset as usize;

    let slice = &buff[ptr..(ptr+offset)];
    
    let slice_vec = Vec::from(slice);

    let Ok(result) = String::from_utf8(slice_vec) else {
        log::error!("panic message decoding problem");
        return None;
    };

    Some(result)
}

fn get_value(caller: &mut Caller<'_, RequestState>, ptr: u32, offset: u32) -> JsValue {
    let Some(mem) = get_memory(caller) else {
        return JsValue::Undefined;
    };

    let context = caller.as_context_mut();
    let buff = mem.data(&context);
    
    let ptr = ptr as usize;
    let offset = offset as usize;

    let slice = &buff[ptr..(ptr+offset)];

    let block = MemoryBlock::from_slice(slice);
    match JsValue::from_block(block) {
        Ok(value) => value,
        Err(error) => {
            log::info!("JsValue decoding problem, error={error}");
            JsValue::Undefined
        }
    }
}


fn save_value(caller: &mut Caller<'_, RequestState>, value: JsValue) -> u32 {
    let block = value.to_snapshot();
    let block = block.convert_to_vec();
    let size = block.len();

    let method = caller.get_export("alloc").unwrap();
    
    if let Extern::Func(alloc_inner) = method {
        let Some(mem) = get_memory(caller) else {
            return 0;
        };

        let mut context = caller.as_context_mut();

        let alloc = alloc_inner.typed::<u32, u32, _>(&mut context).unwrap();

        let ptr = alloc.call(&mut context, size as u32).unwrap() as usize;
        
        let buff = mem.data_mut(&mut context);

        buff[ptr..(ptr+size)].clone_from_slice(block.as_slice());

        ptr as u32

    } else {
        0
    }
}


fn match_hashrouter(arg: &JsValue) -> Result<JsValue, ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "hashRouter"])?;
    let matcher = matcher.test_list(&["call", "get"])?;
    matcher.end()?;

    Ok(JsValue::str(""))
}

fn match_hashrouter_callback(arg: &JsValue) -> Result<(), ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "hashRouter"])?;
    let (matcher, _) = matcher.test_list_with_fn(|matcher: Match| -> Result<u64, ()> {
        let matcher = matcher.str("call")?;
        let matcher = matcher.str("add")?;
        let (matcher, callback_id) = matcher.u64()?;
        matcher.end()?;
    
        Ok(callback_id)
    })?;
    matcher.end()?;

    Ok(())
}

fn match_dom_bulk_update(arg: &JsValue) -> Result<String, ()> {
    let matcher = Match::new(arg)?;
    let matcher = matcher.test_list(&["api"])?;
    let matcher = matcher.test_list(&["get", "dom"])?;
    let (matcher, data) = matcher.test_list_with_fn(|matcher: Match| -> Result<String, ()> {
        let matcher = matcher.str("call")?;
        let matcher = matcher.str("dom_bulk_update")?;
        let (matcher, data) = matcher.string()?;
        matcher.end()?;
    
        Ok(data)
    })?;
    matcher.end()?;

    Ok(data)
}


pub struct WasmInstance {
    instance: Instance,
    store: Store<RequestState>,
}

impl WasmInstance {    
    pub fn new(sender: UnboundedSender<Message>, engine: &Engine, module: &Module, request: RequestState) -> Self {

        let mut store = Store::new(engine, request);

        let import_panic_message = Func::wrap(&mut store, {
            let sender = sender.clone();

            move |mut caller: Caller<'_, RequestState>, ptr: u32, offset: u32| {
                let message = get_data(&mut caller, ptr, offset);
                log::error!("panic: {message:?}");

                //TODO - to remove ?
                println!("Calling back...");
                println!("> {}", caller.data().name);
                caller.data_mut().count += 1;

                sender.send(Message::Panic(message)).unwrap();
            }
        });

        let import_dom_access = {
            Func::wrap(
                &mut store,
                move |mut caller: Caller<'_, RequestState>, ptr: u32, offset: u32| -> u32 {

                let value = get_value(&mut caller, ptr, offset);

                //get hash router
                if let Ok(result) = match_hashrouter(&value) {
                    return save_value(&mut caller, result);
                }

                //adding callback for hashrouter
                if match_hashrouter_callback(&value).is_ok() {
                    return 0;
                }

                if let Ok(data) = match_dom_bulk_update(&value) {
                    sender.send(Message::DomUpdate(data)).unwrap();
                    return 0;
                }

                log::error!("unsupported message: {value:?}");
                0
            })
        };

        let imports = [
            import_panic_message.into(),
            import_dom_access.into()
        ];
        let instance = Instance::new(&mut store, module, &imports).unwrap();

        WasmInstance {
            instance,
            store
        }
    }

    pub fn call_function<Params: wasmtime::WasmParams>(&mut self, name: &'static str, params: Params) {
        let start_application = {
            self.instance.get_typed_func::<Params, (), _>(&mut self.store, name).unwrap()
        };

        let result = start_application.call(&mut self.store, params);
        if let Err(_) = result {
            log::error!("wasm_instance:call_function -> {name} - ended with error");
        }
    }

    //TODO - tutaj będzie miejsce na callback zwrotny, zasilający danymi wasma (callback zwrotny z fetch)
}

