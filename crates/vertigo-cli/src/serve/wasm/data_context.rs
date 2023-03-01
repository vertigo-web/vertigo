use wasmtime::{
    AsContextMut,
    Caller,
    Extern,
    Instance,
    Memory,
    Store,
    StoreContextMut,
};

use crate::serve::request_state::RequestState;
use crate::serve::js_value::{JsValue, MemoryBlock};

pub enum DataContext<'a> {
    Caller {
        caller: Caller<'a, RequestState>,
    },
    Store {
        store: &'a mut Store<RequestState>,
        instance: Instance,
    }
}

impl<'a> DataContext<'a> {
    pub fn from_caller(caller: Caller<'a, RequestState>) -> Self {
        DataContext::Caller {
            caller,
        }
    }

    pub fn from_store(store: &'a mut Store<RequestState>, instance: Instance) -> Self {
        DataContext::Store {
            store,
            instance
        }
    }

    fn get_context(&mut self) -> StoreContextMut<'_, RequestState> {
        match self {
            Self::Caller { caller } => caller.as_context_mut(),
            Self::Store { store, ..} => store.as_context_mut(),
        }
    }
    fn get_memory(&mut self) -> Memory {
        match self {
            Self::Caller { caller } => {
                let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                    unreachable!()
                };

                memory
            },
            Self::Store { instance, store } => {
                let context = store.as_context_mut();
                let Some(Extern::Memory(memory)) = instance.get_export(context, "memory") else {
                    unreachable!();
                };

                memory
            }
        }
    }

    pub fn get_value(&mut self, ptr: u32, offset: u32) -> JsValue {
        let memory = self.get_memory();
        let context = self.get_context();

        let buff = memory.data(&context);

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

    pub fn get_string_from(&mut self, ptr: u32, offset: u32) -> Option<String> {
        let memory = self.get_memory();
        let context = self.get_context();
        let buff = memory.data(&context);

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

    fn alloc(&mut self, size: usize) -> usize {
        let alloc_inner = match self {
            Self::Caller { caller, .. } => {
                let Some(Extern::Func(alloc_inner)) = caller.get_export("alloc") else {
                    unreachable!();
                };
                alloc_inner
            },
            Self::Store { store, instance, .. } => {
                let Some(Extern::Func(alloc_inner)) = instance.get_export(store, "alloc") else {
                    unreachable!();
                };

                alloc_inner
            }
        };

        let mut context = self.get_context();
        let alloc = alloc_inner.typed::<u32, u32>(&mut context).unwrap();

        alloc.call(&mut context, size as u32).unwrap() as usize
    }

    pub fn save_value(&mut self, value: JsValue) -> u32 {
        if let JsValue::Undefined = value {
            return 0;
        }

        let block = value.to_snapshot();
        let block = block.convert_to_vec();
        let size = block.len();

        let ptr = self.alloc(size);

        let memory = self.get_memory();
        let context = self.get_context();
        let buff = memory.data_mut(context);

        buff[ptr..(ptr+size)].clone_from_slice(block.as_slice());

        ptr as u32
    }

}
