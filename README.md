# ohua-rust-runtime

This is a Rust-based implementation of the Ohua-Runtime.

## Project Structure

The project is split into two modules, `ohua_codegen` and `ohua_runtime`. The codegen module contains the procedural macro invoked at compile time to generate the algorithms. Some recurring functionality in the generated code has been outsourced and is instead loaded from the runtime library as necessary.

## Documentation

A documentation for the project can be obtained by running
```
RUSTDOCFLAGS="--document-private-items" cargo doc
```
in the respective libraries or by reading the docs inline.

## License

This project is licensed under the Eclipse Public License version 1.0. For further information please refer to the `LICENSE` file.
