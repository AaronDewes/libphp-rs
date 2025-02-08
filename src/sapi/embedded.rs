use std::io::Write;

use super::safe::{Sapi, TrackVarsArray};
use crate::sys::{
    php_module_shutdown, php_module_startup, php_request_shutdown, sapi_shutdown,
};


pub struct EmbeddedSapi;

impl Sapi for EmbeddedSapi {
    type Context = ();

    const name: *const std::ffi::c_char = c"rust-embedded".as_ptr();

    const pretty_name: *const std::ffi::c_char = c"Rust Embedded".as_ptr();

    fn startup(module: *mut crate::sys::sapi_module_struct) -> i32 {
        unsafe { php_module_startup(module, std::ptr::null_mut()) }
    }

    fn shutdown() -> i32 {
        unsafe {
            /* Request shutdown (RSHUTDOWN) */
            php_request_shutdown(std::ptr::null_mut());

            /* Module shutdown (MSHUTDOWN) */
            php_module_shutdown();

            /* SAPI shutdown (SSHUTDOWN) */
            sapi_shutdown();
            
            #[cfg(feature = "zts")]
            crate::sys::tsrm_shutdown();
        }
        0
    }

    fn activate() -> i32 {
        0
    }

    fn deactivate() -> i32 {
        0
    }

    fn ub_write(str: &str) -> usize {
        std::io::stdout().write(str.as_bytes()).unwrap()
    }

    fn flush(_ctx: &mut Self::Context) {
        std::io::stdout().flush().unwrap();
    }

    fn get_stat() -> *mut crate::sys::zend_stat_t {
        dbg!("get_stat");
        todo!()
    }

    fn getenv(name: &str) -> &Option<String> {
        // TODO: Don't leak memory
        Box::leak(Box::new(std::env::var(name).ok()))
    }

    /*fn header_handler(
        header: Option<String>,
        op: super::safe::SapiHeaderOp,
        headers: Option<super::safe::Headers>,
    ) -> i32 {
        dbg!(header, op, headers);
        0
    }

    fn send_headers(headers: super::safe::Headers) -> i32 {
        eprintln!("send_headers: {:?}", headers);
        0
    }*/

    fn send_header(_header: String, _ctx: Option<&mut Self::Context>) {
        //println!("send_header: {:?}", header);
    }

    fn read_post(_buffer: &mut [u8]) -> usize {
        0
    }

    fn read_cookies() -> String {
        "".to_string()
    }

    fn register_server_variables(_track_vars_array: &mut TrackVarsArray) {
        // Empty
    }

    fn get_request_time() -> f64 {
        tracing::debug!("get_request_time");
        0.0
    }

    fn terminate_process() {
        todo!()
    }

    fn log_message(_message: &str, _syslog_type_int: i32) {
        todo!()
    }
}
