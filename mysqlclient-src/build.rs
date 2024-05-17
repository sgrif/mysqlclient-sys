fn main() {
    let openssl_dir = std::env::var("DEP_OPENSSL_ROOT").unwrap();

    let mut config = cmake::Config::new("source");

    config
        .define("WITHOUT_SERVER", "ON")
        .define("WITH_SSL", openssl_dir)
        .build_target("mysqlclient");

    if cfg!(feature = "with-asan") {
        config.define("WITH_ASAN", "ON");
    }

    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        // rust links the release MVSC runtime
        // also for debug builds. If we let
        // cmake choose debug/release builds
        // based on the underlying cargo build
        // version that results in linker errors
        config.profile("Release");
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
