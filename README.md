[![Build Status](https://travis-ci.org/georust/proj-sys.svg?branch=master)](https://travis-ci.org/georust/proj-sys)

# Low-level bindings for PROJ v7.0.x
**This is a [`*-sys`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#a-sys-packages) crate; you shouldn't use its API directly.** The [`proj`](https://github.com/georust/proj) crate is designed for general use.

A guide to the functions can be found here: https://proj.org/development/reference/functions.html. Run `cargo doc (optionally --open)` to generate the crate documentation.

## Requirements

Sqlite3 must be present on your system.

By default, this crate depends on a pre-built library, so PROJ `v7.0.x` must be present on your system. While this crate may be backwards-compatible with older PROJ 6 versions, this is neither tested or supported.

## Using the Bundled PROJ

This crate can internally build and depend on a bundled PROJ `v7.0.0` library, which can be enabled via the "bundled_proj" feature. This might make it easier to compile the crate, but it is not thoroughly tested yet so it might not work on some platforms.

Currently this feature only supports Linux.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
