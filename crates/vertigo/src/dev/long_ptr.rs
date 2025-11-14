#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct LongPtr(u64);

impl LongPtr {
    pub fn from(long: u64) -> LongPtr {
        LongPtr(long)
    }

    pub fn new(ptr: u32, offset: u32) -> Self {
        LongPtr(((ptr as u64) << 32) + (offset as u64))
    }

    pub fn into_parts(self) -> (u32, u32) {
        let LongPtr(long_ptr) = self;

        let ptr = (long_ptr >> 32) as u32;
        let offset = long_ptr as u32;
        (ptr, offset)
    }

    pub fn is_undefined(&self) -> bool {
        self.0 == 0
    }

    pub fn get_long_ptr(self) -> u64 {
        self.0
    }
}
