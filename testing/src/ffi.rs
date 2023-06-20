use std::ffi::{c_char, CStr};
use std::slice;

pub type PyTuple = *const f32;

pub trait PyTupleToSlice {
    fn to_slice(self) -> [f32; 2];
    fn to_tile(self) -> [usize; 2];
}

impl PyTupleToSlice for PyTuple {
    fn to_slice(self) -> [f32; 2] {
        unsafe { slice::from_raw_parts(self, 2).try_into().unwrap() }
    }

    fn to_tile(self) -> [usize; 2] {
        let [x, y] = self.to_slice();
        [x as usize, y as usize]
    }
}

pub type PyString = *const c_char;

pub trait PyStringToString {
    fn to_str(self) -> &'static str;
    fn to_string(self) -> String;
}

impl PyStringToString for PyString {
    fn to_str(self) -> &'static str {
        unsafe { CStr::from_ptr(self).to_str().unwrap() }
    }

    fn to_string(self) -> String {
        self.to_str().to_string()
    }
}
