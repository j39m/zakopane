# zakopane

... is a script that checksums your files.

In a sentence, `zakopane` provides recursive sha256sums for your home
directory. (The resourceful or scrappy wielder may find other uses for
it - but that's my primary use case.)

> **NOTE**: This project is free software, a personal project by j39m.
> However, Google LLC owns the copyright on commits
> `677a32b167502f6d5092add7f95178e81acf4d5d` and newer. This does not
> impact your ability to use and to hack at this free software; I
> provide this notice only for attribution purposes.

## Usage

### `simple-zakopane.sh`

... is a shell script that uses `find` and `sha256sum` to checksum files
of your choosing. You feed the directory-to-sum as its sole argument.
I typically use it like so:

```sh
# simple-zakopane.sh writes the checksums into its cwd. Before invoking
# simple-zakopane.sh, you should "cd" into the desired output dir.
[j39m@SERN ~/Downloads]
$ cd ~/.config/zakopane/

# I want to checksum my home directory; I invoke simple-zakopane.sh
# like so:
[j39m@SERN ~/.config/zakopane]
$ simple-zakopane.sh ~/
# much time passes...
simple-zakopane.sh: Wrote 2019-09-27-140303.sums.
[j39m@SERN ~/.config/zakopane]
$ ls
2019-09-27-140303.sums 
```

### `zakocmp`

... is a program that reports notable differences between two `zakopane`
snapshots. The invoker provides a configuration file to instruct
`zakocmp` on what constitutes a noteworthy discrepancy.

You can find its README.md [here](zakocmp/README.md).

## Notes

*   For performance and correctness, perhaps manually implementing the
    checksum snapshot is preferable. But for my purposes,
    `simple-zakopane.sh` is perfectly adequate.
*   Note that `simple-zakopane.sh` does not traverse directories whose
    names begin with a dot (`.`). This is an intentional, historical
    choice made to avoid the visual churn of tracking transient files in
    `~/.config`, `~/.local`, etc. etc.
