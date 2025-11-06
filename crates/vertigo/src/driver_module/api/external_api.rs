#[cfg(all(not(test), target_arch = "wasm32", target_os = "unknown"))]
pub mod api {
    mod inner {
        #[link(wasm_import_module = "mod")]
        extern "C" {
            pub fn panic_message(long_ptr: u64);
            pub fn dom_access(long_ptr: u64) -> u64;
        }
    }

    pub mod safe_wrappers {
        use super::inner::*;
        use crate::LongPtr;

        pub fn safe_panic_message(long_ptr: LongPtr) {
            let long_ptr = long_ptr.get_long_ptr();
            unsafe { panic_message(long_ptr) }
        }

        pub fn safe_dom_access(long_ptr: LongPtr) -> LongPtr {
            let long_ptr = long_ptr.get_long_ptr();
            let result = unsafe { dom_access(long_ptr) };
            LongPtr::from(result)
        }
    }
}

#[cfg(any(test, not(target_arch = "wasm32"), not(target_os = "unknown")))]
pub mod api {
    pub mod safe_wrappers {
        use crate::LongPtr;

        pub fn safe_panic_message(_long_ptr: LongPtr) {}

        pub fn safe_dom_access(_long_ptr: LongPtr) -> LongPtr {
            LongPtr::from(0)
        }
    }
}
