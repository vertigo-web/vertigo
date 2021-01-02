use alloc::string::String;

#[derive(Clone)]
pub struct VDomText {
    pub value: String,
}

impl VDomText {
    pub fn new<T: Into<String>>(value: T) -> VDomText {
        VDomText {
            value: value.into()
        }
    }
}

