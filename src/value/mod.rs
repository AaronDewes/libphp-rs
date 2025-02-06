use std::{
    ffi::CStr,
    fmt::{Debug, Display},
    ptr::NonNull,
};

use crate::sys::{
    libphp_var_export, libphp_zval_addref_p, libphp_zval_delref_p, libphp_zval_get_string, libphp_zval_get_type, zend_refcounted, zval, zval_ptr_dtor, HashTable, IS_ARRAY, IS_DOUBLE, IS_FALSE, IS_LONG, IS_NULL, IS_STRING, IS_TRUE
};

use self::array::Array;

pub mod array;
mod int;
mod string;

pub use string::create_zend_str;

#[derive(Clone)]
enum InnerValue {
    Owned(Box<zval>),
    Borrowed(NonNull<zval>),
}

impl InnerValue {
    fn as_ref(&self) -> &zval {
        match self {
            InnerValue::Owned(zval) => zval,
            InnerValue::Borrowed(zval) => unsafe { zval.as_ref() },
        }
    }

    fn as_ptr(&self) -> *const zval {
        match self {
            InnerValue::Owned(zval) => zval.as_ref(),
            InnerValue::Borrowed(zval) => zval.as_ptr(),
        }
    }
    
    fn as_mut_ptr(&mut self) -> *mut zval {
        match self {
            InnerValue::Owned(zval) => zval.as_mut(),
            InnerValue::Borrowed(zval) => zval.as_ptr(),
        }
    }
}

pub struct Value {
    inner: InnerValue,
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self.inner {
            InnerValue::Owned(ref zval) => Self {
                // TODO: Is this correct?
                inner: InnerValue::Owned(zval.clone()),
            },
            InnerValue::Borrowed(ref zval) => unsafe {
                libphp_zval_addref_p(zval.as_ptr());
                Self {
                    inner: InnerValue::Borrowed(zval.clone()),
                }
            },
        }
    }
}

impl Value {
    /// Create a new Value from an existing zval.
    pub fn new(zval: &zval) -> Self {
        Self {
            inner: InnerValue::Owned(Box::new(*zval)),
        }
    }

    pub fn new_maybe_gc(zval: NonNull<zval>) -> Self {
        let zval_inner = unsafe { zval.as_ref() };
        unsafe {
            // TODO: Check if this is correct
            if zval_inner.value.counted.is_null() || zval_inner.value.lval == (zval_inner.value.counted.addr() as i64) {
                return Self {
                    inner: InnerValue::Owned(Box::new(*zval.as_ptr())),
                };
            } else {
                let zval_inner_counted = zval_inner.value.counted;
                
                dbg!((*zval_inner_counted).gc.u.type_info);
                println!("Aligned");
            }
        }
        unsafe {
            libphp_zval_addref_p(zval.as_ptr());
        }
        Self {
            inner: InnerValue::Borrowed(zval),
        }
    }

    /// Get the type byte that represents the type of the value.
    pub fn get_type(&self) -> u8 {
        unsafe { libphp_zval_get_type(self.inner.as_ref()) }
    }

    /// Check if the value is an integer (long).
    pub fn is_int(&self) -> bool {
        self.get_type() == IS_LONG
    }

    /// Check if the value is a float (double).
    pub fn is_float(&self) -> bool {
        self.get_type() == IS_DOUBLE
    }

    /// Check if the value is null.
    pub fn is_null(&self) -> bool {
        self.get_type() == IS_NULL
    }

    /// Check if the value is a string.
    pub fn is_string(&self) -> bool {
        self.get_type() == IS_STRING
    }

    /// Check if the value is true.
    pub fn is_true(&self) -> bool {
        self.get_type() == IS_TRUE
    }

    /// Check if the value is false.
    pub fn is_false(&self) -> bool {
        self.get_type() == IS_FALSE
    }

    /// Check if the value is a boolean.
    pub fn is_bool(&self) -> bool {
        self.is_true() || self.is_false()
    }

    /// Check if the value is an array.
    pub fn is_array(&self) -> bool {
        self.get_type() == IS_ARRAY
    }

    /// Check a raw pointer to the underlying zval.
    pub fn as_ptr(&self) -> *const zval {
        self.inner.as_ptr()
    }

    /// Check a mutable raw pointer to the underlying zval.
    pub fn as_mut_ptr(&mut self) -> *mut zval {
        self.inner.as_mut_ptr()
    }

    /// Convert the value to a string.
    ///
    /// WARNING: This method will panic if the PHP string is not valid UTF-8.
    pub fn as_str(&self) -> &str {
        unsafe {
            let cstr = CStr::from_ptr(libphp_zval_get_string(self.inner.as_ref()));
            cstr.to_str().unwrap()
        }
    }

    /// Convert the value to a slice of bytes.
    ///
    /// WARNING: This method will panic if the PHP string is not valid UTF-8.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let cstr = CStr::from_ptr(libphp_zval_get_string(self.inner.as_ptr()));
            cstr.to_bytes()
        }
    }

    /// Convert the value to a C string (const char*).
    pub fn as_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(libphp_zval_get_string(self.inner.as_ptr())) }
    }

    /// Convert the value to a 64-bit integer.
    pub fn to_int(&self) -> i64 {
        unsafe { (self.inner.as_ref()).value.lval }
    }

    /// Convert the value to a 64-bit floating point number.
    pub fn to_float(&self) -> f64 {
        unsafe { (self.inner.as_ref()).value.dval }
    }

    /// Convert the value to an Array.
    pub fn to_array<'a>(&'a self) -> Array<'a> {
        let arr: &'a mut HashTable = unsafe { (self.inner.as_ref()).value.arr.as_mut() }.unwrap();
        arr.into()
    }

    /// Convert the value to null (unit type).
    ///
    /// NOTE: This method only exists for consistency, there's no reason to use it.
    pub fn to_null(&self) {}

    /// Get a pretty name for the type of the value.
    pub fn get_type_name(&self) -> &'static str {
        match self.get_type() {
            IS_LONG => "int",
            IS_DOUBLE => "float",
            IS_NULL => "null",
            IS_STRING => "string",
            _ => "unknown",
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let var_exported = unsafe { libphp_var_export(self.inner.as_ref()) };

        write!(f, "{}", unsafe {
            CStr::from_ptr(var_exported).to_string_lossy()
        })
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            match &mut self.inner {
                InnerValue::Owned(ref mut zval) => {
                    zval_ptr_dtor(zval.as_mut());
                }
                InnerValue::Borrowed(ref zval) => {
                    libphp_zval_delref_p(zval.as_ptr());
                }
            }
        }
    }
}
