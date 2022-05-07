use std::{cell::RefCell};
use std::rc::Rc;

enum StackItem {
    String {
        ptr: *mut u8,
        len: usize
    },
    EmptyString,
}

#[derive(Clone)]
pub struct StackStringAlloc {
    list: Rc<RefCell<Vec<StackItem>>>,
}

impl StackStringAlloc {
    pub(crate) fn new() -> StackStringAlloc {
        StackStringAlloc {
            list: Rc::new(RefCell::new(Vec::new()))
        }
    }

    pub(crate) fn alloc_empty_string(&self) {
        let mut state = self.list.borrow_mut();
        state.push(StackItem::EmptyString);
    }

    pub(crate) fn alloc(&self, size: usize) -> *mut u8 {
        use std::alloc::{alloc, Layout};
        use std::mem;

        let align = mem::align_of::<usize>();
        if let Ok(layout) = Layout::from_size_align(size, align) {
            unsafe {
                if layout.size() > 0 {
                    let ptr = alloc(layout);
                    if !ptr.is_null() {

                        let mut state = self.list.borrow_mut();
                        state.push(StackItem::String { ptr, len: size });
                        
                        return ptr;
                    }
                } else {
                    //return align
                }
            }
        }

        log::info!("Attempted allocation: {size}");

        panic!("Allocation error");
    }

    pub(crate) fn pop(&self) -> String {
        let mut state = self.list.borrow_mut();
        match state.pop().unwrap() {
            StackItem::String { ptr, len: size } => {

                let data = unsafe {
                    Vec::<u8>::from_raw_parts(ptr, size, size)
                };
                
                String::from_utf8(data).unwrap()
            },
            StackItem::EmptyString => String::from("")
        }

    }
}
