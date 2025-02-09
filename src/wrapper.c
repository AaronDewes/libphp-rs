#include "wrapper.h"

uint8_t libphp_zval_get_type(const zval* pz) {
    return zval_get_type(pz);
}

const char *libphp_zval_get_string(zval *pz)
{
    convert_to_string(pz);
    return Z_STRVAL_P(pz);
}

zend_string* libphp_zend_string_init(const char *str)
{
    return zend_string_init(str, strlen(str), 0);
}

const char* libphp_var_export(zval *pz) 
{
    smart_str buf = {0};
    php_var_export_ex(pz, 1, &buf);
    smart_str_0(&buf);

    const char* exported = buf.s->val;
    smart_str_free(&buf); 

    return exported;
}

void libphp_zval_create_string(zval *pz, const char *str)
{
    ZVAL_STRING_FAST(pz, str);
}

void libphp_zval_create_long(zval *pz, long l)
{
    ZVAL_LONG(pz, l);
}

void libphp_register_variable(const char *key, zval *value)
{
    zend_hash_str_update(&EG(symbol_table), key, strlen(key), value);
}

void libphp_register_constant(const char *name, zval *value)
{
    zend_constant c;

    ZVAL_COPY(&c.value, value);
    ZEND_CONSTANT_SET_FLAGS(&c, CONST_CS | CONST_PERSISTENT, 0);

    c.name = zend_string_init_interned(name, strlen(name), 1);

    zend_register_constant(&c);
}

uint32_t libphp_zval_addref_p(zval* pz) {
	return Z_ADDREF_P(pz);
}

uint32_t libphp_zval_delref_p(zval* pz) {
	return Z_DELREF_P(pz);
}

static zend_always_inline void i_init_code_execute_data(zend_execute_data *execute_data, zend_op_array *op_array, zval *return_value) /* {{{ */
{
	ZEND_ASSERT(EX(func) == (zend_function*)op_array);

	EX(opline) = op_array->opcodes;
	EX(call) = NULL;
	EX(return_value) = return_value;

	if (op_array->last_var) {
		zend_attach_symbol_table(execute_data);
	}

	if (!ZEND_MAP_PTR(op_array->run_time_cache)) {
		void *ptr;

		ZEND_ASSERT(op_array->fn_flags & ZEND_ACC_HEAP_RT_CACHE);
		ptr = emalloc(op_array->cache_size);
		ZEND_MAP_PTR_INIT(op_array->run_time_cache, ptr);
		memset(ptr, 0, op_array->cache_size);
	}
	EX(run_time_cache) = RUN_TIME_CACHE(op_array);

	EG(current_execute_data) = execute_data;
}

void libphp_execute(zend_op_array *op_array, zval *return_value, bool reset_global_ctx)
{
	zend_execute_data *execute_data;
	void *object_or_called_scope;
	uint32_t call_info;

	if (EG(exception) != NULL) {
		return;
	}

	object_or_called_scope = zend_get_this_object(EG(current_execute_data));
	if (EXPECTED(!object_or_called_scope)) {
		object_or_called_scope = zend_get_called_scope(EG(current_execute_data));
		call_info = ZEND_CALL_TOP_CODE | ZEND_CALL_HAS_SYMBOL_TABLE;
	} else {
		call_info = ZEND_CALL_TOP_CODE | ZEND_CALL_HAS_SYMBOL_TABLE | ZEND_CALL_HAS_THIS;
	}
	execute_data = zend_vm_stack_push_call_frame(call_info,
		(zend_function*)op_array, 0, object_or_called_scope);
	if (EG(current_execute_data) && reset_global_ctx) {
		execute_data->symbol_table = zend_rebuild_symbol_table();
	} else {
		execute_data->symbol_table = &EG(symbol_table);
	}
	EX(prev_execute_data) = EG(current_execute_data);
	i_init_code_execute_data(execute_data, op_array, return_value);
	ZEND_OBSERVER_FCALL_BEGIN(execute_data);
	zend_execute_ex(execute_data);
	/* Observer end handlers are called from ZEND_RETURN */
	zend_vm_stack_free_call_frame(execute_data);
}



