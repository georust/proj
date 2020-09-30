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
