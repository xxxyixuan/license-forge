`lic` is a command line tool to create LICENSE files.

It provides good defaults for author, year and project name (customizable if
needed), and ships 17 license templates compiled into the binary — so it needs
no network access and no cache. The license texts come from the
[GitHub Licenses API](https://docs.github.com/en/rest/licenses) and the
[SPDX License List](https://spdx.org/licenses/).

```
agpl-3.0      (GNU Affero General Public License v3.0)
apache-2.0    (Apache License 2.0)
artistic-2.0  (Artistic License 2.0)
bsd-2-clause  (BSD 2-Clause "Simplified" License)
bsd-3-clause  (BSD 3-Clause "New" or "Revised" License)
bsd-4-clause  (BSD 4-Clause "Original" or "Old" License)
bsl-1.0       (Boost Software License 1.0)
cc0-1.0       (Creative Commons Zero v1.0 Universal)
gpl-3.0       (GNU General Public License v3.0)
isc           (ISC License)
lgpl-3.0      (GNU Lesser General Public License v3.0)
mit           (MIT License)
mpl-2.0       (Mozilla Public License 2.0)
mulanpsl-2.0  (木兰宽松许可证 第2版)
unlicense     (The Unlicense)
wtfpl         (Do What The Fuck You Want To Public License)
zlib          (zlib License)
```

SPDX identifiers and common aliases work too: `lic MIT`, `lic apache`, `lic gpl`,
`lic mulan`.

## Install

### Building from Source

With Rust 1.75 or higher:

```sh
$ cargo install license-forge
```

Or from a checkout:

```sh
$ cargo install --path .
```

The crate is named `license-forge`; the binary is `lic`.

## Usage

#### Print license

To print a license to stdout, run `lic` followed by the license name:

```sh
$ lic mit
```

#### Save to file

Use the `-o` flag to save the license to a file, or use your shell's redirection
operator:

```sh
$ lic mit -o LICENSE
$ lic mit > LICENSE
```

#### Customize author, year and project name

```sh
$ lic --author "Alice L" --year 2025 isc
$ lic -n "Alice L" -y 2025 -p "myapp" bsd-4-clause
```

The current year is used if `--year` is omitted.

The current folder is used as the project name if `--project` is omitted.

To determine the author, the following are used in this order:

```
- command line flags: --author, -n
- environment variables: LICENSE_FULL_NAME, LICENSE_AUTHOR
- gitconfig and hgconfig
- the system user name
- empty string
```

If you have your name set in `$HOME/.gitconfig`, you can almost always omit the
`--author` flag.

#### List licenses

```sh
$ lic list
$ lic ls
$ lic --list
```

#### Show license details

```sh
$ lic info mit
$ lic --info mit
```

#### Output formats

```sh
$ lic --format json mit
$ lic --format markdown mit
$ lic --format text mit
```

`text` is the default. JSON output carries the SPDX identifier, the rendered
text, the resolved author/year/project, and the license's permissions,
conditions and limitations.

#### Non-interactive use

```sh
$ lic --author "CI Bot" --year 2025 --project "myapp" --yes -o ./LICENSE mit
```

#### Options

```
-n, --author <AUTHOR>     copyright holder
-y, --year <YEAR>         copyright year
-p, --project <PROJECT>   project name
-o, --output <FILE>       write to FILE instead of stdout
    --format <FORMAT>     text (default), markdown, json
    --yes                 never prompt
    --force               overwrite an existing output file
    --offline             use only built-in templates and cache
    --auth <AUTH>         GitHub API credentials (token or username:token); accepted, not used yet
    --cache-ttl <DAYS>    template cache TTL in days; accepted, not used yet
    --list                same as `lic list`
    --info <LICENSE>      same as `lic info <LICENSE>`
    --update              same as `lic update`
-h, --help                print help
-V, --version             print version
```

If the output file already exists, `lic` asks before overwriting when stdin is a
terminal. In non-interactive use it refuses unless `--force` is given.

Exit codes are `0` for success, `1` when the user aborts, and `2` for an error.
An unknown license name gets a fuzzy-match suggestion:

```sh
$ lic apachee
error: unknown license `apachee`
hint: did you mean: apache-2.0? (run `lic list` for every built-in license)
```

## Contributing

Pull requests for new features, bug fixes, and suggestions are welcome! Please
create an issue for discussion before working on a substantial change.

## License

[MIT](LICENSE)
