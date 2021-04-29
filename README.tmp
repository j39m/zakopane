# zakopane

`zakopane` is a tool that captures filesystem checksums.
The present implementation can be characterized "`sha256sum`, but
recursive."

## Usage

```sh
# Captures checksums of all files under <directory>.
# Not presently implemented.
zakopane checksum <directory>

# Compares zakopane snapshots <before> and <after> using rules defined
# defined in <config>.
zakopane compare --config <config> <before> <after>
```

## "checksum" Subcommand

There are two noteworthy aspects of the behavior of `zakopane checksum`:

1.  `zakopane` does not descend into directories whose names begin with
    a dot.
1.  `zakopane` does not attempt to stay within the same filesystem and
    will happily cross over bind-mounted filesystem boundaries.

## "compare" Subcommand

`zakopane compare` accepts a config file in the form of a YAML document
comprising

*   a default policy and
*   more specific policies.

Both are optional; in fact, empty YAML documents and YAML dictionaries
with irrelevant keys will be treated as no-op (but valid) configs.

### Policy Values

1.  `ignore` tells `zakopane` to do nothing with matching files. It's as
    though they don't exist.
1.  `noadd` tells `zakopane` to report added files.
1.  `nomodify` tells `zakopane` to report modified files.
1.  `nodelete` tells `zakopane` to report deleted files.
1.  `immutable` is shorthand that means the same thing as
    `noadd,nomodify,nodelete` all together.

Policies are joined together (without spaces) by a comma as in the
definition of the `immutable` policy. Order and repetition do not
matter.

### Default Policy

`zakopane compare` determines the default policy

1.  by looking for it on the command-line (`--default-policy` or `-d`),
1.  by looking for it in the config file (if given), and
1.  finally by falling back to a hardcoded default of `immutable`.

## Appendix: Comparison Configurations

```yaml
# Anything not covered by a specific policy should be ignored.
default-policy: ignore

# We only care about paths spelling out prequel memes, it seems.
policies:
    ./Documents/hello/there: nomodify,nodelete
    ./Documents/general/kenobi: noadd,nodelete
```

In a `zakopane compare` config, the longest path match wins. Take the
following policies excerpt:

```yaml
policies:
    ./Documents/: nomodify
    ./Documents/you/are/a/bold/one/: ignore
```

Then a file named `./Documents/you/are/shorter-than-i-expected.txt` will
be subject to the former `nomodify` rule, while a file named
`./Documents/you/are/a/bold/one/poo/doo.txt` will be subject to the
latter `ignore` rule.

There is no concept of policy "strength;" the longest path match always
wins. Suppose the year is CE 2020, and I'm still actively adding family
photos to the directory of the same year. Here's an appropriate
policies excerpt:

```yaml
policies:
    ./family-pictures/: immutable
    ./family-pictures/2020/: nomodify,nodelete
```

The above policies excerpt specifies that new entities may appear under
`./family-pictures/2020`, but existing entities must never change or
disappear. All other entities under `./family-pictures/` must never
change in any way; `zakopane` will visually warn of addition, deletion,
or modification of these.
