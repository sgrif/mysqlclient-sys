#!/usr/bin/env bash

# You can run this via podman (or docker):
# podman run --rm -it -v "${PWD}:/app" debian:bookworm /app/generate_mapping.sh
#
# If you want to persist the repo's on disk use thes following
# Keep in mind that if the container runs as root, the git files will be owned by root
# podman run --rm -it -v "${PWD}:/app" -v "/path/to/storage/for/repo:/tmp/repo" debian:bookworm /app/generate_mapping.sh

ENV_BASEDIR=$(RL=$(readlink -n "$0"); SP="${RL:-$0}"; dirname "$(cd "$(dirname "${SP}")"; pwd)/$(basename "${SP}")")

# Check if this script is run inside a docker container and if it is apt based
# If so check and verify if we need to install the needed tools
if [[ -f /.dockerenv || -f /run/.containerenv ]] && command -v apt >/dev/null 2>&1; then
    apt update
    apt install -y --no-install-recommends gawk git ca-certificates
fi

# Go to the correct script base path
if [[ "${ENV_BASEDIR}" != "${PWD}" ]]; then
    pushd "${ENV_BASEDIR}" || exit 1
fi

# We check if both repo's are cloned already
# If not, clone them, else reset, pull and fetch new tags
if [ ! -d "/tmp/repo/mariadb-server/.git" ]; then
    git clone https://github.com/MariaDB/server.git /tmp/repo/mariadb-server
else
    # In case the user is different, mark it as a safe directory
    git config --global --add safe.directory /tmp/repo/mariadb-server
    pushd /tmp/repo/mariadb-server || exit 1
    git reset --hard
    git pull
    git fetch --all --tags
    popd || exit 1
fi

if [ ! -d "/tmp/repo/mariadb-connector-c/.git" ]; then
    git clone https://github.com/mariadb-corporation/mariadb-connector-c.git /tmp/repo/mariadb-connector-c
else
    # In case the user is different, mark it as a safe directory
    git config --global --add safe.directory /tmp/repo/mariadb-connector-c
    pushd /tmp/repo/mariadb-connector-c || exit 1
    git reset --hard
    git pull
    git fetch --all --tags
    popd || exit 1
fi

# Generate the compatibility map
./map_submodule.sh | ./group_compatible.sh
