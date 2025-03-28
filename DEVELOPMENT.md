# Development related notices


Bindings were generated with the following steps:

* Start a debian container via `podman run -it --rm -v ./bindings/:/bindings:z debian bash`
* Run `/bindings/generate_bindings.sh` inside of the container