zend_result libphp_eval_stringl(const char *str, size_t str_len, zval *retval_ptr, const char *string_name, bool reset_global_ctx) /* {{{ */
{
	zend_op_array *new_op_array;
	uint32_t original_compiler_options;
	zend_result retval;
	zend_string *code_str;

	if (retval_ptr) {
		code_str = zend_string_concat3(
			"return ", sizeof("return ")-1, str, str_len, ";", sizeof(";")-1);
	} else {
		code_str = zend_string_init(str, str_len, 0);
	}

	/*printf("Evaluating '%s'\n", pv.value.str.val);*/

	original_compiler_options = CG(compiler_options);
	CG(compiler_options) = ZEND_COMPILE_DEFAULT_FOR_EVAL;
	new_op_array = zend_compile_string(code_str, string_name, ZEND_COMPILE_POSITION_AFTER_OPEN_TAG);
	CG(compiler_options) = original_compiler_options;

	if (new_op_array) {
		zval local_retval;

		EG(no_extensions)=1;

		new_op_array->scope = zend_get_executed_scope();

		zend_try {
			ZVAL_UNDEF(&local_retval);
			libphp_execute(new_op_array, &local_retval, reset_global_ctx);
		} zend_catch {
			destroy_op_array(new_op_array);
			efree_size(new_op_array, sizeof(zend_op_array));
			zend_bailout();
		} zend_end_try();

		if (Z_TYPE(local_retval) != IS_UNDEF) {
			if (retval_ptr) {
				ZVAL_COPY_VALUE(retval_ptr, &local_retval);
			} else {
				zval_ptr_dtor(&local_retval);
			}
		} else {
			if (retval_ptr) {
				ZVAL_NULL(retval_ptr);
			}
		}

		EG(no_extensions)=0;
		zend_destroy_static_vars(new_op_array);
		destroy_op_array(new_op_array);
		efree_size(new_op_array, sizeof(zend_op_array));
		retval = SUCCESS;
	} else {
		retval = FAILURE;
	}
	zend_string_release(code_str);
	return retval;
}

zend_result libphp_eval_stringl_ex(const char *str, size_t str_len, zval *retval_ptr, const char *string_name, bool reset_global_ctx)
{
	zend_result result;

	result = libphp_eval_stringl(str, str_len, retval_ptr, string_name, reset_global_ctx);
	if (EG(exception)) {
		result = zend_exception_error(EG(exception), E_ERROR);
	}
	return result;
}

bool libphp_execute_script_ex(zend_file_handle *primary_file, zval *retval, bool reset_global_ctx)
{
	zend_file_handle *prepend_file_p = NULL, *append_file_p = NULL;
	zend_file_handle prepend_file, append_file;
#ifdef HAVE_BROKEN_GETCWD
	volatile int old_cwd_fd = -1;
#else
	char *old_cwd;
	ALLOCA_FLAG(use_heap)
#endif
	bool result = true;

#ifndef HAVE_BROKEN_GETCWD
# define OLD_CWD_SIZE 4096
	old_cwd = do_alloca(OLD_CWD_SIZE, use_heap);
	old_cwd[0] = '\0';
#endif

	zend_try {
		char realfile[MAXPATHLEN];

#ifdef PHP_WIN32
		if(primary_file->filename) {
			UpdateIniFromRegistry(ZSTR_VAL(primary_file->filename));
		}
#endif

		PG(during_request_startup) = 0;

		if (primary_file->filename && !(SG(options) & SAPI_OPTION_NO_CHDIR)) {
#ifdef HAVE_BROKEN_GETCWD
			/* this looks nasty to me */
			old_cwd_fd = open(".", 0);
#else
			php_ignore_value(VCWD_GETCWD(old_cwd, OLD_CWD_SIZE-1));
#endif
			VCWD_CHDIR_FILE(ZSTR_VAL(primary_file->filename));
		}

		/* Only lookup the real file path and add it to the included_files list if already opened
		 *   otherwise it will get opened and added to the included_files list in libphp_zend_execute_scripts
		 */
		if (primary_file->filename &&
			!zend_string_equals_literal(primary_file->filename, "Standard input code") &&
			primary_file->opened_path == NULL &&
			primary_file->type != ZEND_HANDLE_FILENAME
		) {
			if (expand_filepath(ZSTR_VAL(primary_file->filename), realfile)) {
				primary_file->opened_path = zend_string_init(realfile, strlen(realfile), 0);
				zend_hash_add_empty_element(&EG(included_files), primary_file->opened_path);
			}
		}

		if (PG(auto_prepend_file) && PG(auto_prepend_file)[0]) {
			zend_stream_init_filename(&prepend_file, PG(auto_prepend_file));
			prepend_file_p = &prepend_file;
		}

		if (PG(auto_append_file) && PG(auto_append_file)[0]) {
			zend_stream_init_filename(&append_file, PG(auto_append_file));
			append_file_p = &append_file;
		}
		if (PG(max_input_time) != -1) {
#ifdef PHP_WIN32
			zend_unset_timeout();
#endif
			zend_set_timeout(INI_INT("max_execution_time"), 0);
		}

		if (prepend_file_p && result) {
			result = libphp_zend_execute_script(ZEND_REQUIRE, NULL, prepend_file_p, reset_global_ctx) == SUCCESS;
		}
		if (result) {
			result = libphp_zend_execute_script(ZEND_REQUIRE, retval, primary_file, reset_global_ctx) == SUCCESS;
		}
		if (append_file_p && result) {
			result = libphp_zend_execute_script(ZEND_REQUIRE, NULL, append_file_p, reset_global_ctx) == SUCCESS;
		}
	} zend_catch {
		result = false;
	} zend_end_try();

	if (prepend_file_p) {
		zend_destroy_file_handle(prepend_file_p);
	}

	if (append_file_p) {
		zend_destroy_file_handle(append_file_p);
	}

	if (EG(exception)) {
		zend_try {
			zend_exception_error(EG(exception), E_ERROR);
		} zend_end_try();
	}

#ifdef HAVE_BROKEN_GETCWD
	if (old_cwd_fd != -1) {
		fchdir(old_cwd_fd);
		close(old_cwd_fd);
	}
#else
	if (old_cwd[0] != '\0') {
		php_ignore_value(VCWD_CHDIR(old_cwd));
	}
	free_alloca(old_cwd, use_heap);
#endif
	return result;
}

