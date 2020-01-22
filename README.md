Foundry [![Build Status](https://travis-ci.com/CodeChain-io/foundry.svg?branch=master)](https://travis-ci.com/CodeChain-io/foundry) [![Gitter: CodeChain](https://img.shields.io/badge/gitter-codechain-4AB495.svg)](https://gitter.im/CodeChain-io/codechain) [![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
==============

Foundry is a programmable open source blockchain engine.

## Build

Download Foundry code

```sh
git clone git@github.com:CodeChain-io/foundry.git
cd foundry
```

Build in release mode

```sh
cargo build --release
```

This will produce an executable in the `./target/release` directory.

### Building From Source

#### Build Dependencies
Foundry requires Rust version 1.40.0 to build. Using [rustup](https://rustup.rs/ "rustup URL") is recommended.

- For Linux Systems:
  - Ubuntu

    > `gcc`, `g++` and `make` are required for installing packages.
    ```sh
    $ curl https://sh.rustup.rs -sSf | sh
    ```
        

- For Mac Systems:
  - MacOS 10.13.2 (17C88) tested
    > `clang` is required for installing packages.

    ```sh
    $ curl https://sh.rustup.rs -sSf | sh
    ```
        

- For Windows Systems:
  - Currently not supported for Windows. If on a Windows system, please install [WSL](https://docs.microsoft.com/en-us/windows/wsl/install-win10) to continue as Ubuntu.

Please make sure that all of the binaries above are included in your `PATH`. These conditions must be fulfilled before building Foundry from source.


Download Foundry's source code and go into its directory.
```sh
git clone git@github.com:CodeChain-io/foundry.git
cd foundry
```

#### Build as Release Version
```sh
cargo build --release
```

This will produce an executable in the ./target/release directory.

## Run

To run Foundry, just run

```sh
./target/release/foundry -c solo
```
You can create a block by sending a transaction through [JSON-RPC](https://github.com/CodeChain-io/foundry/blob/master/spec/JSON-RPC.md) or [JavaScript SDK](https://api.codechain.io/).

## Formatting

Make sure you run `rustfmt` before creating a PR to the repo. You need to install the nightly-2019-12-19 version of `rustfmt`.

```sh
rustup toolchain install nightly-2019-12-19
rustup component add rustfmt --toolchain nightly-2019-12-19
```

To run `rustfmt`,

```sh
cargo +nightly-2019-12-19 fmt
```

## Linting

You should run `clippy` also. This is a lint tool for rust. It suggests more efficient/readable code.
You can see [the clippy document](https://rust-lang.github.io/rust-clippy/master/index.html) for more information.
You need to install the nightly-2019-12-19 version of `clippy`.

### Install
```sh
rustup toolchain install nightly-2019-12-19
rustup component add clippy --toolchain nightly-2019-12-19
```

### Run

```sh
cargo +nightly-2019-12-19 clippy --all --all-targets
```

## Testing

Developers are strongly encouraged to write unit tests for new code, and to submit new unit tests for old code. Unit tests can be compiled and run with: `cargo test --all`. For more details, please reference [Unit Tests](https://github.com/CodeChain-io/codechain/wiki/Unit-Tests).

## License
CodeChain is licensed under the AGPL License - see the [LICENSE](https://github.com/CodeChain-io/foundry/blob/master/LICENSE) file for details
