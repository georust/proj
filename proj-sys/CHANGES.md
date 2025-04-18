# 0.26.0 - 2025-04-18

- Update to PROJ 9.6.0
- Update flate2 dependency for ureq 3.x compat
- Bump MSRV to 1.82

# 0.25.0 - 2024-12-20

- BREAKING: Update bindgen to 0.71.1


# 0.24.0

- Update to PROJ 9.4.0 (#196) 
- Bump MSRV to 1.70 (#188)


# 0.23.2

- Add explicit `tiff` feature for people using tiff files outside of `network`
  downloads. Tiff was already being enabled for used of the `network` feature.
      - <https://github.com/georust/proj/pull/143>
- Update to PROJ 9.2.1
- Update MSRV to 1.58.0
- Changed license field to [SPDX 2.1 license expression](https://spdx.dev/spdx-specification-21-web-version/#h.jxpfx0ykyb60)
  -  <https://github.com/georust/proj/pull/146>

# 0.23.1

- Update to PROJ 9.0.1

# 0.23.0

- Update to PROJ 9.0.0

# 0.22.0

- Only require tiff for source builds when network feature is enabled
    - <https://github.com/georust/proj/pull/95/commits/b0f447446d07cadc2da86d6be3d37eb35c3620d6>

# 0.21.0

- BREAKING: Remove `bundled_proj_tiff` feature and assume system libproj has
  the default enabled tiff support. Otherwise, the current setup would
  unnecessarily build libproj from source in some cases, e.g. the geo crate's
  proj network integration would compile libproj from source.
    - <https://github.com/georust/proj/pull/92>

# 0.20.1
- Fix docs to refer to correct libproj version

# 0.20.0
- Update to PROJ 8.1.0

# 0.19.1

- Upgrade bindgen to fix incorrect results on aarch64 (e.g. Apple's M1)
    - <https://github.com/georust/proj/pull/80>
- Allow proj-sys to link to `proj_d.lib`. This prevents a linker error when building in debug mode with MSVC
    - <https://github.com/georust/proj/pull/83>

# 0.19.0
- Update to PROJ 7.2.1

# 0.18.4
- add `bundled_proj_tiff` feature

# 0.18.3
- Unify repo with proj repo
- Switch to GH actions

# 0.18.2
- Add inline docs

# 0.18.1
- Expand link-search paths for bundled_proj feature in build.rs

# 0.18.0
- The `bundled_proj` feature statically links libproj and disables native network functionality

# 0.17.0
- Update to PROJ 7.1.0

# 0.16.4
- Enabled bundled PROJ for macOS target

# 0.16.0
- Enable `pkg_config` option for Linux targets
- `pkg_config` is now optional on macOS

# 0.15.0
- Add pkgconfig to macOS build script for more robust PROJ library resolution

# 0.13.0
- Add bundled Linux build option

# 0.12.0
- Update to PROJ 7.0.0

# 0.11.1
- Fixed link to function references

# 0.11.0
- Bumped minimum PROJ version to 6.2.0
- Updated to 2018 edition

# 0.9.0
- Bumped minimum PROJ.4 version to 6.0.0
- Updated to 2018 edition

# 0.8.0
- Bumped minimum PROJ.4 version to 5.2.0

# 0.7.0
- Bumped minimum PROJ.4 version to 5.1.0
- Removed the `pj_strerrno` method, now that proj_errno_string exists
