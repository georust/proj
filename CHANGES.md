# Changes

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

