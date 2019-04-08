![Ohua for Rust](https://raw.githubusercontent.com/ohua-dev/ohua/master/logos/fileIcons/iconFull/rust/export_wide_lang_transparent.png)

# ohua-rust-runtime

This is a Rust-based implementation of the [Ohua](https://ohua-dev.github.io/) Runtime.

## Project Structure

The project is split into two modules, `ohua_codegen` and `ohua_runtime`.
The codegen module forms the backend of the Ohua compiler infrastructure and generates the code necessary to integrate an algorithm in native Rust code.
Some recurring functionality and types used throughout the generated code have been bundled into the `ohua_runtime` module.

## Getting Started

### The `ohuac` compiler

`cargo` and the libraries in this repository only provide the backend for Ohua. So, you will additionally need to have [`ohuac`](https://github.com/ohua-dev/ohuac), the standalone compiler, installed. Please follow the installation instructions in the repository.

Due to the fact, that we are still heavily developing both this runtime and the compiler, use `master` branch versions of both to make sure the tools are interoperable.

The `ohauc` binary will automatically be invoked by `rustc`, so you don't have to familiarize yourself with its usage.

### Using the right toolchain

Due to the fact, that the backend uses unstable features such as `fnbox`, Ohua can currently only be used in nightly Rust.
When using [`rustup`](https://rustup.rs), you can simply run
```
rustup override set nightly
```
in your project root.

### Using Ohua in your project

Add Ohua to your project's dependencies:
```toml
[dependencies]
ohua_codegen = { git = "https://github.com/ohua-dev/ohua-rust-runtime" }
ohua_runtime = { git = "https://github.com/ohua-dev/ohua-rust-runtime" }
```

Then, specify the necessary feature flags in your projects' `main.rs` or `lib.rs`. Also, don't forget the `extern crate` specification, if you are using the 2015 edition.
```rust
#![feature(proc_macro_hygiene, fnbox)]
```

Now you are good to go! Wherever you want to use an Ohua algorithm in your code, import the macro and link your algorithm file! Assuming, you have an algorithm file saved at `src/foo/bar.ohuac` and want to use this in `src/foo/mod.rs`, you would write:
```rust
use ohua_codegen::ohua; // the codegen macro

fn something() {
    // This invokes the algorithm with no arguments, expecting no return value.
    #[ohua]
    foo::bar();

    // You can take arguments and return values, too!
    #[ohua]
    let result = foo::bar(some_var, 42);
}
```

Note that the algorithm is always invoked by specifying the _complete_ path, separated by double-colons and omitting the `.ohuac` file extension.

For a complete example, you can have a look at the [example folder](example/) or the testcases.

## Testing

Ohua's Rust backend comes with a variety of tests that are designed to verify the correct operation of Ohua's core functionalities.

To run the tests, simply switch into the `testcases` folder and fire up `cargo test`.
Any failing tests have to be de-activated using the `unimplemented!` macro, since errors during code generation will force the whole test run to come to a stop.

## Documentation

A documentation for the project can be obtained by running
```
RUSTDOCFLAGS="--document-private-items" cargo doc
```
in the respective libraries or by reading the docs inline.
Please note that this project is still under heavy development and the documentation might be outdated.

## Caveats

Although Ohua algorithms can be defined in many dialects, only `ohuac` files, which use Rust-like syntax, are currently supported.

## License

This project is licensed under the Eclipse Public License version 1.0. For further information please refer to the `LICENSE` file.

The [Rust Logo](https://github.com/rust-lang-nursery/rust-artwork) is trademark of the Rust programming language.
