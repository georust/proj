# Notes for maintainers

## Generating docs from bindings

Whenever updating libproj or bindgen you must regenerate the prebuilt bindings
at `proj-sys/src/bindings_docs-rs.rs`.

These prebuilt bindings are only used for generating documentation - e.g. on
https://docs.rs. Actual usage of the crate depends on dynamically built bindings, but
that entails having libproj installed or built from source, which we can't
expect docs.rs to do.

## To update the prebuilt bindings

Currently, the process looks like:

```
cd proj-sys
cargo clean
cargo build
find ../target/proj-sys-* -name bindings_docs-rs.rs
```

copy that file over the `src/bindings_docs-rs.rs`, but retain the header:

```
/* THESE ARE NOT LIVE BINDINGS */
/* THEY EXIST FOR USE BY DOCS-RS ONLY */
```
