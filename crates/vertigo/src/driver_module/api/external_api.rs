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

        pub fn safe_panic_message(long_ptr: u64) {
            unsafe { panic_message(long_ptr) }
        }

        pub fn safe_dom_access(long_ptr: u64) -> u64 {
            unsafe { dom_access(long_ptr) }
        }
    }
}

#[cfg(any(test, not(target_arch = "wasm32"), not(target_os = "unknown")))]
pub mod api {
    pub mod safe_wrappers {
        pub fn safe_panic_message(_long_ptr: u64) {}

        pub fn safe_dom_access(_long_ptr: u64) -> u64 {
            0
        }
    }
}
