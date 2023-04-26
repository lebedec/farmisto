use std::ffi::{c_char, CStr};

pub type PyString = *const c_char;

pub trait PyStringToString {
    fn to_string(self) -> String;
}

impl PyStringToString for PyString {
    fn to_string(self) -> String {
        unsafe { CStr::from_ptr(self).to_str().unwrap().to_string() }
    }
}
