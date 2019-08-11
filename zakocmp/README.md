# zakocmp

`zakocmp` is a tool that compares zakopane snapshots.

## Usage

```sh
# Compares zakopane snapshots <before> and <after> using rules defined
# defined in <config>.
# All arguments are required.
zakocmp <config> <before> <after>
```

## The config file

A `zakocmp` config file is a YAML document comprising
*   at least a default policy and
*   optionally more specific policies.

### Policy appendix

1.  `ignore` tells `zakocmp` to do nothing with matching files. It's as
    though they don't exist.
1.  `noadd` tells `zakocmp` to report added files.
1.  `nomodify` tells `zakocmp` to report modified files.
1.  `nodelete` tells `zakocmp` to report deleted files.
1.  `immutable` is shorthand that means the same thing as
    `noadd,nomodify,nodelete` all together.

Policies are joined together (without spaces) by a comma as in the
definition of the `immutable` policy. Order and repetition do not
matter.

### Examples

```yaml
# This specification is always necessary - even if you just want to
# ignore everything not covered by a specific policy.
default-policy: ignore

# This specification is optional, but must be well-formed if present.
policies:
    ./Documents/hello/there: nomodify,nodelete
    ./Documents/general/kenobi: noadd,nodelete
```

In a `zakocmp` config, the longest path match wins. Take the following
policies excerpt:

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
change in any way; `zakocmp` will visually warn of addition, deletion,
or modification of these.
