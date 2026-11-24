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
        // The optional client authentication plugins (webauthn, kerberos, ldap, oci, …)
        // are built as separate shared libraries and are not part of
        // libmysqlclient itself. Disable them to avoid requiring additional
        // system libraries like libfido2/libudev/libcurl.
        .define("WITH_AUTHENTICATION_WEBAUTHN", "OFF")
        .define("WITH_AUTHENTICATION_CLIENT_PLUGINS", "OFF")
        .define("WITH_FIDO", "none")
        // curl is only used by server components and client plugins, not by
        // libmysqlclient. Since 26.x the configure step requires the system
        // curl development files by default (or builds the bundled one on windows).
        .define("WITH_CURL", "none")
        .build_target("mysqlclient");

    // If openssl-src appears in the dependency tree just use this,
    // otherwise use the default `system`
    if openssl_dir.is_ok() {
        config.define("WITH_SSL", openssl_dir.unwrap());
    }

    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("musl") {
        // MySQL only enables its musl specific code paths (e.g. the
        // non thread safe resolver functions in `libmysql/dns_srv.cc`) if
        // it detects that it is built on Alpine linux by checking for the
        // existence of `/etc/alpine-release` on the build host.
        // That does not work while cross compiling from a glibc based host,
        // so set that flag based on the rust target.
        config.define("LINUX_ALPINE", "1");
        // musl provides the resolver functions as part of libc itself
        // (`libresolv.a` is only an empty compatibility stub).
        // MySQL searches for `libresolv` via `FIND_LIBRARY`, which does not
        // look into the sysroot of cross compiling toolchains and therefore
        // fails to configure. Point it to libc directly instead.
        config.define("RESOLV_LIBRARY", "c");
        // The build executes some helper tools (comp_err, uca9dump, …)
        // compiled for the target. Link them statically so that they
        // can be executed on the (glibc based) build host, which usually has
        // neither the musl dynamic loader nor a musl libstdc++.so installed.
        // This only affects these executables, libmysqlclient itself is
        // built as static library anyway.
        config.define("CMAKE_EXE_LINKER_FLAGS", "-static");
    }

    let host = std::env::var("HOST").expect("Set by cargo");
    let target = std::env::var("TARGET").expect("Set by cargo");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Set by cargo");
    if host != target && target_os == "linux" {
        // The cmake crate puts cmake into cross compiling mode if host != target
        // (this includes `x86_64-unknown-linux-musl` on a `x86_64-unknown-linux-gnu` host).
        // In that mode cmake cannot execute the `CHECK_C_SOURCE_RUNS` checks
        // from `configure.cmake`, so we provide the results here.
        // These features are available on all supported linux libc implementations
        // (glibc and musl), `0` is the exit code of the successful test program.
        config
            .define("HAVE_SETNS_EXITCODE", "0")
            .define("HAVE_CLOCK_GETTIME_EXITCODE", "0")
            .define("HAVE_CLOCK_REALTIME_EXITCODE", "0");

        // The build needs to execute some generated helper tools
        // (comp_err, comp_client_err, uca9dump, …) which are compiled
        // for the target. CMake only runs them in cross compiling mode
        // if an emulator is configured. If host and target share the
        // same architecture (e.g. `x86_64-unknown-linux-musl` on a
        // `x86_64-unknown-linux-gnu` host) the binaries can be executed
        // directly, so we use `env` as no-op "emulator". For other targets
        // an emulator (e.g. `qemu-aarch64`) can be provided via
        // `MYSQLCLIENT_SRC_CROSSCOMPILING_EMULATOR`
        println!("cargo::rerun-if-env-changed=MYSQLCLIENT_SRC_CROSSCOMPILING_EMULATOR");
        let host_arch = host.split('-').next();
        let target_arch = target.split('-').next();
        if let Ok(emulator) = std::env::var("MYSQLCLIENT_SRC_CROSSCOMPILING_EMULATOR") {
            config.define("CMAKE_CROSSCOMPILING_EMULATOR", emulator);
        } else if host_arch == target_arch && host.contains("-linux-") {
            config.define("CMAKE_CROSSCOMPILING_EMULATOR", "env");
        }
    }

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
