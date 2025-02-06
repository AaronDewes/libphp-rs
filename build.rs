#![allow(unused_variables)]

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use bindgen::Builder;

const PHP_VERSION: &str = "8.4";

fn main() {
    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=src/wrapper.c");
    println!("cargo:rerun-if-changed=src/php-sapi.h");
    println!("cargo:rerun-if-changed=src/php-sapi.c");
    println!("cargo:rerun-if-env-change=PHP_VERSION");

    
    let extensions = [
        "opcache",
        #[cfg(feature = "amqp")]
        "amqp",
        #[cfg(feature = "apcu")]
        "apcu",
        #[cfg(feature = "ast")]
        "ast",
        #[cfg(feature = "bcmath")]
        "bcmath",
        #[cfg(feature = "bz2")]
        "bz2",
        #[cfg(feature = "calendar")]
        "calendar",
        #[cfg(feature = "ctype")]
        "ctype",
        #[cfg(feature = "curl")]
        "curl",
        #[cfg(feature = "dba")]
        "dba",
        #[cfg(feature = "dio")]
        "dio",
        #[cfg(feature = "dom")]
        "dom",
        #[cfg(feature = "ds")]
        "ds",
        #[cfg(feature = "enchant")]
        "enchant",
        #[cfg(feature = "event")]
        "event",
        #[cfg(feature = "exif")]
        "exif",
        #[cfg(feature = "ffi")]
        "ffi",
        #[cfg(feature = "fileinfo")]
        "fileinfo",
        #[cfg(feature = "filter")]
        "filter",
        #[cfg(feature = "ftp")]
        "ftp",
        #[cfg(feature = "gd")]
        "gd",
        #[cfg(feature = "gettext")]
        "gettext",
        #[cfg(feature = "glfw")]
        "glfw",
        #[cfg(feature = "gmp")]
        "gmp",
        #[cfg(feature = "gmssl")]
        "gmssl",
        #[cfg(feature = "grpc")]
        "grpc",
        #[cfg(feature = "iconv")]
        "iconv",
        #[cfg(feature = "igbinary")]
        "igbinary",
        #[cfg(feature = "imagick")]
        "imagick",
        #[cfg(feature = "imap")]
        "imap",
        #[cfg(feature = "inotify")]
        "inotify",
        #[cfg(feature = "intl")]
        "intl",
        #[cfg(feature = "ldap")]
        "ldap",
        #[cfg(feature = "libxml")]
        "libxml",
        #[cfg(feature = "mbregex")]
        "mbregex",
        #[cfg(feature = "mbstring")]
        "mbstring",
        #[cfg(feature = "memcache")]
        "memcache",
        // TODO: mcrypt is not supported by static-php-cli yet
        // #[cfg(feature = "mcrypt")] "mcrypt",
        #[cfg(feature = "memcached")]
        "memcached",
        #[cfg(feature = "mongodb")]
        "mongodb",
        #[cfg(feature = "msgpack")]
        "msgpack",
        #[cfg(feature = "mysqli")]
        "mysqli",
        #[cfg(feature = "mysqlnd")]
        "mysqlnd",
        // TODO: oci8 is not supported by static-php-cli yet
        //#[cfg(feature = "oci8")]
        //"oci8",
        #[cfg(feature = "openssl")]
        "openssl",
        #[cfg(feature = "opentelemetry")]
        "opentelemetry",
        #[cfg(feature = "parallel")]
        "parallel",
        #[cfg(feature = "password-argon2")]
        "password-argon2",
        #[cfg(feature = "pcntl")]
        "pcntl",
        #[cfg(feature = "pdo")]
        "pdo",
        #[cfg(feature = "pdo_mysql")]
        "pdo_mysql",
        #[cfg(feature = "pdo_pgsql")]
        "pdo_pgsql",
        #[cfg(feature = "pdo_sqlite")]
        "pdo_sqlite",
        #[cfg(feature = "pdo_sqlsrv")]
        "pdo_sqlsrv",
        #[cfg(feature = "pgsql")]
        "pgsql",
        #[cfg(feature = "phar")]
        "phar",
        #[cfg(feature = "posix")]
        "posix",
        #[cfg(feature = "protobuf")]
        "protobuf",
        #[cfg(feature = "rar")]
        "rar",
        #[cfg(feature = "rdkafka")]
        "rdkafka",
        #[cfg(feature = "readline")]
        "readline",
        #[cfg(feature = "redis")]
        "redis",
        #[cfg(feature = "session")]
        "session",
        #[cfg(feature = "shmop")]
        "shmop",
        #[cfg(feature = "simdjson")]
        "simdjson",
        #[cfg(feature = "simplexml")]
        "simplexml",
        #[cfg(feature = "snappy")]
        "snappy",
        #[cfg(feature = "soap")]
        "soap",
        #[cfg(feature = "sockets")]
        "sockets",
        #[cfg(feature = "sodium")]
        "sodium",
        #[cfg(feature = "spx")]
        "spx",
        #[cfg(feature = "sqlite3")]
        "sqlite3",
        #[cfg(feature = "sqlsrv")]
        "sqlsrv",
        #[cfg(feature = "ssh2")]
        "ssh2",
        #[cfg(feature = "swoole")]
        "swoole",
        #[cfg(feature = "swoole-hook-mysql")]
        "swoole-hook-mysql",
        #[cfg(feature = "swoole-hook-pgsql")]
        "swoole-hook-pgsql",
        #[cfg(feature = "swoole-hook-sqlite")]
        "swoole-hook-sqlite",
        #[cfg(feature = "swow")]
        "swow",
        #[cfg(feature = "sysvmsg")]
        "sysvmsg",
        #[cfg(feature = "sysvsem")]
        "sysvsem",
        #[cfg(feature = "sysvshm")]
        "sysvshm",
        #[cfg(feature = "tidy")]
        "tidy",
        #[cfg(feature = "tokenizer")]
        "tokenizer",
        #[cfg(feature = "uuid")]
        "uuid",
        #[cfg(feature = "uv")]
        "uv",
        // TODO: xdebug is not supported by static-php-cli yet
        // #[cfg(feature = "xdebug")]
        // "xdebug",
        #[cfg(feature = "xhprof")]
        "xhprof",
        #[cfg(feature = "xlswriter")]
        "xlswriter",
        #[cfg(feature = "xml")]
        "xml",
        #[cfg(feature = "xmlreader")]
        "xmlreader",
        #[cfg(feature = "xmlwriter")]
        "xmlwriter",
        #[cfg(feature = "xsl")]
        "xsl",
        #[cfg(feature = "yac")]
        "yac",
        #[cfg(feature = "yaml")]
        "yaml",
        #[cfg(feature = "zip")]
        "zip",
        #[cfg(feature = "zlib")]
        "zlib",
        #[cfg(feature = "zstd")]
        "zstd",
    ].join(",");

    if !target_exists("spc") {
        std::fs::create_dir_all(target_dir("spc")).unwrap();
        run_command_or_fail(target_dir("spc"), "git", &["init"]);
        run_command_or_fail(
            target_dir("spc"),
            "git",
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/crazywhalecc/static-php-cli.git",
            ],
        );
        run_command_or_fail(
            target_dir("spc"),
            "git",
            &[
                "fetch",
                "origin",
                "daa6196afc6090417f073d123728758ae6d117f4",
            ],
        );
        run_command_or_fail(
            target_dir("spc"),
            "git",
            &["checkout", "daa6196afc6090417f073d123728758ae6d117f4"],
        );
        std::env::set_var("SPC_NO_MUSL_PATH", "yes");
        run_command_or_fail(
            target_dir("spc"),
            "composer",
            &["update", "--no-dev", "-n", "--no-plugins"],
        );
        run_command_or_fail(
            target_dir("spc"),
            "php",
            &[
                "bin/spc",
                "download",
                "php-src,pkg-config,micro",
                format!("--with-php={}", PHP_VERSION).as_str(),
                format!("--for-extensions={}", extensions).as_str(),
            ],
        );
        run_command_or_fail(
            target_dir("spc"),
            "php",
            &["bin/spc", "doctor", "--auto-fix"],
        );

        run_command_or_fail(
            target_dir("spc"),
            "php",
            &[
                "bin/spc",
                "--libc=glibc",
                "build",
                &extensions,
                "--build-embed",
                "--enable-zts",
            ],
        );
    }

    let include_dir = target_dir("spc/buildroot/include/php");
    let lib_dir = target_dir("spc/buildroot/lib");

    println!("cargo:rustc-link-lib=static=php");
    println!("cargo:rustc-link-search=native={}", lib_dir);

    let includes = ["/", "Zend", "/main", "/TSRM"]
        .iter()
        .map(|folder| format!("-I{}/{}", &include_dir, &folder))
        .collect::<Vec<String>>();

    let bindings = Builder::default()
        .clang_args(&includes)
        .derive_default(true)
        .derive_debug(true)
        .allowlist_var("PHP_OUTPUT_HANDLER_STDFLAGS")
        .allowlist_type("zval")
        .allowlist_type("zend_constant")
        .allowlist_type("zend_fcall_info")
        .allowlist_type("sapi_header_struct")
        .allowlist_type("sapi_headers_struct")
        .allowlist_type("sapi_header_op_enum")
        .allowlist_type("sapi_module_struct")
        .allowlist_type("zend_llist_position")
        .allowlist_type("zend_stat_t")
        .allowlist_type("partial_sapi_module_struct")
        .allowlist_function("php_module_startup")
        .allowlist_function("zend_llist_get_first_ex")
        .allowlist_function("zend_llist_get_next_ex")
        .allowlist_function("zend_string_init")
        .allowlist_function("zend_call_function")
        .allowlist_function("_zend_new_array")
        .allowlist_function("zend_array_count")
        .allowlist_function("zend_hash_get_current_key_type_ex")
        .allowlist_function("zend_hash_get_current_key_zval_ex")
        .allowlist_function("zend_hash_get_current_data_ex")
        .allowlist_function("zend_hash_move_forward_ex")
        .allowlist_function("zend_eval_string_ex")
        .allowlist_function("php_rust_init")
        .allowlist_function("php_request_shutdown")
        .allowlist_function("php_module_shutdown")
        .allowlist_function("sapi_shutdown")
        .allowlist_function("tsrm_shutdown")
        .allowlist_function("zend_compile_string")
        .allowlist_function("zend_get_type")
        .allowlist_function("zval_ptr_dtor")
        .allowlist_function("zend_stream_init_filename")
        .allowlist_function("php_execute_script")
        .allowlist_function("php_execute_simple_script")
        .allowlist_function("php_register_variable_ex")
        .allowlist_function("zend_register_functions")
        .allowlist_function("php_output_activate")
        .allowlist_function("php_output_start_user")
        .allowlist_function("php_output_end_all")
        .allowlist_function("php_output_deactivate")
        .allowlist_function("php_handle_aborted_connection")
        .allowlist_function("php_rust_clear_server_context")
        .allowlist_function("php_rust_set_tmp_server_ctx")
        .allowlist_function("php_rust_set_server_context")
        .allowlist_function("php_register_variable_safe")
        .allowlist_function("libphp_zval_addref_p")
        .allowlist_function("libphp_zval_delref_p")
        .allowlist_function("zend_hash_add")
        .allowlist_function("zend_hash_next_index_insert")
        .allowlist_type("zend_function_entry")
        .header("src/wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .file("src/wrapper.c")
        .file("src/rust-sapi.c")
        .includes(
            &includes
                .iter()
                .map(|s| s.as_str()[2..].to_string())
                .collect::<Vec<String>>(),
        )
        .flag("-fPIC")
        .flag("-m64")
        .static_flag(true)
        .compile("wrapper");
}

fn target_dir(path: &str) -> String {
    let out_dir = env::var("OUT_DIR").unwrap();
    format!("{}/{}", out_dir, path)
}

fn target_exists(path: &str) -> bool {
    Path::new(target_dir(path).as_str()).exists()
}

fn run_command_or_fail(dir: String, cmd: &str, args: &[&str]) {
    let fmt_cmd = format!("{} {}", cmd, args.join(" "));
    println!("Running command: \"{}\" in dir: {}", &fmt_cmd, dir);
    let ret = Command::new(cmd).current_dir(dir).args(args).status();
    match ret.map(|status| (status.success(), status.code())) {
        Ok((true, _)) => (),
        Ok((false, Some(c))) => panic!("Command failed with error code {} [cmd] {}", c, &fmt_cmd),
        Ok((false, None)) => panic!("Command got killed [cmd] {}", &fmt_cmd),
        Err(e) => panic!("Command failed with error: {} [cmd] {}", e, &fmt_cmd),
    }
}
