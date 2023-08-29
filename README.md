# GCamera Tools

This is a tool for manipulating images taken with Google Camera.

Images taken with Google Camera contain additional data in them than a
normal JPEG. This tool allows you to extract or remove that data from
the images.
## TODO

* Use `serde`?
* Find JPEG libraries for Rust?
* Use `clap` to have a better CLI interface.
* Look into using `cargo-nextest` for tests.

## Testing

There are unit tests written. I have been using `cargo-tarpaulin` for unit
testing, since it allows for also getting code coverage.
