use std::{
    ffi::{c_char, CString},
    ptr::{null, null_mut},
};

use crate::{
    sapi::{
        embedded::EmbeddedSapi,
        raw::{get_partial_module_for_c, RawPhpSapi},
    },
    sys::{
        libphp_eval_stringl_ex, libphp_execute_simple_script, libphp_register_constant,
        libphp_register_variable, libphp_zval_create_string, php_module_shutdown,
        php_request_startup, php_rust_clear_server_context, php_rust_init, zend_call_function,
        zend_execute_data, zend_fcall_info, zend_fcall_info_cache, zend_file_handle,
        zend_function_entry, zend_internal_arg_info, zend_register_functions,
        zend_stream_init_filename, zend_type, zval,
    },
    value::Value,
};

pub type FunctionImplementation = unsafe extern "C" fn(*mut zend_execute_data, *mut zval);

pub struct Context<'a, Sapi: crate::sapi::raw::RawPhpSapi = EmbeddedSapi> {
    initd: bool,
    on_init: Option<Box<dyn FnOnce(&mut Context<Sapi>)>>,
    argc: i32,
    argv: Vec<String>,
    bindings: Vec<Value>,
    content: &'a mut Sapi::Context,
}

impl<'a> Context<'a, EmbeddedSapi> {
    /// Create a new PHP execution context.
    pub fn new() -> Self {
        Self {
            initd: false,
            on_init: None,
            argc: 0,
            argv: Vec::new(),
            bindings: Vec::new(),
            // TODO: Free context again
            content: Box::leak(Box::new(())),
        }
    }
}

impl<'a, Sapi: RawPhpSapi> Context<'a, Sapi> {
    /// Create a new PHP execution context.
    pub fn new_with_sapi(content: Box<Sapi::Context>) -> Self {
        Self {
            initd: false,
            on_init: None,
            argc: 0,
            argv: Vec::new(),
            bindings: Vec::new(),
            content: Box::leak(content),
        }
    }

    /// Bind a variable to the PHP context.
    /// The variable will be available in the PHP context as a global variable.
    pub fn bind(&mut self, name: &str, value: impl Into<Value>) {
        let mut value = value.into();
        let var_name_cstr = CString::new(name).unwrap();

        unsafe {
            libphp_register_variable(var_name_cstr.as_ptr(), value.as_mut_ptr());
        }

        self.bindings.push(value);
    }

    /// Define a constant in the PHP context.
    /// The constant will be available in the PHP context as a global constant.
    pub fn define(&mut self, name: &str, value: impl Into<Value>) {
        let mut value = value.into();
        let constant_name_cstr = CString::new(name).unwrap();

        unsafe {
            libphp_register_constant(constant_name_cstr.as_ptr(), value.as_mut_ptr());
        }

        self.bindings.push(value);
    }

    /// Define a new function in the PHP context.
    pub fn define_function(&mut self, name: &str, function: FunctionImplementation) {
        let mut function_entry = zend_function_entry::default();
        let function_name_cstr = CString::new(name).unwrap();

        let mut args: Vec<zend_internal_arg_info> = Vec::new();

        let mut arg = zend_internal_arg_info::default();
        arg.name = null();

        let arg_type = zend_type::default();
        arg.type_ = arg_type;

        args.push(arg);

        function_entry.fname = function_name_cstr.as_ptr();
        function_entry.num_args = 0;
        function_entry.handler = Some(function);
        function_entry.arg_info =
            Box::into_raw(args.into_boxed_slice()) as *const zend_internal_arg_info;

        let mut functions = Vec::new();
        functions.push(function_entry);

        let empty_entry = zend_function_entry::default();
        functions.push(empty_entry);

        unsafe {
            zend_register_functions(null_mut(), functions.as_mut_ptr(), null_mut(), 0);
        }
    }

    /// Specify the number of arguments to pass to the PHP context.
    pub fn argc(&mut self, argc: i32) {
        self.argc = argc;
    }

    /// Specify the arguments to pass to the PHP context.
    pub fn argv(&mut self, argv: Vec<String>) {
        self.argv = argv;
    }

