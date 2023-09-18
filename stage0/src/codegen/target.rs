use std::ffi::CStr;

/// Output target of the code.
pub struct Target<T: AsRef<CStr>> {
    triple: T,
}

impl<T: AsRef<CStr>> Target<T> {
    pub fn new(triple: T) -> Self {
        Self { triple }
    }

    pub fn triple(&self) -> &CStr {
        self.triple.as_ref()
    }
}
