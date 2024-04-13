extern crate pkg_config;

#[cfg(target_env = "msvc")]
extern crate vcpkg;

use std::env;
#[allow(unused_imports)]
use std::path::PathBuf;

include!("./src/helper.rs");

fn main() {
    let include_dir = if env::var("MYSQLCLIENT_INCLUDE_DIR").is_ok() {
        env::var("MYSQLCLIENT_INCLUDE_DIR").unwrap()
    } else {
        match mysql_config_variable("pkgincludedir") {
            None => "".to_string(),
            Some(var_value) => var_value,
        }
    };

    let (version_id, is_mysql, is_mariadb) = get_libmysql_version_id(include_dir.clone());
    if is_mysql {
        if version_id >= 80010 {
            println!("cargo::rustc-cfg=MySql_Version_AboveOrEqual_8_0_1");
        }
    } else if is_mariadb {
        // MariaDB would Remove my_bool in future?
        println!("cargo::rustc-cfg=mariadb_version_id=\"{}\"", version_id);
    }

    #[cfg(feature = "buildtime_bindgen")]
    let mut use_rust_bindgen = true;
    #[cfg(not(feature = "buildtime_bindgen"))]
    let mut use_rust_bindgen = false;

    if cfg!(target_os = "windows") {
        use_rust_bindgen = false;
    }

    if pkg_config::probe_library("mysqlclient").is_ok() {
        // pkg_config did everything for us
        return;
    } else if try_vcpkg() {
        // vcpkg did everything for us
        return;
    } else if let Ok(path) = env::var("MYSQLCLIENT_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", path);
    } else if let Some(path) = mysql_config_variable("pkglibdir") {
        println!("cargo:rustc-link-search=native={}", path);
    }

    if cfg!(all(windows, target_env = "gnu")) {
        println!("cargo:rustc-link-lib=dylib=mysql");
    } else if cfg!(all(windows, target_env = "msvc")) {
        println!("cargo:rustc-link-lib=static=mysqlclient");
    } else {
        println!("cargo:rustc-link-lib=mysqlclient");
    }

    if use_rust_bindgen {
        let lib_dir = if env::var("MYSQLCLIENT_LIB_DIR").is_ok() {
            env::var("MYSQLCLIENT_LIB_DIR").unwrap()
        } else {
            match mysql_config_variable("pkglibdir") {
                None => "".to_string(),
                Some(var_value) => var_value,
            }
        };

        if autogen_bindings(lib_dir, include_dir) == 0 {
            return;
        }
    }
}

#[cfg(target_env = "msvc")]
fn try_vcpkg() -> bool {
    vcpkg::find_package("libmysql").is_ok()
}

#[cfg(not(target_env = "msvc"))]
fn try_vcpkg() -> bool {
    false
}

#[allow(unused_variables)]
fn autogen_bindings(lib_dir: String, include_dir: String) -> i32 {
    if lib_dir == "" || include_dir == "" {
        return -1;
    }

    println!("cargo::rustc-cfg=UseRustBindgen");

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", lib_dir);

    // Tell cargo to tell rustc to link the system mysqlclient shared library.
    println!("cargo:rustc-link-lib=mysqlclient");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    #[cfg(feature = "buildtime_bindgen")]
    {
        let bindings = bindgen::Builder::default()
            // The input header we would like to generate
            // bindings for.
            .header("wrapper.h")
            // Do not generate any bindings for the given type.
            .blocklist_type("my_bool")
            .blocklist_type("mysql_ssl_mode")
            .blocklist_type("mysql_option")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            // Tell clang to look for header files in the specified directory.
            .clang_arg(format!("-I{include_dir}"))
            // Mark the given enum as a Rust enum.
            .rustified_enum("enum_field_types")
            .rustified_enum("mysql_option")
            .rustified_enum("enum_mysql_set_option")
            .rustified_enum("mysql_ssl_mode")
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    0
}
