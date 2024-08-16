use std::env;
use std::path::PathBuf;
use std::process::Command;

const PKG_CONFIG_MYSQL_LIB: &str = "mysqlclient";
const PKG_CONFIG_MARIADB_LIB: &str = "libmariadb";
#[cfg(target_env = "msvc")]
const VCPKG_MYSQL_LIB: &str = "libmysql";
#[cfg(target_env = "msvc")]
const VCPKG_MARIADB_LIB: &str = "libmariadb";

fn main() {
    if cfg!(feature = "bundled") {
        parse_version("8.4.0");
        return;
    }
    let target = std::env::var("TARGET")
        .expect("Set by cargo")
        .to_ascii_uppercase()
        .replace("-", "_");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_VERSION");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_INCLUDE_DIR");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_INCLUDE_DIR_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIB");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIB_DIR");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIB_DIR_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIBNAME");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIBNAME_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_STATIC");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_VERSION_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_LIB_{target}");
    println!("cargo::rerun-if-env-changed=MYSQLCLIENT_STATIC_{target}");
    if cfg!(feature = "buildtime_bindgen") {
        autogen_bindings(&target);
    }
    let libname = env::var("MYSQLCLIENT_LIBNAME")
        .or_else(|_| env::var(format!("MYSQLCLIENT_LIBNAME_{target}")))
        .unwrap_or("mysqlclient".to_string());
    let link_specifier = if env::var("MYSQLCLIENT_STATIC")
        .or(env::var(format!("MYSQLCLIENT_STATIC_{target}")))
        .is_ok()
    {
        "static="
    } else {
        ""
    };
    if let Ok(lib) = pkg_config::probe_library(PKG_CONFIG_MYSQL_LIB)
        .or_else(|_| pkg_config::probe_library(PKG_CONFIG_MARIADB_LIB))
    {
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
        println!("cargo:rustc-link-lib={link_specifier}{libname}");
        if let Ok(version) =
            env::var("MYSQLCLIENT_VERSION").or(env::var(format!("MYSQLCLIENT_VERSION_{target}")))
        {
            parse_version(&version);
            return;
        }
    } else if let Some(output) = mysql_config_variable("--libs") {
        let parts = output.split_ascii_whitespace();
        for part in parts {
            if let Some(lib) = part.strip_prefix("-l") {
                println!("cargo:rustc-link-lib={link_specifier}{lib}");
            } else if let Some(path) = part.strip_prefix("-L") {
                println!("cargo:rustc-link-search=native={path}");
            } else if let Some(path) = part.strip_prefix("-R") {
                println!("cargo:rustc-link-arg=-Wl,-R{path}");
            } else {
                panic!("Unexpected output from mysql_config: `{output}`");
            }
        }

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

#[derive(Clone, Copy, Debug)]
enum MysqlVersion {
    Mysql5,
    Mysql80,
    Mysql83,
    Mysql84,
    MariaDb10,
}

impl MysqlVersion {
    const ALL: &'static [Self] = &[
        Self::Mysql5,
        Self::Mysql80,
        Self::Mysql83,
        Self::Mysql84,
        Self::MariaDb10,
    ];

    fn as_cfg(&self) -> &'static str {
        match self {
            MysqlVersion::Mysql5 => "mysql_5_7_x",
            MysqlVersion::Mysql80 => "mysql_8_0_x",
            MysqlVersion::Mysql83 => "mysql_8_3_x",
            MysqlVersion::Mysql84 => "mysql_8_4_x",
            MysqlVersion::MariaDb10 => "mariadb_10_x",
        }
    }

    fn parse_version(version: &str) -> Option<Self> {
        // ubuntu/debian packages use the following package versions:
        // libmysqlclient20 -> 5.7.x
        // libmysqlclient21 -> 8.0.x
        // libmysqlclient23 -> 8.3.0
        // libmysqlclient24 -> 8.4.0
        // libmariadb-dev 3.3.8 -> mariadb 10.x
        // Linux version becomes the full SONAME like 21.3.2 but MacOS is just the
        // major.
        if version.starts_with("5.7") || version.starts_with("20.") || version == "20" {
            Some(Self::Mysql5)
        } else if version.starts_with("8.0") || version.starts_with("21.") || version == "21" {
            Some(Self::Mysql80)
        } else if version.starts_with("8.3") || version.starts_with("23.") || version == "23" {
            Some(Self::Mysql83)
        } else if version.starts_with("8.4") || version.starts_with("24.") || version == "24" {
            Some(Self::Mysql84)
        } else if version.starts_with("10.")
            || version.starts_with("11.")
            || version.starts_with("3.")
            || version == "3"
        {
            Some(Self::MariaDb10)
        } else {
            None
        }
    }
}

