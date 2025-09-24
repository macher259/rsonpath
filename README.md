# rsonpath &ndash; SIMD-powered JSONPath 🚀 <img src="img/rsonquery-logo.svg" width="50em" align="left" />

[![Rust](https://github.com/V0ldek/rsonpath/actions/workflows/rust.yml/badge.svg)](https://github.com/V0ldek/rsonpath/actions/workflows/rust.yml)
[![docs.rs](https://img.shields.io/docsrs/rsonpath-lib?logo=docs.rs)](https://docs.rs/crate/rsonpath-lib/latest)
[![Book](https://img.shields.io/badge/book-available-4DC720?logo=mdbook)](https://v0ldek.github.io/rsonpath/)

[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/7790/badge)](https://www.bestpractices.dev/projects/7790)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/V0ldek/rsonpath/badge)](https://securityscorecards.dev/viewer/?uri=github.com/V0ldek/rsonpath)
[![fuzzing](https://github.com/V0ldek/rsonpath/actions/workflows/clusterfuzzlite-batch.yml/badge.svg)](https://github.com/V0ldek/rsonpath/actions/workflows/clusterfuzzlite-batch.yml)

[![Crates.io](https://img.shields.io/crates/v/rsonpath?logo=docs.rs)](https://crates.io/crates/rsonpath)
[![GitHub Release Date](https://img.shields.io/github/release-date/v0ldek/rsonpath?logo=github)](https://github.com/V0ldek/rsonpath/releases)
[![GitHub last commit](https://img.shields.io/github/last-commit/v0ldek/rsonpath?logo=github)](https://github.com/V0ldek/rsonpath/commits/main)

![MSRV](https://img.shields.io/badge/msrv-v1.67.1-orange?logo=rust "Minimum Supported Rust Version for `rsonpath-lib`")
[![License](https://img.shields.io/crates/l/rsonpath)](https://choosealicense.com/licenses/mit/)

Experimental JSONPath engine for querying massive streamed datasets.

The `rsonpath` crate provides a JSONPath parser and a query execution engine `rq`,
which utilizes SIMD instructions to provide massive throughput improvements over conventional engines.

Benchmarks of `rsonpath` against a reference no-SIMD engine on the
[Pison dataset](https://github.com/AutomataLab/Pison). **NOTE: Scale is logarithmic!**
![Main throughput plot](/img/main-plot.svg)

## Usage

To run a JSONPath query on a file execute:

```console,ignore
rq '$..a.b' ./file.json
```

If the file is omitted, the engine reads standard input. JSON can also be passed inline:

```console
$ rq '$..a.b' --json '{"c":{"a":{"b":42}}}'
42

```

For details, consult `rq --help` or [the rsonbook](https://v0ldek.github.io/rsonpath/).


### Memory streaming

Rsonpath now supports streaming for large files. Please see [the example usage](https://github.com/macher259/rsonpath/blob/thesis/crates/rsonpath-lib/examples/streaming_matches_usage.rs).
You can run this example:
```console
$  cargo run --release --example streaming_matches_usage json_file_path query
```
Example of running an example:
```console
$  cargo run --release --example streaming_matches_usage crates/rsonpath-benchmarks/data/small/az_tenants.json '$[*].name'
```

You can check results with the non-streamed version via:
```console
$  cargo run --release --example matches_usage json_file_path query
```

Additionally, you can check memory usage using [Heaptrack](https://github.com/KDE/heaptrack) for a built binary:
```console
$  cargo build --release --example streaming_matches_usage
$  heaptrack target/release/examples/streaming_matches_usage json_file_path query
```
Please note that for smaller files it could not show any memory savings.
We recommend testing with bigger files. For example a collection of GitHub events via [GitHub archive](https://www.gharchive.org/).
```console
$  wget https://data.gharchive.org/2021-01-01-18.json.gz && gzip -d 2021-01-01-18.json.gz
$  heaptrack target/release/examples/streaming_matches_usage 2021-01-01-18.json.gz '$..sha'
```
![Comparison of memory usage](/img/memory-streaming-comparison.png)


### Results

The result of running a query is a sequence of matched values, delimited by newlines.
Alternatively, passing `--result count` returns only the number of matches, which might be much faster.
For other result modes consult the `--help` usage page.

## Installation

See [Releases](https://github.com/V0ldek/rsonpath/releases/latest) for precompiled binaries for
all first-class support targets.

### `cargo`

Easiest way to install is via [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```console,ignore
$ cargo install rsonpath
...
```

### Native CPU optimizations

If maximum speed is paramount, you should install `rsonpath` with native CPU instructions support.
This will result in a binary that is _not_ portable and might work incorrectly on any other machine,
but will squeeze out every last bit of throughput.

To do this, run the following `cargo install` variant:

```console,ignore
$ RUSTFLAGS="-C target-cpu=native" cargo install rsonpath
...
```

Check out [the relevant chapter in the rsonbook](https://v0ldek.github.io/rsonpath/user/installation/manual.html).

## Query language

The project is actively developed and currently supports only a subset of the JSONPath query language.
A query is a sequence of segments, each containing one or more selectors.

### Supported segments

| Segment                        | Syntax                           | Supported | Since  | Tracking Issue |
|--------------------------------|----------------------------------|-----------|--------|---------------:|
| Child segment (single)         | `[<selector>]`                   | ✔️        | v0.1.0 |                |
| Child segment (multiple)       | `[<selector1>,...,<selectorN>]`  | ❌        |        |                |
| Descendant segment (single)    | `..[<selector>]`                 | ✔️        | v0.1.0 |                |
| Descendant segment (multiple)  | `..[<selector1>,...,<selectorN>]`| ❌        |        |                |

### Supported selectors

| Selector                                 | Syntax                           | Supported | Since  | Tracking Issue |
|------------------------------------------|----------------------------------|-----------|--------|---------------:|
| Root                                     | `$`                              | ✔️        | v0.1.0 |                |
| Name                                     | `.<member>`, `[<member>]`        | ✔️        | v0.1.0 |                |
| Wildcard                                 | `.*`, `..*`, `[*]`               | ✔️        | v0.4.0 |                |
| Index (array index)                      | `[<index>]`                      | ✔️        | v0.5.0 |                |
| Index (array index from end)             | `[-<index>]`                     | ❌        |        |                |
| Array slice (forward, positive bounds)   | `[<start>:<end>:<step>]`         | ✔️        | v0.9.0       | [#152](https://github.com/V0ldek/rsonpath/issues/152) |
| Array slice (forward, arbitrary bounds)  | `[<start>:<end>:<step>]`         | ❌        |        |                |
| Array slice (backward, arbitrary bounds) | `[<start>:<end>:-<step>]`        | ❌        |        |                |
| Filters &ndash; existential tests        | `[?<path>]`                      | ❌        |        | [#154](https://github.com/V0ldek/rsonpath/issues/154) |
| Filters &ndash; const atom comparisons   | `[?<path> <binop> <atom>]`       | ❌        |        | [#156](https://github.com/V0ldek/rsonpath/issues/156) |
| Filters &ndash; logical expressions      | `&&`, `\|\|`, `!`                | ❌        |        |                |
| Filters &ndash; nesting                  | `[?<expr>[?<expr>]...]`          | ❌        |        |                |
| Filters &ndash; arbitrary comparisons    | `[?<path> <binop> <path>]`       | ❌        |        |                |
| Filters &ndash; function extensions      | `[?func(<path>)]`                | ❌        |        |                |

## Supported platforms

The crate is continuously built for all Tier 1 Rust targets, and tests are continuously ran for targets that can be ran with GitHub action images.
SIMD is supported only on x86/x86_64 platforms.

| Target triple             | nosimd build | SIMD support        | Continuous testing | Tracking issues |
|:--------------------------|:-------------|:--------------------|:-------------------|----------------:|
| aarch64-unknown-linux-gnu | ✔️          | ❌                  | ✔️                | [#21](https://github.com/V0ldek/rsonpath/issues/21), [#115](https://github.com/V0ldek/rsonpath/issues/115) |
| i686-unknown-linux-gnu    | ✔️          | ✔️                  | ✔️                | |
| x86_64-unknown-linux-gnu  | ✔️          | ✔️                  | ✔️                | |
| x86_64-apple-darwin       | ✔️          | ✔️                  | ✔️                | |
| i686-pc-windows-gnu       | ✔️          | ✔️                  | ✔️                | |
| i686-pc-windows-msvc      | ✔️          | ✔️                  | ✔️                | |
| x86_64-pc-windows-gnu     | ✔️          | ✔️                  | ✔️                | |
| x86_64-pc-windows-msvc    | ✔️          | ✔️                  | ✔️                | |

### SIMD support

SIMD support is enabled on a module-by-module basis. Generally, any CPU released in the past
decade supports AVX2, which enables all available optimizations.

Older CPUs with SSE2 or higher get partial support. You can check what exactly is enabled
with `rq --version` &ndash; check the `SIMD support` field:

```console,ignore
$ rq --version
rq 0.9.1

Commit SHA:      c024e1bab89610455537b77aed249d2a05a81ed6
Features:        default,simd
Opt level:       3
Target triple:   x86_64-unknown-linux-gnu
Codegen flags:   link-arg=-fuse-ld=lld
SIMD support:    avx2;fast_quotes;fast_popcnt
```

The `fast_quotes` capability depends on the `pclmulqdq` instruction,
and `fast_popcnt` on the `popcnt` instruction.

## Caveats and limitations

### JSONPath

Not all selectors are supported, see the support table above.

### Duplicate keys

The engine assumes that every object in the input JSON has no duplicate keys.
Behavior on duplicate keys is not guaranteed to be stable, but currently
the engine will simply match the _first_ such key.

```console
$ rq '$.key' --json '{"key":"value","key":"other value"}'
"value"

```

### Unicode

The engine does _not_ parse unicode escape sequences in member names.
This means that a key `"a"` is different from a key `"\u0041"`, even though semantically they represent the same string.
This is actually as-designed with respect to the current JSONPath spec.
Parsing unicode sequences is costly, so the support for this was postponed
in favour of high performance. This is tracked as [#117](https://github.com/v0ldek/rsonpath/issues/117).

## Contributing

The gist is: fork, implement, make a PR back here. More details are in the [CONTRIBUTING](/CONTRIBUTING.md) doc.

### Build & test

The dev workflow utilizes [`just`](https://github.com/casey/just).
Use the included `Justfile`. It will automatically install Rust for you using the `rustup` tool if it detects there is no Cargo in your environment.

```console,ignore
$ just build
...
$ just test
...
```

## Benchmarks

Benchmarks for `rsonpath` are located in a [separate repository](https://github.com/v0ldek/rsonpath-benchmarks),
included as a [git submodule](https://git-scm.com/book/en/v2/Git-Tools-Submodules) in this main repository.

Easiest way to run all the benchmarks is `just bench`. For details, look at the README in the submodule.

## Background

We have a paper on `rsonpath` to be published at [ASPLOS '24](https://www.asplos-conference.org/asplos2024/)! You can read it
[here](/pdf/supporting-descendants-in-simd-accelerated-jsonpath.pdf).

This project was conceived as [my thesis](/pdf/fast-execution-of-jsonpath-queries.pdf). You can read it for details on the theoretical
background on the engine and details of its implementation.

## Dependencies

Showing direct dependencies, for full graph see below.

```bash
cargo tree --package rsonpath --edges normal --depth 1
```

<!-- rsonpath dependencies start -->
```ini
rsonpath v0.9.1 (/home/mat/src/rsonpath/crates/rsonpath)
├── clap v4.5.4
├── color-eyre v0.6.3
├── eyre v0.6.12
├── log v0.4.21
├── rsonpath-lib v0.9.1 (/home/mat/src/rsonpath/crates/rsonpath-lib)
├── rsonpath-syntax v0.3.1 (/home/mat/src/rsonpath/crates/rsonpath-syntax)
└── simple_logger v4.3.3
[build-dependencies]
├── rustflags v0.1.5
└── vergen v8.3.1
    [build-dependencies]
```
<!-- rsonpath dependencies end -->

```bash
cargo tree --package rsonpath-lib --edges normal --depth 1
```

<!-- rsonpath-lib dependencies start -->
```ini
rsonpath-lib v0.9.1 (/home/mat/src/rsonpath/crates/rsonpath-lib)
├── arbitrary v1.3.2
├── cfg-if v1.0.0
├── log v0.4.21
├── memmap2 v0.9.4
├── nom v7.1.3
├── rsonpath-syntax v0.3.1 (/home/mat/src/rsonpath/crates/rsonpath-syntax)
├── smallvec v1.13.2
├── static_assertions v1.1.0
├── thiserror v1.0.58
└── vector-map v1.0.1
```
<!-- rsonpath-lib dependencies end -->

### Justification

- `clap` &ndash; standard crate to provide the CLI.
- `color-eyre`, `eyre` &ndash; more accessible error messages for the parser.
- `log`, `simple-logger` &ndash; diagnostic logs during compilation and execution.
- `cfg-if` &ndash; used to support SIMD and no-SIMD versions.
- `memmap2` &ndash; for fast reading of source files via a memory map instead of buffered copies.
- `nom` &ndash; for parser implementation.
- `smallvec` &ndash; crucial for small-stack performance.
- `static_assertions` &ndash; additional reliability by some constant assumptions validated at compile time.
- `thiserror` &ndash; idiomatic `Error` implementations.
- `vector_map` &ndash; used in the query compiler for measurably better performance.

## Full dependency tree

```bash
cargo tree --package rsonpath --edges normal
```

<!-- rsonpath-full dependencies start -->
```ini
rsonpath v0.9.1 (/home/mat/src/rsonpath/crates/rsonpath)
├── clap v4.5.4
│   ├── clap_builder v4.5.2
│   │   ├── anstream v0.6.13
│   │   │   ├── anstyle v1.0.6
│   │   │   ├── anstyle-parse v0.2.3
│   │   │   │   └── utf8parse v0.2.1
│   │   │   ├── anstyle-query v1.0.2
│   │   │   │   └── windows-sys v0.52.0
│   │   │   │       └── windows-targets v0.52.4
│   │   │   │           ├── windows_aarch64_gnullvm v0.52.4
│   │   │   │           ├── windows_aarch64_msvc v0.52.4
│   │   │   │           ├── windows_i686_gnu v0.52.4
│   │   │   │           ├── windows_i686_msvc v0.52.4
│   │   │   │           ├── windows_x86_64_gnu v0.52.4
│   │   │   │           ├── windows_x86_64_gnullvm v0.52.4
│   │   │   │           └── windows_x86_64_msvc v0.52.4
│   │   │   ├── anstyle-wincon v3.0.2
│   │   │   │   ├── anstyle v1.0.6
│   │   │   │   └── windows-sys v0.52.0 (*)
│   │   │   ├── colorchoice v1.0.0
│   │   │   └── utf8parse v0.2.1
│   │   ├── anstyle v1.0.6
│   │   ├── clap_lex v0.7.0
│   │   ├── strsim v0.11.1
│   │   └── terminal_size v0.3.0
│   │       ├── rustix v0.38.32
│   │       │   ├── bitflags v2.5.0
│   │       │   ├── errno v0.3.8
│   │       │   │   ├── libc v0.2.153
│   │       │   │   └── windows-sys v0.52.0 (*)
│   │       │   ├── libc v0.2.153
│   │       │   ├── linux-raw-sys v0.4.13
│   │       │   └── windows-sys v0.52.0 (*)
│   │       └── windows-sys v0.48.0
│   │           └── windows-targets v0.48.5
│   │               ├── windows_aarch64_gnullvm v0.48.5
│   │               ├── windows_aarch64_msvc v0.48.5
│   │               ├── windows_i686_gnu v0.48.5
│   │               ├── windows_i686_msvc v0.48.5
│   │               ├── windows_x86_64_gnu v0.48.5
│   │               ├── windows_x86_64_gnullvm v0.48.5
│   │               └── windows_x86_64_msvc v0.48.5
│   └── clap_derive v4.5.4 (proc-macro)
│       ├── heck v0.5.0
│       ├── proc-macro2 v1.0.79
│       │   └── unicode-ident v1.0.12
│       ├── quote v1.0.35
│       │   └── proc-macro2 v1.0.79 (*)
│       └── syn v2.0.58
│           ├── proc-macro2 v1.0.79 (*)
│           ├── quote v1.0.35 (*)
│           └── unicode-ident v1.0.12
├── color-eyre v0.6.3
│   ├── backtrace v0.3.71
│   │   ├── addr2line v0.21.0
│   │   │   └── gimli v0.28.1
│   │   ├── cfg-if v1.0.0
│   │   ├── libc v0.2.153
│   │   ├── miniz_oxide v0.7.2
│   │   │   └── adler v1.0.2
│   │   ├── object v0.32.2
│   │   │   └── memchr v2.7.2
│   │   └── rustc-demangle v0.1.23
│   │   [build-dependencies]
│   │   └── cc v1.0.90
│   ├── eyre v0.6.12
│   │   ├── indenter v0.3.3
│   │   └── once_cell v1.19.0
│   ├── indenter v0.3.3
│   ├── once_cell v1.19.0
│   └── owo-colors v3.5.0
├── eyre v0.6.12 (*)
├── log v0.4.21
├── rsonpath-lib v0.9.1 (/home/mat/src/rsonpath/crates/rsonpath-lib)
│   ├── cfg-if v1.0.0
│   ├── log v0.4.21
│   ├── memmap2 v0.9.4
│   │   └── libc v0.2.153
│   ├── nom v7.1.3
│   │   ├── memchr v2.7.2
│   │   └── minimal-lexical v0.2.1
│   ├── rsonpath-syntax v0.3.1 (/home/mat/src/rsonpath/crates/rsonpath-syntax)
│   │   ├── nom v7.1.3 (*)
│   │   ├── owo-colors v4.0.0
│   │   ├── thiserror v1.0.58
│   │   │   └── thiserror-impl v1.0.58 (proc-macro)
│   │   │       ├── proc-macro2 v1.0.79 (*)
│   │   │       ├── quote v1.0.35 (*)
│   │   │       └── syn v2.0.58 (*)
│   │   └── unicode-width v0.1.11
│   ├── smallvec v1.13.2
│   ├── static_assertions v1.1.0
│   ├── thiserror v1.0.58 (*)
│   └── vector-map v1.0.1
│       ├── contracts v0.4.0 (proc-macro)
│       │   ├── proc-macro2 v1.0.79 (*)
│       │   ├── quote v1.0.35 (*)
│       │   └── syn v1.0.109
│       │       ├── proc-macro2 v1.0.79 (*)
│       │       ├── quote v1.0.35 (*)
│       │       └── unicode-ident v1.0.12
│       └── rand v0.7.3
│           ├── getrandom v0.1.16
│           │   ├── cfg-if v1.0.0
│           │   ├── libc v0.2.153
│           │   └── wasi v0.9.0+wasi-snapshot-preview1
│           ├── libc v0.2.153
│           ├── rand_chacha v0.2.2
│           │   ├── ppv-lite86 v0.2.17
│           │   └── rand_core v0.5.1
│           │       └── getrandom v0.1.16 (*)
│           ├── rand_core v0.5.1 (*)
│           └── rand_hc v0.2.0
│               └── rand_core v0.5.1 (*)
├── rsonpath-syntax v0.3.1 (/home/mat/src/rsonpath/crates/rsonpath-syntax) (*)
└── simple_logger v4.3.3
    ├── colored v2.1.0
    │   ├── lazy_static v1.4.0
    │   └── windows-sys v0.48.0 (*)
    ├── log v0.4.21
    ├── time v0.3.34
    │   ├── deranged v0.3.11
    │   │   └── powerfmt v0.2.0
    │   ├── itoa v1.0.11
    │   ├── libc v0.2.153
    │   ├── num-conv v0.1.0
    │   ├── num_threads v0.1.7
    │   │   └── libc v0.2.153
    │   ├── powerfmt v0.2.0
    │   ├── time-core v0.1.2
    │   └── time-macros v0.2.17 (proc-macro)
    │       ├── num-conv v0.1.0
    │       └── time-core v0.1.2
    └── windows-sys v0.48.0 (*)
[build-dependencies]
├── rustflags v0.1.5
└── vergen v8.3.1
    ├── anyhow v1.0.81
    ├── cargo_metadata v0.18.1
    │   ├── camino v1.1.6
    │   │   └── serde v1.0.197
    │   │       └── serde_derive v1.0.197 (proc-macro)
    │   │           ├── proc-macro2 v1.0.79 (*)
    │   │           ├── quote v1.0.35 (*)
    │   │           └── syn v2.0.58 (*)
    │   ├── cargo-platform v0.1.8
    │   │   └── serde v1.0.197 (*)
    │   ├── semver v1.0.22
    │   │   └── serde v1.0.197 (*)
    │   ├── serde v1.0.197 (*)
    │   ├── serde_json v1.0.115
    │   │   ├── itoa v1.0.11
    │   │   ├── ryu v1.0.17
    │   │   └── serde v1.0.197 (*)
    │   └── thiserror v1.0.58 (*)
    ├── cfg-if v1.0.0
    ├── regex v1.10.4
    │   ├── aho-corasick v1.1.3
    │   │   └── memchr v2.7.2
    │   ├── memchr v2.7.2
    │   ├── regex-automata v0.4.6
    │   │   ├── aho-corasick v1.1.3 (*)
    │   │   ├── memchr v2.7.2
    │   │   └── regex-syntax v0.8.3
    │   └── regex-syntax v0.8.3
    ├── rustc_version v0.4.0
    │   └── semver v1.0.22 (*)
    └── time v0.3.34
        ├── deranged v0.3.11 (*)
        ├── itoa v1.0.11
        ├── libc v0.2.153
        ├── num-conv v0.1.0
        ├── num_threads v0.1.7 (*)
        ├── powerfmt v0.2.0
        └── time-core v0.1.2
    [build-dependencies]
    └── rustversion v1.0.14 (proc-macro)
```
<!-- rsonpath-full dependencies end -->
