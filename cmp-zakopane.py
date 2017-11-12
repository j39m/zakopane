#!/usr/bin/env python3

"""
This simple script accepts 2 files outputted by the zakopane checksummer
and compares them, printing a list of files which have changed.

New files and deleted files are ignored.
"""

import sys


ZF_DB_HEADER_LINES = 3


def process_line(zf_line, dictified):
    """Dictify a single nonheader line from the zakopane db."""
    zf_kv = zf_line.strip()

    (checksum_, fname_) = zf_kv.split(maxsplit=1)
    (checksum, fname) = (checksum_.strip(), fname_.strip())
    dictified[fname] = checksum

def dictify_zf(fname):
    """Given path ``fname,'' dictify as zakopane db."""
    dictified = dict()
    in_header = ZF_DB_HEADER_LINES

    with open(fname, "r") as zf_fp:
        for line in zf_fp:
            if in_header:
                in_header -= 1
                continue
            process_line(line, dictified)

    return dictified

def print_diff(dba, dbb):
    """Given 2 dictified zakopane dbs, print their difference."""
    diff = [key for (key, val) in dba.items()
        if key in dbb and dbb[key] != val]

    diff.sort()
    for fname in diff:
        print(fname)


def main(*args):
    (fname_a, fname_b) = args[:2]
    zf_dba = dictify_zf(fname_a)
    zf_dbb = dictify_zf(fname_b)
    print_diff(zf_dba, zf_dbb)
    return 0


if __name__ == "__main__":
    sys.exit(main(*sys.argv[1:]))
