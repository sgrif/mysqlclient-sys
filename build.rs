extern crate pkg_config;
extern crate bindgen;

use bindgen::Builder;
use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    generate_bindgen_file();

    if pkg_config::probe_library("mysqlclient").is_ok() {
        // pkg_config did everything for us
    } else {
        if let Some(path) = mysql_config_variable("pkglibdir") {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rustc-link-lib=mysqlclient");
        }
    }
}

fn mysql_config_variable(var_name: &str) -> Option<String> {
    Command::new("mysql_config")
        .arg(format!("--variable={}", var_name))
        .output()
        .into_iter()
        .filter(|output| output.status.success())
        .flat_map(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_string())
        .next()
}

fn generate_bindgen_file() {
    let out_dir = env::var("OUT_DIR").unwrap();
    Builder::default()
        .no_unstable_rust()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", mysql_include_dir()))
        .whitelisted_function("mysql.*")
        .whitelisted_type("MYSQL.*")
        .whitelisted_var("MYSQL.*")
        .generate()
        .expect("Unable to generate bindings for libmysqlclient")
        .write_to_file(Path::new(&out_dir).join("bindings.rs"))
        .expect("Unable to write bindings to file");
}

fn mysql_include_dir() -> String {
    pkg_config::get_variable("mysqlclient", "includedir")
        .ok()
        .or_else(|| mysql_config_variable("pkgincludedir"))
        .expect("Unable to locate `mysql.h`")
}
