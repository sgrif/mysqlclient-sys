# Development related notices


Bindings were generated with the following command:

```sh
bindgen --allowlist-function "mysql.*" --allowlist-type "MYSQL.*" --allowlist-type "mysql.*" --allowlist-var "MYSQL.*" --default-enum-style rust_non_exhaustive bindings/wrapper.h
```
