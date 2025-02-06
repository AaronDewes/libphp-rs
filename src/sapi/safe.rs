use std::{ffi::CStr, ptr::NonNull};

use crate::{
    sys::{
        libphp_zval_addref_p, php_register_variable_ex, php_register_variable_safe,
        php_rust_set_server_context, sapi_module_struct,
    },
    value::Value,
};

use super::raw::RawPhpSapi;

#[derive(Debug, Clone, Copy)]
pub enum SapiHeaderOp {
    REPLACE = 0,
    ADD = 1,
    DELETE = 2,
    DELETE_ALL = 3,
    SET_STATUS = 4,
}

impl TryFrom<i32> for SapiHeaderOp {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SapiHeaderOp::REPLACE),
            1 => Ok(SapiHeaderOp::ADD),
            2 => Ok(SapiHeaderOp::DELETE),
            3 => Ok(SapiHeaderOp::DELETE_ALL),
            4 => Ok(SapiHeaderOp::SET_STATUS),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Headers {
    pub headers: Vec<String>,
    pub http_response_code: i32,
    pub send_default_content_type: bool,
    pub mime_type: Option<String>,
    pub http_status_line: Option<String>,
}

pub struct TrackVarsArray(NonNull<crate::sys::zval>);

impl TrackVarsArray {
    fn new(ptr: NonNull<crate::sys::zval>) -> Self {
        unsafe {
            libphp_zval_addref_p(ptr.as_ptr());
        }
        Self(ptr)
    }

    pub unsafe fn get_ptr(&mut self) -> *mut crate::sys::zval {
        self.0.as_ptr()
    }

    pub fn insert(&mut self, key: &str, val: &str) {
        let key = std::ffi::CString::new(key).unwrap();
        let val_cstr = std::ffi::CString::new(val).unwrap();
        unsafe {
            php_register_variable_safe(key.as_ptr(), val_cstr.as_ptr(), val.len(), self.0.as_ptr());
        }
    }

    pub fn insert_val(&mut self, key: &str, mut val: Value) {
        let key = std::ffi::CString::new(key).unwrap();
        unsafe {
            php_register_variable_ex(key.as_ptr(), val.as_mut_ptr(), self.0.as_ptr());
        }
    }

    pub fn as_value(&self) -> Value {
        Value::new_maybe_gc(self.0)
    }
}

impl Drop for TrackVarsArray {
    fn drop(&mut self) {
        unsafe {
            crate::sys::libphp_zval_delref_p(self.0.as_ptr());
        }
    }
}

pub trait Sapi {
    type Context;

    const name: *const std::ffi::c_char;

    const pretty_name: *const std::ffi::c_char;

    fn startup(module: *mut sapi_module_struct) -> i32;
    fn shutdown() -> i32;
    fn activate() -> i32;
    fn deactivate() -> i32;
    fn ub_write(str: &str) -> usize;
    fn flush(ctx: &mut Self::Context);
    fn get_stat() -> *mut crate::sys::zend_stat_t;
    fn getenv(name: &str) -> &Option<String>;
    fn send_header(header: String, ctx: Option<&mut Self::Context>);
    fn read_post(buffer: &mut [u8]) -> usize;
    fn read_cookies() -> String;
    fn register_server_variables(track_vars_array: &mut TrackVarsArray);
    fn get_request_time() -> f64;
    fn terminate_process();
    fn log_message(message: &str, syslog_type_int: i32);
}

unsafe impl<T: Sapi> RawPhpSapi for T {
    type Context = T::Context;
    const name: *const std::ffi::c_char = T::name;

    const pretty_name: *const std::ffi::c_char = T::pretty_name;

    unsafe extern "C" fn startup(module: *mut crate::sys::sapi_module_struct) -> std::ffi::c_int {
        T::startup(module)
    }

    unsafe extern "C" fn shutdown(_module: *mut sapi_module_struct) -> std::ffi::c_int {
        T::shutdown()
    }

    unsafe extern "C" fn activate() -> std::ffi::c_int {
        tracing::debug!("activate");
        php_rust_set_server_context();
        T::activate()
    }

    unsafe extern "C" fn deactivate() -> std::ffi::c_int {
        T::deactivate()
    }

    unsafe extern "C" fn ub_write(str: *const std::ffi::c_char, size: usize) -> usize {
        //dbg!(str, size);
        let slice = unsafe { std::slice::from_raw_parts(str as *const u8, size) };
        if let Ok(string) = std::str::from_utf8(slice) {
            T::ub_write(string)
        } else {
            println!("Failed to convert to utf8");
            size
        }
    }

    unsafe extern "C" fn flush(server_context: *mut std::ffi::c_void) {
        let ctx = server_context as *mut T::Context;
        let ctx = NonNull::new(ctx);
        if let Some(mut ctx) = ctx {
            T::flush(ctx.as_mut());
        } else {
            tracing::debug!("server_context is null");
        }
    }

    unsafe extern "C" fn get_stat() -> *mut crate::sys::zend_stat_t {
        T::get_stat()
    }

    unsafe extern "C" fn getenv(
        name: *const std::ffi::c_char,
        name_len: usize,
    ) -> *mut std::ffi::c_char {
        let slice = unsafe { std::slice::from_raw_parts(name as *const u8, name_len) };
        let string = std::str::from_utf8(slice).unwrap();
        if let Some(var) = T::getenv(string) {
            let var_mem = unsafe {
                std::alloc::alloc(
                    std::alloc::Layout::from_size_align(var.len() + 1, std::mem::align_of::<u8>())
                        .expect("Failed to create layout"),
                )
            };
            std::ptr::copy(var.as_ptr(), var_mem, var.len());
            std::ptr::write(var_mem.add(var.len()), 0);
            var_mem as *mut std::ffi::c_char
        } else {
            std::ptr::null_mut()
        }
    }

