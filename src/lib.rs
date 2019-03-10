#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

#[cfg(not(windows))]
include!("bindings_macos.rs");

#[cfg(all(windows, target_arch = "x86_64"))]
include!("bindings_windows_x86_64.rs");

#[cfg(all(windows, target_arch = "x86"))]
include!("bindings_windows_x86_32.rs");