    /// Execute a PHP file.
    pub fn execute_file(&mut self, file: &str, reset_global_ctx: bool) -> Value {
        let mut file_handle = zend_file_handle::default();
        let cstring = CString::new(file).unwrap();

        self.init();

        unsafe {
            zend_stream_init_filename(&mut file_handle, cstring.as_ptr());
        }

        let mut retval_ptr = zval::default();

        unsafe {
            libphp_execute_simple_script(&mut file_handle, &mut retval_ptr, reset_global_ctx);
        }

        Value::new(&retval_ptr)
    }

    /// Evaluate a PHP expression and get the result.
    pub fn result_of(&mut self, expression: &str, clear_globals: bool) -> Value {
        let script_name = CString::new("eval'd code").unwrap();

        self.init();

        let mut retval_ptr = zval::default();

        unsafe {
            libphp_eval_stringl_ex(
                expression.as_ptr() as *const c_char,
                expression.len(),
                &mut retval_ptr as *mut zval,
                script_name.as_ptr(),
                clear_globals,
            );
        }

        self.bindings.clear();

        Value::new(&retval_ptr)
    }

    /// Call a PHP function with no arguments.
    pub fn call(&mut self, name: &str) -> Value {
        let name_cstring = CString::new(name).unwrap();

        self.init();

        let mut retval_ptr = zval::default();

        let mut fcall = zend_fcall_info::default();
        let mut fcall_cache = zend_fcall_info_cache::default();

        unsafe {
            libphp_zval_create_string(&mut fcall.function_name, name_cstring.as_ptr());
        }

        fcall.param_count = 0;
        fcall.object = null_mut();
        fcall.size = std::mem::size_of::<zend_fcall_info>();
        fcall.retval = &mut retval_ptr;

        unsafe {
            zend_call_function(&mut fcall, &mut fcall_cache);
        }

        return Value::new(&retval_ptr);
    }

    /// Call a PHP function with no arguments.
    pub fn call_with(&mut self, name: &str, args: &[impl Into<Value> + Clone]) -> Value {
        let name_cstring = CString::new(name).unwrap();

        self.init();

        // Convert the given arguments into a list of values.
        let mut args = args
            .iter()
            .map(|arg| arg.clone().into())
            .collect::<Vec<Value>>();
        let mut retval_ptr = zval::default();
        let mut fcall = zend_fcall_info::default();
        let mut fcall_cache = zend_fcall_info_cache::default();

        unsafe {
            libphp_zval_create_string(&mut fcall.function_name, name_cstring.as_ptr());
        }

        fcall.param_count = args.len() as u32;
        fcall.params = args.first_mut().unwrap().as_mut_ptr();
        fcall.object = null_mut();
        fcall.size = std::mem::size_of::<zend_fcall_info>();
        fcall.retval = &mut retval_ptr;

        unsafe {
            zend_call_function(&mut fcall, &mut fcall_cache);
        }

        return Value::new(&retval_ptr);
    }

    /// Register a callback to be called when the execution context is initialised.
    pub fn on_init<F: FnOnce(&mut Context<Sapi>) + 'static>(&mut self, callback: F) {
        self.on_init = Some(Box::new(callback));
    }

    /// Initialise the execution context.
    ///
    /// NOTE: This method does not need to be called manually.
    pub fn init(&mut self) {
        if self.initd {
            return;
        }

        unsafe {
            php_rust_init(
                get_partial_module_for_c::<Sapi>(),
                self.content as *mut Sapi::Context as *mut std::ffi::c_void,
                if self.argv.is_empty() {
                    null_mut()
                } else {
                    self.argv.first().unwrap().as_ptr() as *mut i8
                },
            );
        }

        Sapi::on_before_request_init();

        unsafe {
            if php_request_startup() != 0 {
                php_module_shutdown();
                panic!("Failed to start PHP request");
            }
        }

        if let Some(callback) = self.on_init.take() {
            callback(self);
        }

        self.initd = true;
    }

    /// Close the execution context.
    ///
    /// NOTE: This method does not need to be called manually. The execution context is automatically closed when Context is dropped.
    pub fn close(&mut self) {
        if self.initd {
            unsafe { Sapi::shutdown(std::ptr::null_mut()) };
        }
        unsafe {
            php_rust_clear_server_context();
        }
        // Explicitly drop the leaked &mut SapiContext
        drop(unsafe { Box::from_raw(self.content) });
    }
}

impl<Sapi: RawPhpSapi> Drop for Context<'_, Sapi> {
    fn drop(&mut self) {
        self.close();
    }
}
