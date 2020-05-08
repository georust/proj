[![Build Status](https://travis-ci.org/georust/proj-sys.svg?branch=master)](https://travis-ci.org/georust/proj-sys)

# Low-level bindings for PROJ v7.0.x
**This is a [`*-sys`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#a-sys-packages) crate; you shouldn't use its API directly.** The [`proj`](https://github.com/georust/proj) crate is designed for general use.

A guide to the functions can be found here: https://proj.org/development/reference/functions.html. Run `cargo doc (optionally --open)` to generate the crate documentation.

## Requirements

By default, this crate depends on a pre-built library, so `libproj` (via `PROJ v7.0.x`) must be present on your system. While this crate may be backwards-compatible with older PROJ 6 versions, this is neither tested or supported.

## Optional Features
Enable these in your `Cargo.toml` like so:

`proj-sys = { version = "0.16", features = ["pkg_config"] }`  
`proj-sys = { version = "0.16", features = ["bundled_proj"] }`  

Note that these features are **mutually exclusive**.

1. `pkg_config` (Linux and macOS targets)
    - uses [`pkg-config`](https://en.wikipedia.org/wiki/Pkg-config) to add search paths to the build script. Requires `pkg-config` to be installed (available on Homebrew, Macports, apt etc.)
2. `bundled_proj` (Linux and macOS targets):
    - allow the crate to internally build and depend on a bundled PROJ library. This may make it easier to compile the crate, but is not yet thoroughly tested. Note that SQLite3 must be present on your system if you wish to use this feature.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
