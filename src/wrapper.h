#include "Zend/zend.h"
#include "Zend/zend_API.h"
#include "main/php.h"
#include "rust-sapi.h"
#include "Zend/zend_compile.h"
#include <Zend/zend_types.h>
#include <Zend/zend_observer.h>
#include <Zend/zend_exceptions.h>
#include <ext/standard/php_var.h>
#include "zend_smart_str.h"
#include "main/php_variables.h"

uint8_t libphp_zval_get_type(const zval*);

const char* libphp_zval_get_string(zval*);

const char* libphp_var_export(zval *pz);

void libphp_zval_create_string(zval *pz, const char *str);
void libphp_zval_create_long(zval *pz, long l);

zend_string* libphp_zend_string_init();

void libphp_register_variable(const char *key, zval *value);

void libphp_register_constant(const char *name, zval *value);

uint32_t libphp_zval_addref_p(zval* pz);
uint32_t libphp_zval_delref_p(zval* pz);

zend_result libphp_eval_stringl_ex(const char *str, size_t str_len, zval *retval_ptr, const char *string_name, bool reset_global_ctx);
zend_result libphp_zend_execute_script(int type, zval *retval, zend_file_handle *file_handle, bool reset_global_ctx);
zend_result libphp_zend_execute_scripts(int type, zval *retval, int file_count, bool reset_global_ctx, ...);
int libphp_execute_simple_script(zend_file_handle *primary_file, zval *ret, bool reset_global_ctx);
