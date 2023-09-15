# GCamera Tools

This is a tool for manipulating images taken with Google Camera.

Images taken with Google Camera contain additional data in them than a
normal JPEG. This tool allows you to extract or remove that data from
the images.

## TODO

* Use `serde`?
* Find JPEG libraries for Rust?

## Testing

There are unit tests written. I have been using `cargo-llvm-cov` for unit
testing, since it allows for also getting code coverage, and does not require
a complete rebuild, like `cargo tarpaulin` does

### Using cargo-llvm-cov

1. install it with `cargo install cargo-llvm-cov`
2. install necessary component with `rustup component add llvm-tools-preview`
3. Run it with `cargo llvm-cov`
4. Use the subcommand `cargo llvm-cov report --open` to generate and open html report
5. Use the subcommand  `cargo llvm-cov report --lcov --output-path lcov.info` to create coverage file that can be read in by VS Code coverage gutters