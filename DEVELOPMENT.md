# Development related notices


Bindings were generated with the following command:

```sh
bindgen --blocklist-type max_align_t wrapper.h -- -I  PROJSRC/proj-9.6.0/src
```

If you update the above command line you also need to update the arguments for the buildtime_bindgen feature in `build.rs`
