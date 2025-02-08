/*
   +----------------------------------------------------------------------+
   | Copyright (c) The PHP Group                                          |
   +----------------------------------------------------------------------+
   | This source file is subject to version 3.01 of the PHP license,      |
   | that is bundled with this package in the file LICENSE, and is        |
   | available through the world-wide-web at the following url:           |
   | https://www.php.net/license/3_01.txt                                 |
   | If you did not receive a copy of the PHP license and are unable to   |
   | obtain it through the world-wide-web, please send a note to          |
   | license@php.net so we can mail you a copy immediately.               |
   +----------------------------------------------------------------------+
   | Author: Edin Kadribasic <edink@php.net>                              |
   +----------------------------------------------------------------------+
*/

#include "sapi/embed/php_embed.h"
#include "rust-sapi.h"
#include "ext/standard/php_standard.h"
#include "ext/standard/dl_arginfo.h"

#ifdef PHP_WIN32
#include <io.h>
#include <fcntl.h>
#endif

static const char HARDCODED_INI[] =
	"html_errors=0\n"
	"register_argc_argv=1\n"
	"implicit_flush=1\n"
	"output_buffering=0\n"
	"max_execution_time=0\n"
	"max_input_time=-1\n\0";

#if defined(PHP_WIN32) && defined(ZTS)
ZEND_TSRMLS_CACHE_DEFINE()
#endif

static const zend_function_entry additional_functions[] = {
	ZEND_FE(dl, arginfo_dl)
		ZEND_FE_END};

// Global var that contains a pointer to the server context
void *global_server_context;

EMBED_SAPI_API int php_rust_init(struct partial_sapi_module_struct module, int argc, char **argv, void *server_context)
{
#if defined(SIGPIPE) && defined(SIG_IGN)
	signal(SIGPIPE, SIG_IGN); /* ignore SIGPIPE in standalone mode so
								 that sockets created via fsockopen()
								 don't kill PHP if the remote site
								 closes it.  in apache|apxs mode apache
								 does that for us!  thies@thieso.net
								 20000419 */
#endif

	php_tsrm_startup();
#ifdef PHP_WIN32
	ZEND_TSRMLS_CACHE_UPDATE();
#endif

	zend_signal_startup();

	global_server_context = server_context;

	sapi_module_struct php_rust_module = {
		module.name,
		module.pretty_name,

		module.startup,
		module.shutdown,

		module.activate,
		module.deactivate,

		module.ub_write,
		module.flush,
		module.get_stat,
		module.getenv,

		php_error,

		/*module.header_handler,
		module.send_headers,*/
		NULL,
		NULL,
		module.send_header,

		module.read_post,
		module.read_cookies,

		module.register_server_variables,
		module.log_message,
		module.get_request_time,
		module.terminate_process,

		STANDARD_SAPI_MODULE_PROPERTIES};

	/* SAPI initialization (SINIT)
	 *
	 * Initialize the SAPI globals (memset to 0). After this point we can set
	 * SAPI globals via the SG() macro.
	 *
	 * Reentrancy startup.
	 *
	 * This also sets 'php_rust_module.ini_entries = NULL' so we cannot
	 * allocate the INI entries until after this call.
	 */
	sapi_startup(&php_rust_module);

#ifdef PHP_WIN32
	_fmode = _O_BINARY;					 /*sets default for file streams to binary */
	_setmode(_fileno(stdin), O_BINARY);	 /* make the stdio mode be binary */
	_setmode(_fileno(stdout), O_BINARY); /* make the stdio mode be binary */
	_setmode(_fileno(stderr), O_BINARY); /* make the stdio mode be binary */
#endif

	/* This hard-coded string of INI settings is parsed and read into PHP's
	 * configuration hash table at the very end of php_init_config(). This
	 * means these settings will overwrite any INI settings that were set from
	 * an INI file.
	 *
	 * To provide overwritable INI defaults, hook the ini_defaults function
	 * pointer that is part of the sapi_module_struct
	 * (php_rust_module.ini_defaults).
	 *
	 *     void (*ini_defaults)(HashTable *configuration_hash);
	 *
	 * This callback is invoked as soon as the configuration hash table is
	 * allocated so any INI settings added via this callback will have the
	 * lowest precedence and will allow INI files to overwrite them.
	 */
	php_rust_module.ini_entries = HARDCODED_INI;

	/* SAPI-provided functions. */
	php_rust_module.additional_functions = additional_functions;

	if (argv)
	{
		php_rust_module.executable_location = argv[0];
	}

	/* Module initialization (MINIT) */
	if (php_rust_module.startup(&php_rust_module) == FAILURE)
	{
		return FAILURE;
	}

	/* Do not chdir to the script's directory. This is akin to calling the CGI
	 * SAPI with '-C'.
	 */
	SG(options) |= SAPI_OPTION_NO_CHDIR;

	SG(request_info).argc = argc;
	SG(request_info).argv = argv;

	/* Request initialization (RINIT) */
	if (php_request_startup() == FAILURE)
	{
		php_module_shutdown();
		return FAILURE;
	}

	/*SG(headers_sent) = 1;
	SG(request_info).no_headers = 1;*/
	php_register_variable("PHP_SELF", "-", NULL);

	return SUCCESS;
}

EMBED_SAPI_API void php_rust_shutdown(void)
{
	/* Request shutdown (RSHUTDOWN) */
	php_request_shutdown((void *)0);

	/* Module shutdown (MSHUTDOWN) */
	php_module_shutdown();

	/* SAPI shutdown (SSHUTDOWN) */
	sapi_shutdown();

	tsrm_shutdown();
}

void php_rust_clear_server_context()
{
	global_server_context = NULL;
	//SG(server_context) = NULL;
}

void php_rust_set_server_context()
{
	SG(server_context) = global_server_context;
}

sapi_request_info* php_rust_get_request_info() {
	return &SG(request_info);
}