fn parse_version(version_str: &str) {
    use MysqlVersion::*;

    for v in MysqlVersion::ALL {
        println!("cargo::rustc-check-cfg=cfg({})", v.as_cfg());
    }
    let version = MysqlVersion::parse_version(version_str);

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("Set by cargo");
    let is_windows = std::env::var("CARGO_CFG_WINDOWS").is_ok();
    let ptr_size = std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH").expect("Set by cargo");
    let out_dir = std::env::var("OUT_DIR").expect("Set by cargo");
    let mut bindings_target = PathBuf::from(out_dir);
    bindings_target.push("bindings.rs");

    if let Some(version) = version {
        println!("cargo:rustc-cfg={}", version.as_cfg());
    }

    let bindings_path = match (version, target_arch.as_str(), ptr_size.as_str(), is_windows) {
        // if we use bindgen to generate bindings
        // we don't want to copy anything
        _ if cfg!(feature = "buildtime_bindgen") => {
            return;
        }
        (Some(Mysql5), "x86_64" | "aarch64", "64", false) => "bindings_5_7_42_x86_64_linux.rs",
        (Some(Mysql80), "x86_64" | "aarch64", "64", false) => "bindings_8_0_36_x86_64_linux.rs",
        (Some(Mysql80), "x86" | "arm", "32", false) => "bindings_8_0_37_i686_linux.rs",
        (Some(Mysql80), "x86_64", "64", true) => "bindings_8_0_36_x86_64_windows.rs",
        (Some(Mysql80), "x86", "32", true) => "bindings_8_0_36_i686_windows.rs",
        (Some(Mysql83), "x86_64" | "aarch64", "64", false) => "bindings_8_3_0_x86_64_linux.rs",
        (Some(Mysql83), "x86_64", "64", true) => "bindings_8_3_0_x86_64_windows.rs",
        (Some(Mysql83), "x86", "32", true) => "bindings_8_3_0_i686_windows.rs",
        (Some(Mysql84), "x86_64" | "aarch64", "64", false) => "bindings_8_4_0_x86_64_linux.rs",
        (Some(Mysql84), "x86" | "arm", "32", false) => "bindings_8_4_0_i686_linux.rs",
        (Some(Mysql84), "x86_64", "64", true) => "bindings_8_4_0_x86_64_windows.rs",
        (Some(Mysql84), "x86", "32", true) => "bindings_8_4_0_i686_windows.rs",
        (Some(MariaDb10), "x86_64" | "aarch64", "64", false) => {
            "bindings_mariadb_10_11_x86_64_linux.rs"
        }
        (Some(MariaDb10), "x86", "32", false) => "bindings_mariadb_10_11_i686_linux.rs",
        (Some(MariaDb10), "arm", "32", false) => "bindings_mariadb_10_11_armv6_linux.rs",
        (Some(MariaDb10), "x86_64", "64", true) => "bindings_mariadb_10_11_x86_64_windows.rs",
        (Some(MariaDb10), "x86", "32", true) => "bindings_mariadb_10_11_i686_windows.rs",
        _ => {
            panic!(
                "mysqlclient-sys does not provide bundled bindings for libmysqlclient `{version_str}` \
                 for the target `{}`.
                 Consider using the `buildtime_bindgen` feature or \
                 contribute bindings to the crate\n\
                 Debug information: (version: {version:?}, target_arch: {target_arch}, ptr_size: {ptr_size})",
                std::env::var("TARGET").expect("Set by cargo")
            )
        }
    };

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Set by cargo");
    let mut bindings = PathBuf::from(root);
    bindings.push("bindings");
    bindings.push(bindings_path);
    std::fs::copy(bindings, bindings_target).unwrap();
}

#[cfg(target_env = "msvc")]
fn try_vcpkg() -> bool {
    if vcpkg::find_package(VCPKG_MYSQL_LIB).is_ok() {
        return true;
    } else if vcpkg::find_package(VCPKG_MARIADB_LIB).is_ok() {
        return true;
    }
    false
}

#[cfg(not(target_env = "msvc"))]
fn try_vcpkg() -> bool {
    false
}

#[cfg(not(feature = "buildtime_bindgen"))]
fn autogen_bindings(_target: &str) {}

#[cfg(feature = "buildtime_bindgen")]
fn autogen_bindings(target: &str) {
    // if you update the options here you also need to
    // update the bindgen command in `DEVELOPMENT.md`
    // and regenerate the bundled bindings with the new options
    let mut builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("bindings/wrapper.h")
        .allowlist_function("mysql.*")
        .allowlist_function("mariadb.*")
        .allowlist_type("MYSQL.*")
        .allowlist_type("MARIADB.*")
        .allowlist_type("mysql.*")
        .allowlist_type("mariadb.*")
        .allowlist_var("MYSQL.*")
        .allowlist_var("MARIADB.*")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if let Ok(lib) = pkg_config::probe_library(PKG_CONFIG_MYSQL_LIB)
        .or_else(|_| pkg_config::probe_library(PKG_CONFIG_MARIADB_LIB))
    {
        for include in lib.include_paths {
            builder = builder.clang_arg(format!("-I{}", include.display()));
        }
    } else if let Ok(path) = env::var("MYSQLCLIENT_INCLUDE_DIR")
        .or_else(|_| env::var(format!("MYSQLCLIENT_INCLUDE_DIR_{target}")))
    {
        builder = builder.clang_arg(format!("-I{path}"));
    } else if let Some(include) = mysql_config_variable("--include") {
        builder = builder.clang_arg(include);
    } else {
        #[cfg(target_env = "msvc")]
        if let Ok(lib) =
            vcpkg::find_package(VCPKG_MYSQL_LIB).or_else(|_| vcpkg::find_package(VCPKG_MARIADB_LIB))
        {
            for include in lib.include_paths {
                builder = builder.clang_arg(format!("-I{}\\mysql", include.display()));
            }
        }
    }

    let bindings = builder
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
