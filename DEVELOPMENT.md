# Development related notices


Bindings were generated with the following command:

```sh
bindgen --allowlist-function "mysql.*" --allowlist-type "MYSQL.*" --allowlist-type "mysql.*" --allowlist-var "MYSQL.*" --default-enum-style rust_non_exhaustive bindings/wrapper.h -- -I/usr/include/mysql
```

If you update the above command line you also need to update the arguments for the buildtime_bindgen feature in `build.rs`
