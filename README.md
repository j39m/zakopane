# zakopane

... is a script that checksums your files.

zakopane began in Python to practice serialization and working with the
standard cryptographic digests. This Shadokian idea has since been replaced
with a much easier implementation in a shell script. At this time, I don't
plan to return to the Python version.

## Usage

### `simple-zakopane.sh`

... is a shell script that uses `find` and `sha256sum` to checksum files
of your choosing. You feed the directory-to-sum as its sole argument.
I typically use it like so:

```sh
cd ~/.zakopane          # store my checksum snapshot here
simple-zakopane.sh ~/   # checksum my home directory
```

I have a medium-sized home directory and a slow hard disk, so the script
will run for a pretty long time. I usually monitor its progress with
`pstree`. When `simple-zakopane.sh` finishes, it will have written out
an output file containing all the checksums of the directory-to-sum
into a file named for the time of invocation (I refer to this as the
checksum snapshot). As an example:

```sh
[j39m@SERN ~]
$ ls ~/.zakopane/
2019-01-30-074129.sums
[j39m@SERN ~]
$ file !$/*
file ~/.zakopane//*
/home/kalvin/.zakopane//2019-01-30-074129.sums: UTF-8 Unicode text
```

### `cmp-zakopane.py`

... is a Python script that compares two different checksum snapshots.

## Notes

*   Nothing of consequence yet!
*   For performance and correctness, perhaps manually implementing the
    checksum snapshot is preferable. But for my purposes,
    `simple-zakopane.sh` is perfectly adequate.
