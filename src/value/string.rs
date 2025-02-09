use std::ffi::CString;

use crate::sys::{libphp_zend_string_init, libphp_zval_create_string, zend_string, zval};

use super::Value;

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        let mut zval = zval::default();
        let string = value.to_string();
        let cstr = CString::new(string).unwrap();

        unsafe {
            libphp_zval_create_string(&mut zval, cstr.as_ptr());
        }

        Self::new(&zval)
    }
}

// str is memcopyed to zend_string, so we don't have to worry about the lifetime of the string.
pub fn create_zend_str(str: &str) -> *mut zend_string {
    let cstr = CString::new(str).unwrap();
    unsafe { libphp_zend_string_init(cstr.as_ptr()) }
}
