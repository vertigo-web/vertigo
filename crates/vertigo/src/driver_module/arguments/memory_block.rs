use std::alloc::{alloc, Layout};
use std::mem;

fn alloc_memory(size: usize) -> (*mut u8, Layout) {

    let align = mem::align_of::<usize>();

    if let Ok(layout) = Layout::from_size_align(size, align) {
        unsafe {
            if layout.size() > 0 {
                let ptr = alloc(layout);

                if !ptr.is_null() {
                    return (ptr, layout);
                }
            } else {
                //return align
            }
        }
    }

    log::info!("Attempted allocation: {size}");

    panic!("Allocation error");
}

#[derive(Debug)]
pub struct MemoryBlock {
    ptr: *mut u8,
    layout: Layout,
    size: u32,
}

impl MemoryBlock {
    pub fn new(size: u32) -> MemoryBlock {
        let (ptr, layout) = alloc_memory(size as usize);

        MemoryBlock {
            ptr,
            layout,
            size,
        }
    }

    pub fn get_ptr_and_size(&self) -> (u32, u32) {
        (self.ptr as u32, self.size as u32)
    }

    pub fn get_ptr(&self) -> u32 {
        self.ptr as u32
    }

    pub fn write(&self, offset: u32, data: &[u8]) {
        let data_len = data.len() as u32;

        if offset + data_len <= self.size {
            let dest = unsafe { self.ptr.offset(offset as isize) };

            unsafe {
                std::ptr::copy(data.as_ptr(), dest, data_len as usize);
            }

        } else {
            panic!("Buffer overflow size={} offset={} new_data={}", self.size, offset, data_len);
        }
    }

    pub fn convert_to_vec(self) -> Vec<u8> {
        let ptr = self.ptr;
        let size = self.size as usize;

        std::mem::forget(self);

        unsafe {
            Vec::<u8>::from_raw_parts(ptr, size, size)
        }
    }
}

impl Drop for MemoryBlock {
    fn drop(&mut self) {
        use std::alloc::{dealloc};

        unsafe {
            dealloc(self.ptr, self.layout);
        }
    }
}

/*
Inne sposoby alokacji i dealokacji:
https://www.hellorust.com/demos/sha1/index.html
*/
