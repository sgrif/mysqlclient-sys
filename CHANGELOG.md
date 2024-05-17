# Change Log
All user visible changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/), as described
for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)


## [0.3.0] 2024-05-17

### Added

- We added a `mysqlclient-src` crate and a `bundled` feature for `mysqlclient-sys`. This allows to build and link a static version of libmysqlclient during the rust build process. This feature currently supports targeting Windows, Linux and macOS. It requires a c++ compiler toolchain and cmake to build libmysqlclient from source.
- We added a `buildtime_bindgen` feature flag that allows to generate bindings for your locally installed libmysqlclient version. This is helpful for cases where the target architecture is significantly different to what the built-in bindings assume.


### Changed

- We regenerated the bundled bindings for several libmysqlclient versions. You might now need to set the `MYSQLCLIENT_VERSION` environment to select the matching bindings for your libmysqlclient version
