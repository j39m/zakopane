#!/bin/sh
set -u  # use strict; use warnings
#set -x  # make things visible

# Travels through selected directories and outputs checksums of the files
# within.
# This is way easier than writing all the Python skullduggery. It is bad
# and wrong in many ways, but it will do.

CHECKABLE="$1";
CHECKSUMMER="/usr/bin/sha256sum"

EXECNAME=$(basename "$0");

SUFFIX_FIN="sums"
NOW="$(date -u "+%Y-%m-%d-%H%M%S")"
FILE_FIN="$NOW"."$SUFFIX_FIN"


# Log a message.
whine() {
    printf "%s: %s\n" "$EXECNAME" "$1";
}

# This function bails us out if a file exists.
fileExistsAbort() {
    [ -f "$1" ] && (whine "file ""$1"" exists!" && exit 1);
    return 0;
}

# Run the traversal / checksumming routine using -exec.
findAndSum() {
    pushd "$CHECKABLE" 1>/dev/null;
    find . -type f -exec "$CHECKSUMMER" {} "+";
    popd 1>/dev/null;
}


# The main function.
main() {
    fileExistsAbort "$FILE_FIN";
    findAndSum > "$FILE_FIN";
    sort -o "$FILE_FIN" -k2 "$FILE_FIN";
    whine "Wrote ""$FILE_FIN"".";
    return 0;
}


# Here's the main entry point of this script.
main;
exit $?;
