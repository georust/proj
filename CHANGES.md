## 0.28.0 - 2024-12-20

- Update to proj-sys 0.25.0 (libproj 9.4.0)
- BREAKING: Provide bundled bindings by default and move support for build time generated bindings behind the `buildtime_bindgen` feature of proj-sys
- Bump MSRV to 1.70

## 0.27.2

- Update to proj-sys 0.23.2 (libproj 9.2.1)
  - https://github.com/georust/proj/pull/167

## 0.27.1

- FIX: `network` feature fails to compile on aarch64/arm64
  - https://github.com/georust/proj/issues/163
- Bump `approx` dev dependency to match `geo-types`. This doesn't affect
  downstream users, only those building the proj crate for development.
  -  <https://github.com/georust/proj/pull/138>
- Changed license field to [SPDX 2.1 license expression](https://spdx.dev/spdx-specification-21-web-version/#h.jxpfx0ykyb60)
  -  <https://github.com/georust/proj/pull/146>
- Run clippy and apply fixes
  -  <https://github.com/georust/proj/pull/151>
- Update to geo-types 0.7.10
  -  <https://github.com/georust/proj/pull/153>
- Update MSRV to 1.63
  -  <https://github.com/georust/proj/pull/160>

## 0.27.0
- Inline the functionality of the legacy `Info` trait directly into `Proj`/`ProjBuilder` and remove the `Info` trait.
  - BREAKING: Getting information about the version of libproj installed was renamed from proj.info() to proj.lib_info()
  - Make `PjInfo` struct public, and rename it to `ProjInfo`
  - <https://github.com/georust/proj/pull/133>
- Actually return an error if a definition can't be retrieved
  - <https://github.com/georust/proj/pull/132>
- Update to PROJ 9.0.1 (proj-sys 0.23.1)
  - https://github.com/georust/proj/pull/135

## 0.26.0

- Update to proj 9
  - <https://github.com/georust/proj/pull/119>

## 0.25.2

- Introduce `Transform` trait, add implementations for `geo-types`
  - <https://github.com/georust/proj/pull/109>

## 0.25.1

- Fix intermittently wrong results due to memory initialization error.
  - <https://github.com/georust/proj/pull/104>

## 0.25.0

- Fix memory leak in network grid functionality
  - <https://github.com/georust/proj/pull/94>
- Mark mutable methods with `&mut`
  - <https://github.com/georust/proj/pull/102>
- Update `proj::Proj` constructors to return `Result` instead of `Option`
  - <https://github.com/georust/proj/pull/98>
- Add `TryFrom` impls for `proj::Proj`
  - <https://github.com/georust/proj/pull/100>
- Refactor `proj_create*` calls
  - <https://github.com/georust/proj/pull/103>

## 0.24.0
- update to proj-sys 0.21.0

## 0.23.1
- Update docs to refer to correct libproj version

## 0.23

- Update to PROJ 8.1.0 via proj-sys 0.20.0
- Add Debug impl for proj::Proj

## 0.22.1

- Update proj-sys to 0.19.1
  - https://github.com/georust/proj/blob/proj-sys-0.19.1/proj-sys/CHANGES.md

## 0.22.0
- Update PROJ to 7.2.1 via proj-sys 0.19.0

## 0.21.0

- geo-types integration is now optional, but enabled by default.  If you are
  not using the geo-types feature, instead of a `geo_types::Point`, you can
  project `(f64, f64)`, or anything conforming to the new `proj::Coord` trait.

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

