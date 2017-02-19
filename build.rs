extern crate pkg_config;

use std::process::Command;

fn main() {
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
