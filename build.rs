use std::env;
use std::process::Command;

fn main() {
    if cfg!(feature = "bundled") {
        parse_version("8.4.0");
        return;
    }
    if cfg!(feature = "buildtime_bindgen") {
        autogen_bindings();
    }
    let target = std::env::var("TARGET")
        .expect("Set by cargo")
        .to_ascii_uppercase()
        .replace("-", "_");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_VERSION");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIB");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_STATIC");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_VERSION_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIB_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_STATIC_{target}");
    let link_specifier = if env::var("MYSQLCLIENT_STATIC")
        .or(env::var(format!("MYSQLCLIENT_STATIC_{target}")))
        .is_ok()
    {
        "static="
    } else {
        ""
    };
    if let Ok(lib) = pkg_config::probe_library("mysqlclient") {
        // pkg_config did everything but the version flags for us
        parse_version(&lib.version);
        return;
    } else if let Ok(lib) = pkg_config::probe_library("libmariadb") {
        // pkg_config did everything but the version flags for us
        parse_version(&lib.version);
        return;
    } else if try_vcpkg() {
        // vcpkg did everything for us
        if let Ok(version) =
            env::var("MYSQLCLIENT_VERSION").or(env::var(format!("MYSQLCLIENT_VERSION_{target}")))
        {
            parse_version(&version);
            return;
        }
    } else if let Ok(path) =
        env::var("MYSQLCLIENT_LIB_DIR").or(env::var(format!("MYSQLCLIENT_LIB_DIR_{target}")))
    {
        println!("cargo:rustc-link-search=native={path}");
        println!("cargo:rustc-link-lib={link_specifier}mysqlclient");
        if let Ok(version) =
            env::var("MYSQLCLIENT_VERSION").or(env::var(format!("MYSQLCLIENT_VERSION_{target}")))
        {
            parse_version(&version);
            return;
        }
    } else if let Some(output) = mysql_config_variable("--libs") {
        let parts = output.split_ascii_whitespace().collect::<Vec<_>>();
        assert_eq!(
            parts.len(),
            2,
            "Unexpected output from mysql_config: `{output}`"
        );
        let lib = parts[1]
            .strip_prefix("-l")
            .unwrap_or_else(|| panic!("Unexpected output from mysql_config: {output}"));
        let path = parts[0]
            .strip_prefix("-L")
            .unwrap_or_else(|| panic!("Unexpected output from mysql_config: {output}"));
        println!("cargo:rustc-link-search=native={path}");
        println!("cargo:rustc-link-lib={link_specifier}{lib}");
        if let Some(version) = mysql_config_variable("--version") {
            parse_version(&version);
            return;
        }
    }
    panic!(
        "Did not find a compatible version of libmysqlclient.\n\
            Ensure that you installed one and teached mysqlclient-sys how to find it\n\
            You have the following options for that:\n\
            \n\
            * Use `pkg_config` to automatically detect the right location\n\
            * Use vcpkg to automatically detect the right location. \n\
              You also need to set `MYSQLCLIENT_VERSION` to specify which\n\
              version of libmysqlclient you are using\n\
            * Set the `MYSQLCLIENT_LIB_DIR` and `MYSQLCLIENT_VERSION` environment \n\
              variables to point the compiler to the right directory and specify \n\
              which version is used\n\
            * Make the `mysql_config` binary avaible in the environment that invokes\n\
              the compiler"
    );
}

fn mysql_config_variable(var_name: &str) -> Option<String> {
    Command::new("mysql_config")
        .arg(var_name)
        .output()
        .into_iter()
        .filter(|output| output.status.success())
        .flat_map(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_string())
        .next()
}

fn parse_version(version: &str) {
    // we have bundled bindings for:
    // * x86_64 for windows, linux and macos
    // * x86 for windows
    // * aarch64 for linux + macos
    if !(matches!(
        std::env::var("CARGO_CFG_TARGET_ARCH").as_deref(),
        Ok("x86_64")
    ) || (std::env::var("CARGO_CFG_WINDOWS").is_ok()
        && matches!(std::env::var("CARGO_CFG_TARGET_ARCH").as_deref(), Ok("x86")))
        || std::env::var("CARGO_CFG_WINDOWS").is_err()
            && matches!(
                std::env::var("CARGO_CFG_TARGET_ARCH").as_deref(),
                Ok("aarch64")
            ))
    {
        panic!(
            "mysqlclient-sys does not provide bundled bindings for {}\n\
                Consider using the `buildtime_bindgen` feature or \
                contribute bindings for your target to the crate",
            std::env::var("TARGET").expect("Set by cargo")
        );
    }
    println!("cargo::rustc-check-cfg=cfg(mysql_5_7_x)");
    println!("cargo::rustc-check-cfg=cfg(mysql_8_0_x)");
    println!("cargo::rustc-check-cfg=cfg(mysql_8_3_x)");
    println!("cargo::rustc-check-cfg=cfg(mysql_8_4_x)");
    println!("cargo::rustc-check-cfg=cfg(mariadb_10_x)");
    // ubuntu/debian packages use the following package versions:
    // libmysqlclient20 -> 5.7.x
    // libmysqlclient21 -> 8.0.x
    // libmysqlclient23 -> 8.3.0
    // libmysqlclient24 -> 8.4.0
    // libmariadb-dev 3.3.8 -> mariadb 10.x
    if version.starts_with("5.7.") || version.starts_with("20.") {
        if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
            panic!(
                "mysqlclient-sys does not provide bundled bindings \
                 for libmysqlclient {version} on windows. \n\
                 Consider using the `buildtime_bindgen` feature or \
                 contribute bindings to the crate"
            );
        }
        println!("cargo:rustc-cfg=mysql_5_7_x");
    } else if version.starts_with("8.0.") || version == "21" || version.starts_with("21.") {
        println!("cargo:rustc-cfg=mysql_8_0_x");
    } else if version.starts_with("8.3.") || version == "23" || version.starts_with("23.") {
        println!("cargo:rustc-cfg=mysql_8_3_x");
    } else if version.starts_with("8.4.") || version == "24" || version.starts_with("24.") {
        println!("cargo:rustc-cfg=mysql_8_4_x");
    } else if version.starts_with("10.") || version.starts_with("11.") || version.starts_with("3.")
    {
        println!("cargo:rustc-cfg=mariadb_10_x");
    } else {
        #[cfg(not(feature = "buildtime_bindgen"))]
        panic!(
            "mysqlclient-sys does not provide bundled bindings for libmysqlclient {version}. \
             Consider using the `buildtime_bindgen` feature or \
             contribute bindings to the crate"
        )
    }
}

#[cfg(target_env = "msvc")]
fn try_vcpkg() -> bool {
    if vcpkg::find_package("libmariadb").is_ok() {
        return true;
    } else if vcpkg::find_package("libmysql").is_ok() {
        return true;
    }
    false
}

#[cfg(not(target_env = "msvc"))]
fn try_vcpkg() -> bool {
    false
}

#[cfg(not(feature = "buildtime_bindgen"))]
fn autogen_bindings() {}

#[cfg(feature = "buildtime_bindgen")]
fn autogen_bindings() {
    // if you update the options here you also need to
    // update the bindgen command in `DEVELOPMENT.md`
    // and regenerate the bundled bindings with the new options
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("bindings/wrapper.h")
        .allowlist_function("mysql.*")
        .allowlist_type("MYSQL.*")
        .allowlist_type("mysql.*")
        .allowlist_var("MYSQL.*")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = std::path::PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
