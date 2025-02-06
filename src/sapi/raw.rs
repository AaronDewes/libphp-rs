use std::ffi::{c_char, c_double, c_int, c_void};

use crate::sys::{
    partial_sapi_module_struct, sapi_header_struct, sapi_module_struct, zend_result, zend_stat_t,
    zval,
};

pub unsafe trait RawPhpSapi {
    type Context;

    const name: *const c_char;
    const pretty_name: *const c_char;

    unsafe extern "C" fn startup(module: *mut sapi_module_struct) -> c_int;
    unsafe extern "C" fn shutdown(module: *mut sapi_module_struct) -> c_int;

    unsafe extern "C" fn activate() -> c_int;
    unsafe extern "C" fn deactivate() -> c_int;

    unsafe extern "C" fn ub_write(str: *const c_char, size: usize) -> usize;
    unsafe extern "C" fn flush(server_context: *mut c_void);

    unsafe extern "C" fn get_stat() -> *mut zend_stat_t;
    unsafe extern "C" fn getenv(name: *const c_char, name_len: usize) -> *mut c_char;

    // TODO: Error handler would require variadic functions, which are currently only supported in nightly Rust

    unsafe extern "C" fn send_header(
        sapi_header: *mut sapi_header_struct,
        _server_context: *mut c_void,
    ) -> ();

    unsafe extern "C" fn read_post(buffer: *mut c_char, count: usize) -> usize;
    unsafe extern "C" fn read_cookies() -> *mut c_char;

    unsafe extern "C" fn register_server_variables(track_vars_array: *mut zval) -> ();
    unsafe extern "C" fn get_request_time(req_time: *mut c_double) -> zend_result;
    unsafe extern "C" fn terminate_process() -> ();

    unsafe extern "C" fn log_message(message: *const c_char, syslog_type_int: c_int) -> ();
}

pub fn get_partial_module_for_c<Sapi: RawPhpSapi>() -> partial_sapi_module_struct {
    partial_sapi_module_struct {
        name: Sapi::name as *mut c_char,
        pretty_name: Sapi::pretty_name as *mut c_char,
        startup: Some(Sapi::startup),
        shutdown: Some(Sapi::shutdown),
        activate: Some(Sapi::activate),
        deactivate: Some(Sapi::deactivate),
        ub_write: Some(Sapi::ub_write),
        flush: Some(Sapi::flush),
        get_stat: Some(Sapi::get_stat),
        getenv: Some(Sapi::getenv),
        send_header: Some(Sapi::send_header),
        read_post: Some(Sapi::read_post),
        read_cookies: Some(Sapi::read_cookies),
        register_server_variables: Some(Sapi::register_server_variables),
        log_message: Some(Sapi::log_message),
        get_request_time: Some(Sapi::get_request_time),
        terminate_process: Some(Sapi::terminate_process),
    }
}
