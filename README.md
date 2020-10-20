Foundry
[![Build Status](https://travis-ci.com/CodeChain-io/foundry.svg?branch=master)](https://travis-ci.com/CodeChain-io/foundry)
[![chat](https://img.shields.io/discord/569610676205781012.svg?logo=discord)](https://discord.gg/xhpdXm7)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
==============

CodeChain Foundry is a blockchain engine based on a composable module system, called _Mold_.
Users can write their own modules and additionally bring those written by others in to construct an arbitrary blockchain application.
The reason why we provide such a composable and user-configurable module system is
because we want to make as much of an application configurable as possible and foster an ecosystem of reusable modules,
while reusing the underlying consensus engine across all the different kinds of applications.

On an execution of a transaction, that is essentially a state transition, the coordinator will be told to do so from the underlying consensus engine.
Then the coordinator literally coordinates multiple modules constituting an application
by invoking services exported by the modules to the coordinator in an appropriate order passing appropriate arguments.
Transactions will be delivered to the responsible modules, and that the modules will handle the executions of the transactions,
which might also involve communications with other modules.

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
Foundry requires Rust version 1.47.0 to build. Using [rustup](https://rustup.rs/ "rustup URL") is recommended.

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

Make sure you run `rustfmt` before creating a PR to the repo. You need to install the nightly-2020-10-20 version of `rustfmt`.

```sh
rustup toolchain install nightly-2020-10-20
rustup component add rustfmt --toolchain nightly-2020-10-20
```

To run `rustfmt`,

```sh
cargo +nightly-2020-10-20 fmt
```

## Linting

You should run `clippy` also. This is a lint tool for rust. It suggests more efficient/readable code.
You can see [the clippy document](https://rust-lang.github.io/rust-clippy/master/index.html) for more information.
You need to install the stable version of `clippy`.

### Install
```sh
rustup component add clippy
```

### Run

```sh
cargo clippy --all --all-targets
```

## Testing

Developers are strongly encouraged to write unit tests for new code, and to submit new unit tests for old code. Unit tests can be compiled and run with: `cargo test --all`. For more details, please reference [Unit Tests](https://github.com/CodeChain-io/codechain/wiki/Unit-Tests).

## License
CodeChain is licensed under the GPL License - see the [LICENSE](https://github.com/CodeChain-io/foundry/blob/master/LICENSE) file for details
