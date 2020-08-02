#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! # Low-level bindings for PROJ v7.1.x
//! **This is a [`*-sys`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#a-sys-packages) crate; you shouldn't use its API directly.** The [`proj`](https://github.com/georust/proj) crate is designed for general use.
//! 
//! A guide to the functions can be found on [proj.org](https://proj.org/development/reference/functions.html). Run `cargo doc (optionally --open)` to generate the crate documentation.
//! 
//! ## Requirements
//! 
//! By default, `libproj` (via `PROJ v7.1.x`) must be present on your system. While this crate may be backwards-compatible with older PROJ 7 and PROJ 6 versions, this is neither tested or supported.
//! 
//! ## Optional Features
//! Enable these in your `Cargo.toml` like so:
//! 
//! `proj-sys = { version = "0.18.2", features = ["bundled_proj"] }`  
//! `proj-sys = { version = "0.18.2", features = ["pkg_config"] }`  
//! 
//! Note that these features are **mutually exclusive**.
//! 
//! 1. `bundled_proj` (Linux and macOS targets):
//!     - allow the crate to internally build and depend on a bundled `libproj`. Note that SQLite3 and `libtiff` must be present on your system if you wish to use this feature, and that it builds `libproj` **without** its native network functionality; you will have to implement your own set of callbacks if you wish to make use of them (see the [`proj`](https://crates.io/crates/proj) crate for an example).
//! 2. `pkg_config` (Linux and macOS targets)
//!     - uses [`pkg-config`](https://en.wikipedia.org/wiki/Pkg-config) to add search paths to the build script. Requires `pkg-config` to be installed (available on Homebrew, Macports, apt etc.)
//! 
//! ## License
//! 
//! Licensed under either of
//! 
//!  * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
//!  * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
//! 
//! at your option.


#[cfg(not(feature = "nobuild"))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "nobuild")]
include!("bindings_docs-rs.rs");
