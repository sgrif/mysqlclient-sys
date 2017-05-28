#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

#[cfg(not(windows))]
include!("bindings_macos.rs");

#[cfg(windows)]
include!("bindings_windows.rs");