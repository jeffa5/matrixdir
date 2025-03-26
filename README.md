# Matrixdir

A file-first approach to message storage for chat programs.

Matrixdir is exploring some ideas from maildir for Matrix chat programs.

Through Matrix bridges it aims to be suitable to more than just Matrix.

## Building

Its just a Rust crate at the moment, library and binary so you can do the usual cargo invocations:

```sh
# build the docs and open them in a browser
cargo doc

# build the library and binary
cargo build

# run the binary
cargo run
```

The binary is aiming to be a bit of a testing and debugging tool so don't currently expect stability from what it does.

## More info

See [the blog post](https://www.jeffas.net/blog/matrixdir/)
