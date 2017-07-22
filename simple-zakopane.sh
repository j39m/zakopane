#!/bin/sh
set -u  # use strict; use warnings
#set -x  # make things visible

# Travels through selected directories and outputs checksums of the files
# within.
# This is way easier than writing all the Python skullduggery. It is bad
# and wrong in many ways, but it will do.

CHECKABLE="$(realpath "$1")";
CHECKSUMMER="/usr/bin/sha256sum"

EXECNAME=$(basename "$0");

SUFFIX_TMP="tmp";
SUFFIX_FIN="sums";
NOW="$(date -u "+%Y-%m-%d-%H%M%S")";
FILE_TMP="$NOW"."$SUFFIX_TMP";
FILE_FIN="$NOW"."$SUFFIX_FIN";


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
    find . -path "*/.*" -prune -o -type f -exec "$CHECKSUMMER" {} "+";
    popd 1>/dev/null;
}


# The main function.
main() {
    fileExistsAbort "$FILE_TMP";
    fileExistsAbort "$FILE_FIN";

    # Write the file header.
    whine "$NOW" > "$FILE_FIN";
    whine "$CHECKABLE" >> "$FILE_FIN";
    printf "\n" >> "$FILE_FIN";

    # Write the initial pass.
    findAndSum > "$FILE_TMP";

    # Finalize the checksum results.
    sort -k2 "$FILE_TMP" >> "$FILE_FIN";
    rm -f "$FILE_TMP";

    whine "Wrote ""$FILE_FIN"".";
    return 0;
}


# Here's the main entry point of this script.
main;
exit $?;
