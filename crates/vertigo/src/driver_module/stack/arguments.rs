use std::fmt::Display;
use std::rc::Rc;
use std::{collections::HashMap, cell::RefCell};
use crate::driver_module::api::PanicMessage;

use super::alloc::WasmMemoryPtr;
use super::alloc_buffer::AllocBuffer;
use super::alloc_string::AllocString;
use super::param_list::{ParamList, ParamItem};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ListId(pub u32);

impl ListId {
    pub fn next(&mut self) {
        self.0 += 1;
    }

    pub fn to_u32(self) -> u32 {
        let ListId(value) = self;
        value
    }
}

impl Display for ListId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("ListId = {}", self.0).as_str())
    }
}

struct StackAllocInner {
    next_id: ListId,
    params: HashMap<ListId, ParamList>,
    freeze: HashMap<ListId, ParamList>,
}

impl StackAllocInner {
    pub fn new() -> StackAllocInner {
        StackAllocInner {
            next_id: ListId(1),
            params: HashMap::new(),
            freeze: HashMap::new(),
        }
    }

    fn get_next_id(&mut self) -> ListId {
        let next_id = self.next_id;
        self.next_id.next();
        next_id
    }

    fn get_mut(&mut self, id: ListId) -> &mut ParamList {
        let item = self.params.get_mut(&id);

        if let Some(item) = item {
            item
        } else {
            panic!("Missing list {id}");
        }
    }

    fn get(&self, id: ListId) -> Option<&ParamList> {
        let item = self.params.get(&id);

        if let Some(item) = item {
            Some(item)
        } else {
            None
        }
    }

    fn remove(&mut self, id: ListId) -> ParamList {
        if let Some(list) = self.params.remove(&id) {
            list
        } else {
            panic!("Missing list {id}");
        }
    }
}


#[derive(Clone)]
pub struct ArgumentsManager {
    panic_message: PanicMessage,
    inner: Rc<RefCell<StackAllocInner>>,
}

impl ArgumentsManager {
    pub fn new(panic_message: PanicMessage,) -> ArgumentsManager {
        ArgumentsManager {
            panic_message,
            inner: Rc::new(RefCell::new(StackAllocInner::new()))
        }
    }

    pub fn debug(&self, list_id: ListId) {
        let message = {
            let data = self.inner.borrow();
            let item = data.get(list_id);
            format!("Debug {list_id} --> {item:#?}")
        };

        log::info!("{message}");
    }

    pub fn new_list(&self) -> ListId {
        let mut data = self.inner.borrow_mut();

        let next_id = data.get_next_id();
        data.params.insert(next_id, ParamList::new());
        next_id
    }

    fn push_item(&self, id: ListId, new_item: ParamItem) {
        let mut data = self.inner.borrow_mut();
        let item = data.get_mut(id);
        item.0.push(new_item);
    }

    pub fn push_string_empty(&self, id: ListId) {
        self.push_item(id, ParamItem::StringEmpty);
    }

    pub fn push_string_alloc(&self, id: ListId, size: u32) -> WasmMemoryPtr {
        let string = AllocString::new(size);
        let ptr = string.get_ptr();

        self.push_item(id, ParamItem::StringAlloc(string));
        ptr
    }

    pub fn push_buffer_alloc(&self, id: ListId, size: u32) -> WasmMemoryPtr {
        let buffer = AllocBuffer::new(size);
        let ptr = buffer.get_ptr();

        self.push_item(id, ParamItem::Buffer(buffer));
        ptr
    }

    pub fn push_string_static(&self, id: ListId, value: &'static str) {
        self.push_item(id, ParamItem::StringStatic(value));
    }

    pub fn push_string(&self, id: ListId, value: String) {
        self.push_item(id, ParamItem::String(value));
    }

    pub fn push_u32(&self, id: ListId, value: u32) {
        self.push_item(id, ParamItem::U32(value));
    }

    pub fn push_i32(&self, id: ListId, value: i32) {
        self.push_item(id, ParamItem::I32(value));
    }

    pub fn push_u64(&self, id: ListId, value: u64) {
        self.push_item(id, ParamItem::U64(value));
    }

    pub fn push_i64(&self, id: ListId, value: i64) {
        self.push_item(id, ParamItem::I64(value));
    }

    pub fn push_true(&self, id: ListId) {
        self.push_item(id, ParamItem::True);
    }

    pub fn push_false(&self, id: ListId) {
        self.push_item(id, ParamItem::False);
    }
    
    pub fn push_null(&self, id: ListId) {
        self.push_item(id, ParamItem::Null);
    }

    pub fn push_undefined(&self, id: ListId) {
        self.push_item(id, ParamItem::Undefined);
    }

    pub fn push_list(&self, id: ListId, sublist_id: ListId) {
        let mut data = self.inner.borrow_mut();
        let sublist = data.remove(sublist_id);
        let item = data.get_mut(id);
        item.0.push(ParamItem::List(sublist));
    }

    ///freezeing the parameters on the js side
    pub fn freeze(&self, id: ListId) {
        let mut data = self.inner.borrow_mut();

        let item = data.remove(id);
        data.freeze.insert(id, item);
    }

    ///Receiving parameters from js on the rust side
    pub fn unfreeze(&self, id: u32) -> Option<ParamList> {
        if id == 0 {
            return None;
        }

        let mut data = self.inner.borrow_mut();

        let id = ListId(id);
        let params = data.freeze.remove(&id);

        if params.is_none() {
            self.panic_message.show(format!("Frozen parameters for id={id} are missing"));
        }

        params
    }

    pub fn get_snapshot(&self, id: ListId) -> ParamsListSnapshot {
        let mut data = self.inner.borrow_mut();
        let item = data.remove(id);
        let snapshot = item.to_snapshot();

        ParamsListSnapshot::new(item, snapshot)
    }

}



pub struct ParamsListSnapshot {
    ///We need to keep this memory alive because the pointers stored in the snapshot buffer refer to it
    _param_list: ParamList,
    snapshot: AllocBuffer,
}

impl ParamsListSnapshot {
    pub fn new(param_list: ParamList, snapshot: AllocBuffer) -> ParamsListSnapshot {
        ParamsListSnapshot {
            _param_list: param_list,
            snapshot
        }
    }

    pub fn to_ptr(&self) -> u32 {
        self.snapshot.get_ptr().to_u32()
    }
}

