#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! `proj` provides bindings to the [PROJ](https://proj.org) v7.0.x API
//!
//! Two coordinate transformation operations are currently provided: [projection](struct.Proj.html#method.project)
//! (and inverse projection)
//! and [conversion](struct.Proj.html#method.convert).
//! Projection is intended for transformations between geodetic and projected coordinates,
//! and vice versa (inverse projection), while conversion is intended for transformations between projected
//! coordinate systems. The PROJ [documentation](https://proj.org/operations/index.html)
//! explains the distinction between these operations.
//!
//! Anything that can be converted into a [`geo-types`](https://docs.rs/geo-types) `Point` via the `Into`
//! trait can be used as input for the projection and conversion functions, and methods
//! for [conversion](struct.Proj.html#method.convert_array) and [projection](struct.Proj.html#method.project_array)
//! of slices of `Point`s are available.
//! # Usage
//! Instantiating a transformation instance (`Proj`) is a two-step process:
//! 1. Create a new `Context` by calling `Context::new`. This context may be modified to enable network downloads or modify search paths if you wish;
//! 2. Create the transformation instance (`Proj`) by calling [`transform()`](proj/struct.Context.html#method.transform) or [`transform_known_crs()`](proj/struct.Context.html#method.transform_known_crs).
//!
//! ## Network, Cache, and Search Path Functionality
//!
//! ### Grid File Download
//! `proj` supports [network grid download](https://proj.org/usage/network.html) functionality.
//! Network access is **disabled** by default, and
//! can be activated by passing a `true` `bool` to [`enable_network()`](proj/struct.Context.html#method.enable_network).
//! Network functionality status can be queried with
//! `network_enabled`, and the download endpoint can be queried and set using `get_url_endpoint` and `set_url_endpoint`.
//!
//! #### Grid File Cache
//! Up to 300 mb of downloaded grids are cached to save bandwidth: This cache can be enabled or disabled using [`grid_cache_enable`](proj/struct.Context.html#method.grid_cache_enable).
//!
//! ### Search Path Modification
//! The path used to search for resource files can be modified using [`set_search_paths`](proj/struct.Context.html#method.set_search_paths)
//!
//!
//! # Requirements
//!
//! By default, this requires `libproj` 7.0.x to be present on your system. While this crate may be backwards-compatible with older PROJ 6 versions, this is neither tested nor supported.
//!
//! Two features are available:
//!
//! `proj = { version = "0.16.1", features = ["pkg_config"] }`  
//! `proj = = { version = "0.16.1", features = ["bundled_proj"] }`  
//!
//! The `pkg_config` feature enables the use of `pkg-config` when linking against `libproj` – note that `pkg-config` must be available on your system.
//!
//! The `bundled_proj` feature allows you to link against a `libproj` version included with (and built from source by) the `proj-sys` crate, upon which this crate is built. To do so, enable the `bundled_proj` Cargo feature. Note that this feature requires sqlite3 to be available on your system.
//!
//! # Example
//!
//! ```
//! use assert_approx_eq::assert_approx_eq;
//! extern crate proj;
//! use proj::Proj;
//!
//! extern crate geo_types;
//! use geo_types::Point;
//!
//! let from = "EPSG:2230";
//! let to = "EPSG:26946";
//! let nad_ft_to_m = Proj::new_known_crs(&from, &to, None).unwrap();
//! let result = nad_ft_to_m
//!     .convert(Point::new(4760096.421921f64, 3744293.729449f64))
//!     .unwrap();
//! assert_approx_eq!(result.x(), 1450880.29f64, 1.0e-2);
//! assert_approx_eq!(result.y(), 1141263.01f64, 1.0e-2);
//! ```

mod network;
mod proj;

pub use crate::proj::Area;
pub use crate::proj::TransformBuilder;
pub use crate::proj::Info;
pub use crate::proj::Proj;
pub use crate::proj::ProjError;
pub use crate::proj::Projinfo;
