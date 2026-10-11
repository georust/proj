# Development related notices


Bindings were generated with bindgen 0.71.1, from the `proj-sys` directory, after
unpacking `PROJSRC/proj-9.6.2.tar.gz` into `PROJSRC/proj`:

```sh
bindgen --blocklist-type max_align_t wrapper.h -- -I PROJSRC/proj/proj-9.6.2/src > src/bundled_bindings.rs
```

If you update the above command line you also need to update the arguments for the buildtime_bindgen feature in `build.rs`
