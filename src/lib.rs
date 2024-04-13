#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

#[cfg(not(windows))]
#[cfg(not(UseRustBindgen))]
include!("bindings_macos.rs");

#[cfg(windows)]
#[cfg(not(UseRustBindgen))]
include!("bindings_windows.rs");

#[cfg(UseRustBindgen)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod helper;

#[allow(unused_imports)]
use std::env;

// MySQL 8.0.1 Remove my_bool
// https://bugs.mysql.com/bug.php?id=85131
#[cfg(UseRustBindgen)]
#[cfg(MySql_Version_AboveOrEqual_8_0_1)]
pub type my_bool = bool;
#[cfg(UseRustBindgen)]
#[cfg(not(MySql_Version_AboveOrEqual_8_0_1))]
pub type my_bool = ::std::os::raw::c_char;

#[cfg(UseRustBindgen)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum mysql_ssl_mode {
    SSL_MODE_DISABLED = 1,
    SSL_MODE_PREFERRED = 2,
    SSL_MODE_REQUIRED = 3,
    SSL_MODE_VERIFY_CA = 4,
    SSL_MODE_VERIFY_IDENTITY = 5,
}

#[cfg(UseRustBindgen)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum mysql_option {
    MYSQL_OPT_CONNECT_TIMEOUT = 0,
    MYSQL_OPT_COMPRESS = 1,
    MYSQL_OPT_NAMED_PIPE = 2,
    MYSQL_INIT_COMMAND = 3,
    MYSQL_READ_DEFAULT_FILE = 4,
    MYSQL_READ_DEFAULT_GROUP = 5,
    MYSQL_SET_CHARSET_DIR = 6,
    MYSQL_SET_CHARSET_NAME = 7,
    MYSQL_OPT_LOCAL_INFILE = 8,
    MYSQL_OPT_PROTOCOL = 9,
    MYSQL_SHARED_MEMORY_BASE_NAME = 10,
    MYSQL_OPT_READ_TIMEOUT = 11,
    MYSQL_OPT_WRITE_TIMEOUT = 12,
    MYSQL_OPT_USE_RESULT = 13,
    MYSQL_REPORT_DATA_TRUNCATION = 14,
    MYSQL_OPT_RECONNECT = 15,
    MYSQL_PLUGIN_DIR = 16,
    MYSQL_DEFAULT_AUTH = 17,
    MYSQL_OPT_BIND = 18,
    MYSQL_OPT_SSL_KEY = 19,
    MYSQL_OPT_SSL_CERT = 20,
    MYSQL_OPT_SSL_CA = 21,
    MYSQL_OPT_SSL_CAPATH = 22,
    MYSQL_OPT_SSL_CIPHER = 23,
    MYSQL_OPT_SSL_CRL = 24,
    MYSQL_OPT_SSL_CRLPATH = 25,
    MYSQL_OPT_CONNECT_ATTR_RESET = 26,
    MYSQL_OPT_CONNECT_ATTR_ADD = 27,
    MYSQL_OPT_CONNECT_ATTR_DELETE = 28,
    MYSQL_SERVER_PUBLIC_KEY = 29,
    MYSQL_ENABLE_CLEARTEXT_PLUGIN = 30,
    MYSQL_OPT_CAN_HANDLE_EXPIRED_PASSWORDS = 31,
    MYSQL_OPT_MAX_ALLOWED_PACKET = 32,
    MYSQL_OPT_NET_BUFFER_LENGTH = 33,
    MYSQL_OPT_TLS_VERSION = 34,
    MYSQL_OPT_SSL_MODE = 35,
    MYSQL_OPT_GET_SERVER_PUBLIC_KEY = 36,
    MYSQL_OPT_RETRY_COUNT = 37,
    MYSQL_OPT_OPTIONAL_RESULTSET_METADATA = 38,
    MYSQL_OPT_SSL_FIPS_MODE = 39,
    MYSQL_OPT_TLS_CIPHERSUITES = 40,
    MYSQL_OPT_COMPRESSION_ALGORITHMS = 41,
    MYSQL_OPT_ZSTD_COMPRESSION_LEVEL = 42,
    MYSQL_OPT_LOAD_DATA_LOCAL_DIR = 43,
    MYSQL_OPT_USER_PASSWORD = 44,
    MYSQL_OPT_SSL_SESSION_DATA = 45,
    MYSQL_OPT_TLS_SNI_SERVERNAME = 46,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_libmysql_version_id() {
        let include_dir = if env::var("MYSQLCLIENT_INCLUDE_DIR").is_ok() {
            env::var("MYSQLCLIENT_INCLUDE_DIR").unwrap()
        } else {
            match helper::mysql_config_variable("pkgincludedir") {
                None => "".to_string(),
                Some(var_value) => var_value,
            }
        };

        let (version_id, is_mysql, is_mariadb) = helper::get_libmysql_version_id(include_dir);
        if 0 == version_id {
            assert!(
                0 == version_id && false == is_mysql && false == is_mariadb,
                "Invalid MySQL Version ID"
            );
        }

        let version_str = match helper::mysql_config("version", "") {
            None => "".to_string(),
            Some(var_value) => var_value,
        };
        if is_mariadb {
            assert!(version_str.starts_with("11"));
        }
    }
}
