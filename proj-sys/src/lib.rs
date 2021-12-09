#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! # Low-level bindings for PROJ v8.2.x
//!
//! **This is a
//! [`*-sys`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#-sys-packages)
//! crate; you shouldn't use its API directly.** See the
//! [`proj`](https://github.com/georust/proj) crate for general use.
//!
//! A guide to the functions can be found here:
//! <https://proj.org/development/reference/functions.html>.
//!
//! By default, the crate will search for an acceptable existing `libproj`
//! installation on your system using
//! [pkg-config](https://www.freedesktop.org/wiki/Software/pkg-config/).
//!
//! If an acceptable installation is not found, proj-sys will attempt to build
//! libproj from source bundled in the crate.
//!
//! ## Features
//!
//! `bundled_proj` - forces building libproj from source even if an acceptable
//! version could be found on your system.  Note that SQLite3 and `libtiff` must be
//! present on your system if you wish to use this feature, and that it builds
//! `libproj` **without** its native network functionality; you will have to
//! implement your own set of callbacks if you wish to make use of them (see the
//! [`proj`](https://crates.io/crates/proj) crate for an example).

#[cfg(not(feature = "nobuild"))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "nobuild")]
include!("bindings_docs-rs.rs");