    /*unsafe extern "C" fn header_handler(
        sapi_header: *mut crate::sys::sapi_header_struct,
        op: crate::sys::sapi_header_op_enum,
        sapi_headers: *mut crate::sys::sapi_headers_struct,
    ) -> std::ffi::c_int {
        let header = if !sapi_header.is_null() {
            let header = unsafe { &*sapi_header };
            let slice = unsafe {
                std::slice::from_raw_parts(header.header as *const u8, header.header_len)
            };
            Some(std::str::from_utf8(slice).unwrap().to_string())
        } else {
            None
        };

        let headers = if !sapi_headers.is_null() {
            Some(load_sapi_headers(sapi_headers))
        } else {
            None
        };

        T::header_handler(header, SapiHeaderOp::try_from(op as i32).unwrap(), headers)
    }

    unsafe extern "C" fn send_headers(
        sapi_header: *mut crate::sys::sapi_headers_struct,
    ) -> std::ffi::c_int {
        let headers = load_sapi_headers(sapi_header);
        T::send_headers(headers)
    }*/

    unsafe extern "C" fn send_header(
        sapi_header: *mut crate::sys::sapi_header_struct,
        server_context: *mut std::ffi::c_void,
    ) -> () {
        let sapi_header = NonNull::new(sapi_header);
        let ctx = server_context as *mut T::Context;
        let mut ctx = NonNull::new(ctx);
        if let Some(sapi_header) = sapi_header {
            let sapi_header = sapi_header.as_ref();
            let header = unsafe {
                if sapi_header.header.is_null() {
                    tracing::debug!("sapi_header.header is null");
                    return;
                }
                let cstr = CStr::from_ptr(sapi_header.header);
                cstr.to_str().unwrap()
            };
            T::send_header(header.to_string(), ctx.map(|mut ctx| ctx.as_mut()));
        } else {
            tracing::debug!("sapi_header is null");
        }
    }

    unsafe extern "C" fn read_post(buffer: *mut std::ffi::c_char, count: usize) -> usize {
        let slice = unsafe { std::slice::from_raw_parts_mut(buffer as *mut u8, count) };
        T::read_post(slice)
    }

    unsafe extern "C" fn read_cookies() -> *mut std::ffi::c_char {
        let cookies = T::read_cookies();
        let cookie_mem = unsafe {
            std::alloc::alloc(
                std::alloc::Layout::from_size_align(cookies.len() + 1, std::mem::align_of::<u8>())
                    .expect("Failed to create layout"),
            )
        };
        std::ptr::copy(cookies.as_ptr(), cookie_mem, cookies.len());
        std::ptr::write(cookie_mem.add(cookies.len()), 0);
        cookie_mem as *mut std::ffi::c_char
    }

    unsafe extern "C" fn register_server_variables(track_vars_array: *mut crate::sys::zval) -> () {
        let mut track_vars_array = TrackVarsArray::new(NonNull::new(track_vars_array).unwrap());
        T::register_server_variables(&mut track_vars_array)
    }

    unsafe extern "C" fn get_request_time(
        req_time: *mut std::ffi::c_double,
    ) -> crate::sys::ZEND_RESULT_CODE {
        unsafe {
            *req_time = T::get_request_time();
        }
        crate::sys::ZEND_RESULT_CODE_SUCCESS
    }

    unsafe extern "C" fn terminate_process() -> () {
        todo!("terminate_process not implemented");
    }

    unsafe extern "C" fn log_message(
        message: *const std::ffi::c_char,
        syslog_type_int: std::ffi::c_int,
    ) -> () {
        let slice = unsafe { std::ffi::CStr::from_ptr(message) };
        let string = slice.to_str().unwrap();
        T::log_message(string, syslog_type_int)
    }
}

/*fn load_sapi_headers(sapi_headers: *mut sapi_headers_struct) -> Headers {
    if sapi_headers.is_null() {
        panic!("sapi_headers is null");
    }
    let headers = unsafe { &mut *sapi_headers };
    let mut headers_vec = Vec::new();
    let mut pos: zend_llist_position = std::ptr::null_mut();
    let mut tmp_entry =
        unsafe { zend_llist_get_first_ex(&mut headers.headers, &mut pos) } as *mut zval;

    while !tmp_entry.is_null() {
        let header = unsafe {
            let cstr = CStr::from_ptr(libphp_zval_get_string(tmp_entry));
            cstr.to_str().unwrap()
        };
        headers_vec.push(header.to_string());
        tmp_entry = unsafe { zend_llist_get_next_ex(&mut headers.headers, &mut pos) as *mut zval };
    }

    Headers {
        headers: headers_vec,
        http_response_code: headers.http_response_code,
        send_default_content_type: headers.send_default_content_type != 0,
        mime_type: if headers.mimetype.is_null() {
            None
        } else {
            unsafe {
                Some(std::ffi::CStr::from_ptr(headers.mimetype)
                    .to_string_lossy()
                    .into_owned())
            }
        },
        http_status_line: if headers.http_status_line.is_null() {
            None
        } else {
            unsafe {
                Some(std::ffi::CStr::from_ptr(headers.http_status_line)
                    .to_string_lossy()
                    .into_owned())
            }
        },
    }
}*/
