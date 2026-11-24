#!/usr/bin/env bash

DEBIAN_FRONTEND=noninteractive
export DEBIAN_FRONTEND

set +ex

declare -A versions
versions['26.10.0']="https://dev.mysql.com/get/Downloads/MySQL-26.10/libmysqlclient-dev_26.10.0-1debian12_amd64.deb"
versions['26.7.0']="https://dev.mysql.com/get/Downloads/MySQL-26.7/libmysqlclient-dev_26.7.0-1debian12_amd64.deb"
versions['9.7.4']="https://dev.mysql.com/get/Downloads/MySQL-9.7/libmysqlclient-dev_9.7.4-1debian12_amd64.deb"
versions['9.6.0']="https://dev.mysql.com/get/Downloads/MySQL-9.6/libmysqlclient-dev_9.6.0-1debian12_amd64.deb"
versions['9.5.0']="https://dev.mysql.com/get/Downloads/MySQL-9.5/libmysqlclient-dev_9.5.0-1debian12_amd64.deb"
versions['9.4.0']="https://dev.mysql.com/get/Downloads/MySQL-9.4/libmysqlclient-dev_9.4.0-1debian12_amd64.deb"
versions['9.3.0']="https://dev.mysql.com/get/Downloads/MySQL-9.3/libmysqlclient-dev_9.3.0-1debian12_amd64.deb"
versions['9.2.0']="https://dev.mysql.com/get/Downloads/MySQL-9.2/libmysqlclient-dev_9.2.0-1debian12_amd64.deb"
versions['9.1.0']="https://downloads.mysql.com/archives/get/p/23/file/libmysqlclient-dev_9.1.0-1debian12_amd64.deb"
versions['9.0.1']="https://downloads.mysql.com/archives/get/p/23/file/libmysqlclient-dev_9.0.1-1debian12_amd64.deb"
versions['8.4.3']="https://downloads.mysql.com/archives/get/p/23/file/libmysqlclient-dev_8.4.3-1debian12_amd64.deb"
versions['8.3.0']="https://downloads.mysql.com/archives/get/p/23/file/libmysqlclient-dev_8.3.0-1debian12_amd64.deb"
versions['8.2.0']="https://downloads.mysql.com/archives/get/p/23/file/libmysqlclient-dev_8.2.0-1debian12_amd64.deb"
versions['8.1.0']="https://downloads.mysql.com/archives/get/p/23/file/libmysqlclient-dev_8.1.0-1debian11_amd64.deb"
versions['8.0.39']="https://downloads.mysql.com/archives/get/p/23/file/libmysqlclient-dev_8.0.39-1debian12_amd64.deb"
versions['5.7.42']="https://downloads.mysql.com/archives/get/p/23/file/libmysqlclient-dev_5.7.42-1debian10_amd64.deb"

# Newer MariaDB Connector/C releases are only published as source tarballs,
# so the headers are generated via cmake (see below)
declare -A mariadb_versions
mariadb_versions['3.1.28']="https://downloads.mariadb.org/rest-api/connector-c/3.1.28/mariadb-connector-c-3.1.28-src.tar.gz"
mariadb_versions['3.3.21']="https://downloads.mariadb.org/rest-api/connector-c/3.3.21/mariadb-connector-c-3.3.21-src.tar.gz"
mariadb_versions['3.4.11']="https://downloads.mariadb.org/rest-api/connector-c/3.4.11/mariadb-connector-c-3.4.11-src.tar.gz"

apt update
apt install -y binutils xz-utils curl libclang-dev gcc mingw-w64 gcc-i686-linux-gnu clang gcc-arm-linux-gnueabi cmake libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal -y -c rustfmt
. "/root/.cargo/env"
cargo install bindgen-cli@0.73.2

function bindgen_common() {
    bindgen --allowlist-function "mysql.*" \
        --allowlist-function "mariadb.*" \
        --allowlist-type "MYSQL.*" \
        --allowlist-type "MARIADB.*" \
        --allowlist-type "mysql.*" \
        --allowlist-type "mariadb.*" \
        --allowlist-var "MYSQL.*" \
        --allowlist-var "MARIADB.*" \
        --default-enum-style rust_non_exhaustive \
        /bindings/wrapper.h \
        -- -I./usr/include/mysql $@
}

for version in "${!versions[@]}"; do
    rm -rf /scratch
    mkdir -p /scratch
    cd /scratch
    curl -L "${versions[$version]}" -o libmysqlclient.deb
    ar x libmysqlclient.deb
    tar -xf data.tar.xz .

    bindgen_common >/bindings/bindings_${version//./_}_x86_64_linux.rs
    bindgen_common -target i686-unknown-linux-gnu >/bindings/bindings_${version//./_}_i686_linux.rs
    bindgen_common -target arm-unknown-linux-gnueabi >/bindings/bindings_${version//./_}_arm_linux.rs
    bindgen_common -target i686-pc-windows-gnu -I /usr/lib/gcc/i686-w64-mingw32/14-win32/include/ -D_GCC_MAX_ALIGN_T >/bindings/bindings_${version//./_}_i686_windows.rs
    bindgen_common -target x86_64-pc-windows-gnu -I /usr/lib/gcc/x86_64-w64-mingw32/14-win32/include -D_X86INTRIN_H_INCLUDED -D_EMMINTRIN_H_INCLUDED -D_GCC_MAX_ALIGN_T >/bindings/bindings_${version//./_}_x86_64_windows.rs
done

for version in "${!mariadb_versions[@]}"; do
    rm -rf /scratch
    mkdir -p /scratch
    cd /scratch
    curl -L "${mariadb_versions[$version]}" -o out.tar.gz
    tar -xf out.tar.gz
    # configure the sources to generate `mariadb_version.h` and install only the headers
    # (same layout as in the binary releases)
    cmake -S mariadb-connector-c-*-src -B build -DCMAKE_INSTALL_PREFIX=/usr/local -DWITH_UNIT_TESTS=OFF
    DESTDIR=/scratch/install cmake -DCMAKE_INSTALL_COMPONENT=Development -P build/include/cmake_install.cmake
    mkdir -p usr/include/mysql/
    cp -r install/usr/local/include/mariadb/* usr/include/mysql/
    bindgen_common >/bindings/bindings_mariadb_${version//./_}_x86_64_linux.rs
    bindgen_common -target i686-unknown-linux-gnu >/bindings/bindings_mariadb_${version//./_}_i686_linux.rs
    bindgen_common -target arm-unknown-linux-gnueabi >/bindings/bindings_mariadb_${version//./_}_arm_linux.rs
    bindgen_common -target i686-pc-windows-gnu -I /usr/lib/gcc/i686-w64-mingw32/14-win32/include/ -D_GCC_MAX_ALIGN_T >/bindings/bindings_mariadb_${version//./_}_i686_windows.rs
    bindgen_common -target x86_64-pc-windows-gnu -I /usr/lib/gcc/x86_64-w64-mingw32/14-win32/include -D_X86INTRIN_H_INCLUDED -D_EMMINTRIN_H_INCLUDED -D_GCC_MAX_ALIGN_T >/bindings/bindings_mariadb_${version//./_}_x86_64_windows.rs
done