bool libphp_execute_script(zend_file_handle *primary_file, bool reset_global_ctx)
{
	return libphp_execute_script_ex(primary_file, NULL, reset_global_ctx);
}

int libphp_execute_simple_script(zend_file_handle *primary_file, zval *ret, bool reset_global_ctx)
{
	char *old_cwd;
	ALLOCA_FLAG(use_heap)

	EG(exit_status) = 0;
#define OLD_CWD_SIZE 4096
	old_cwd = do_alloca(OLD_CWD_SIZE, use_heap);
	old_cwd[0] = '\0';

	zend_try {
#ifdef PHP_WIN32
		if(primary_file->filename) {
			UpdateIniFromRegistry(ZSTR_VAL(primary_file->filename));
		}
#endif

		PG(during_request_startup) = 0;

		if (primary_file->filename && !(SG(options) & SAPI_OPTION_NO_CHDIR)) {
			php_ignore_value(VCWD_GETCWD(old_cwd, OLD_CWD_SIZE-1));
			VCWD_CHDIR_FILE(ZSTR_VAL(primary_file->filename));
		}
		libphp_zend_execute_scripts(ZEND_REQUIRE, ret, reset_global_ctx, primary_file);
	} zend_end_try();

	if (old_cwd[0] != '\0') {
		php_ignore_value(VCWD_CHDIR(old_cwd));
	}

	free_alloca(old_cwd, use_heap);
	return EG(exit_status);
}

zend_result libphp_zend_execute_script(int type, zval *retval, zend_file_handle *file_handle, bool reset_global_ctx)
{
	zend_op_array *op_array = zend_compile_file(file_handle, type);
	if (file_handle->opened_path) {
		zend_hash_add_empty_element(&EG(included_files), file_handle->opened_path);
	}

	zend_result ret = SUCCESS;
	if (op_array) {
		libphp_execute(op_array, retval, reset_global_ctx);
		zend_exception_restore();
		if (UNEXPECTED(EG(exception))) {
			if (Z_TYPE(EG(user_exception_handler)) != IS_UNDEF) {
				zend_user_exception_handler();
			}
			if (EG(exception)) {
				ret = zend_exception_error(EG(exception), E_ERROR);
			}
		}
		zend_destroy_static_vars(op_array);
		destroy_op_array(op_array);
		efree_size(op_array, sizeof(zend_op_array));
	} else if (type == ZEND_REQUIRE) {
		ret = FAILURE;
	}

	return ret;
}

zend_result libphp_zend_execute_scripts(int type, zval *retval, int file_count, bool reset_global_ctx, ...) /* {{{ */
{
	va_list files;
	int i;
	zend_file_handle *file_handle;
	zend_result ret = SUCCESS;

	va_start(files, file_count);
	for (i = 0; i < file_count; i++) {
		file_handle = va_arg(files, zend_file_handle *);
		if (!file_handle) {
			continue;
		}
		if (ret == FAILURE) {
			continue;
		}
		ret = libphp_zend_execute_script(type, retval, file_handle, reset_global_ctx);
	}
	va_end(files);

	return ret;
}