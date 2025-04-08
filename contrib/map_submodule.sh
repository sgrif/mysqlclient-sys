#!/usr/bin/env bash

SERVER_REPO_DIR="/tmp/repo/mariadb-server" # Path to the MariaDB Server repo
CONN_C_REPO_DIR="/tmp/repo/mariadb-connector-c" # Path to the Connector/C repo

# Check if both directories exist
if [ ! -d "${SERVER_REPO_DIR}" ] || [ ! -d "${CONN_C_REPO_DIR}" ]; then
    echo "Error: Repository directories not found. Make sure you've cloned them."
    echo "Expected: '${SERVER_REPO_DIR}' and '${CONN_C_REPO_DIR}'"
    exit 1
fi

# Change into the server repository directory
pushd "${SERVER_REPO_DIR}" || exit 1

# Get all `mariadb-` tags for major versions 10 and 11
# By using `-oE` and matching specifically on mariadb-X.Y.Z, we only get specific release tags.
# So anything like `mariadb-10.10.11-testing` will only return `mariadb-10.10.11`
# This prevents incompatible or wrong matches when using this specific tag.
git tag | grep -oE '^mariadb-(10|11)\.[0-9]{1,2}\.[0-9]{1,2}' | sort -uV | while read -r server_tag; do
    # Use git ls-tree to find the submodule's commit hash for this specific server tag
    submodule_entry=$(git ls-tree "${server_tag}" libmariadb 2>/dev/null)

    if [ -z "${submodule_entry}" ]; then
        # Submodule likely didn't exist at this tag or path was different
        continue
    fi

    # Get the commit hash (3rd field)
    submodule_commit=$(echo "${submodule_entry}" | gawk '{print $3}')

    if [ -z "${submodule_commit}" ]; then
        # If for some reason we can't find the submodule commit, continue to the next one
        continue
    fi

    # Go to the Connector/C repo to find the tag for that commit
    pushd "${CONN_C_REPO_DIR}" > /dev/null || exit 1

    # Use git describe to find the nearest tag annotated or not
    # We grep major v3 and minor 1-9, which skips 0 which is EOL
    # While v3.2 is also EOL, it seems to be 100% compatible with v3.3
    # Because of this, replace v3.2 with v3.3 to make grouping more efficient
    lib_tag=$(git describe --tags --always "${submodule_commit}" 2>/dev/null | grep -oE '^v3\.[1-9]' | sed -e 's#v3\.2#v3.3#')

    # Return to server repo dir before skipping a Connector/C match when it didn't start with v3.
    popd > /dev/null || exit 1

    # We only want good matches between MariaDB Server and Connector/C
    # Anything else is probably a rc or beta version and we do not want those
    if [ -z "${lib_tag}" ]; then
        continue
    fi

    # Some special filtering for server v11 and connector/c v3.3
    # These were only linked during development releases.
    if [[ "${lib_tag}" == v3.3* && "${server_tag}" == mariadb-11* ]]; then
        continue
    fi

    # Print the mapping
    printf "%s|%s\n" "${server_tag/mariadb-/}" "${lib_tag/v/}"
done

# Return to the current work directory
popd > /dev/null || exit 1
