#!/bin/bash

set -eu;

CHECKABLE="$(realpath "$1")";
CHECKSUMMER="/usr/bin/sha256sum"

EXECNAME=$(basename "$0");

NOW="$(date -u "+%Y-%m-%d-%H%M%S")";
FILE_FIN="$NOW".sums

whine() {
    printf "%s: %s\n" "$EXECNAME" "$1";
}

findAndSum() {
    pushd "$CHECKABLE" 1>/dev/null;
    find . -path "*/.*" -prune -o -type f -exec "$CHECKSUMMER" {} "+";
    popd 1>/dev/null;
}

# The main function.
main() {
    [ -f "$FILE_FIN" ] && exit 1;

    # Write the file header.
    whine "$NOW" > "$FILE_FIN";
    whine "$CHECKABLE" >> "$FILE_FIN";
    printf "%s\n" "" >> "$FILE_FIN";

    findAndSum | sort -k2 >> "$FILE_FIN";

    whine "Wrote ""$FILE_FIN"".";
    return 0;
}


main;
exit $?;
