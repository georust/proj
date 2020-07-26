#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! `proj` provides bindings to the [PROJ](https://proj.org) v7.0.x API
//!
//! Two coordinate transformation operations are currently provided: _projection_
//! (and inverse projection) and _conversion_.
//! Projection is intended for transformations between geodetic and projected coordinates
//! and vice versa (inverse projection), while conversion is intended for transformations between projected
//! coordinate systems. The PROJ [documentation](https://proj.org/operations/index.html)
//! explains the distinction between these operations in more detail.
//!
//! Anything that can be converted into a [`geo-types`](https://docs.rs/geo-types) `Point` via the `Into`
//! trait can be used as input for the projection and conversion functions, and methods
//! for [conversion](struct.Proj.html#method.convert_array) and [projection](struct.Proj.html#method.project_array)
//! of slices of `Point`s are available.
//!
//! # Usage
//! There are two options for creating a transformation:
//!
//! 1. If you don't require additional grids or other customisation:
//!     - Call `Proj::new` or `Proj::new_known_crs`. This creates a transformation instance ([`Proj`](proj/struct.Proj.html))
//! 2. If you require a grid for the transformation you wish to carry out, or you need to customise the search path or the grid endpoint:
//!     - Create a new [`ProjBuilder`](proj/struct.ProjBuilder.html) by calling `ProjBuilder::new()`. It may be modified to enable network downloads, disable the grid, cache or modify search paths;
//!     - Call [`ProjBuilder.proj()`](proj/struct.ProjBuilder.html#method.proj) or [`ProjBuilder.proj_known_crs()`](proj/struct.ProjBuilder.html#method.proj_known_crs). This creates a transformation instance (`Proj`)
//!
//! **Note**:
//!
//! 1. Both `ProjBuilder` and `Proj` implement the [`Info`](proj/trait.Info.html) trait, which can be used to get information about the current state of the `PROJ` instance;
//! 2. `Proj::new()` and `ProjBuilder::proj()` have the same signature;
//! 3. `Proj::new_known_crs()` and `ProjBuilder::proj_known_crs()` have the same signature.
//!
//! ## Network, Cache, and Search Path Functionality
//!
//! ### Grid File Download
//! `proj` supports [network grid download](https://proj.org/usage/network.html) functionality.
//! Network access is **disabled** by default, and
//! can be activated by passing a `true` `bool` to [`enable_network()`](proj/struct.ProjBuilder.html#method.enable_network).
//! Network functionality status can be queried with
//! `network_enabled`, and the download endpoint can be queried and set using `get_url_endpoint` and `set_url_endpoint`.
//!
//! #### Grid File Cache
//! Up to 300 mb of downloaded grids are cached to save bandwidth: This cache can be enabled or disabled using [`grid_cache_enable`](proj/struct.ProjBuilder.html#method.grid_cache_enable).
//!
//! ### Search Path Modification
//! The path used to search for resource files can be modified using [`set_search_paths`](proj/struct.ProjBuilder.html#method.set_search_paths)
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
pub use crate::proj::Info;
pub use crate::proj::Proj;
pub use crate::proj::ProjBuilder;
pub use crate::proj::ProjError;
pub use crate::proj::Projinfo;
