fn main() {
    let ptr_width = std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH").expect("This is set by cargo");
    if ptr_width != "64" {
        panic!("libmysqlclient only supports 64 bit platforms, but the target platform uses {ptr_width} bit pointers");
    }

    // `DEP_OPENSSL_ROOT` is only set if openssl-src is used
    let openssl_dir = std::env::var("DEP_OPENSSL_ROOT");

    let mut config = cmake::Config::new("source");

    config
        .define("WITHOUT_SERVER", "ON")
        .define("WITH_EDITLINE", "bundled")
        .build_target("mysqlclient");

    // If openssl-src appears in the dependency tree just use this,
    // otherwise use the default `system`
    if openssl_dir.is_ok() {
        config.define("WITH_SSL", openssl_dir.unwrap());
    }

    if cfg!(feature = "with-asan") {
        config.define("WITH_ASAN", "ON");
    }

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV");
    if target_env.as_deref() == Ok("msvc") {
        // rust links the release MVSC runtime
        // also for debug builds. If we let
        // cmake choose debug/release builds
        // based on the underlying cargo build
        // version that results in linker errors
        config.profile("Release");
    } else if target_env.as_deref() == Ok("musl") {
        // when (cross) compiling for musl targets
        // you need an extra flag to tell the
        // compiler it is building a musl target
        config.define("LINUX_ALPINE", "1");
    }

    let mut dst = config.build();
    dst.push("build");
    dst.push("archive_output_directory");

    // on windows the library is in a different folder
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        dst.push("Release");
    }

    println!("cargo::rustc-link-search=native={}", dst.display());
    for entry in std::fs::read_dir(&dst).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let extension = path.extension().and_then(|e| e.to_str());
        if extension == Some("a") || extension == Some("lib") {
            let lib_name = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .trim_start_matches("lib");
            println!("cargo::rustc-link-lib=static={lib_name}");
        }
    }
}
