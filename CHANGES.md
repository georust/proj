# Changes

## 0.21.0
- Updated to `geo-types` v0.7.0 and `reqwest` v0.11.0

- TIFF support is now opt-in when building PROJ via the `bundled_proj` feature
    - <https://github.com/georust/proj/pull/58>

## 0.20.4
- Incorporate proj-sys repo
- Switch to GH actions

## 0.20.3
* Disable default features in Reqwest

## 0.20.0
* Add network control and grid download functionality

## 0.19.0
* Update to proj-sys 0.17.1

## 0.18.0
* Bump geo-types

## 0.17.1
* Fix docs
* Make Projinfo struct public
* Generalise array ops

## 0.17.0
* More extensive error-handling
* Error enum is now public

## 0.16.3
* add info() and set_search_paths methods (#30)

## 0.16.2
* Enable bundled_proj for macOS target

## 0.16.1
* Update to proj-sys v0.16.3 (PROJ 7.0.1)
* Re-export the bundled_proj feature introduced in proj-sys v0.15.0
* Re-export the pkg_config feature introduced in proj-sys v0.15.0

## 0.16.0
* Update to geo-types v0.5.0

## 0.15.0
* Update to proj-sys v0.13.0
* Update to use PROJ v7.0.0

## 0.14.4
* Add array projection method
* Fix potential leak of PJ object in `new_known_crs`

## 0.14.0
* Normalise input and output coordinate order to Lat, Lon / Easting, Northing for conversions between known CRS (#21)

## 0.13.0
* Updated to proj-sys 0.12.0 (PROJ 6.3)

## 0.12.1
- `convert` and `project` operations now accept any type that has an `Into<Point<T>>`impl. This is a backward-compatible API change
- New `Area` `bbox`es no longer need to be wrapped in an `Option`

## 0.10.9
* add bulk conversion (#17)

## 0.9.7
* Update to PROJ v6.2.
    * This requires a minimum PROJ version of 6.2.0

## 0.9.6
* Fix README example

## 0.9.5
* Fix doctests

## 0.9.3
* Destroy threading context before dropping Proj (#15)

## 0.9.2
* Ensure that errors are reset before projection / conversion calls

## 0.9.0
* Update proj-sys to v0.9.0
    * This requires a minimum PROJ.4 version of 6.0.0
* Add support for `proj_create_crs_to_crs` for creating a transformation object that is a pipeline between two known coordinate reference systems.

## 0.7.0
* Update proj-sys to v0.8.0
    * This requires a minimum PROJ.4 version of 5.2.0

## 0.6.0

* Update proj-sys to v0.7.0
    * This requires a minimum PROJ.4 version of 5.1.0
* Deprecate use of `pj_strerrno` in favour of proj_errno_string

## 0.5.0

* [Switch to `geo-types` crate](https://github.com/georust/rust-proj/pull/8)

## 0.4.0

* [Switch to `proj-sys` crate, and PROJ.4 v5.0.0 API](https://github.com/georust/rust-proj/pull/6)
    * Split operations into `project` and `convert`
    * `project` and `convert` return `Result`


## 0.3.0

* [Use `c_void` instead of unit](https://github.com/georust/rust-proj/pull/5)
    * Add example to README

