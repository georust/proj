use flate2::read::GzDecoder;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use tar::Archive;

const MINIMUM_PROJ_VERSION: &str = "9.4.0";

#[cfg(feature = "nobuild")]
fn main() {} // Skip the build script on docs.rs

#[cfg(not(feature = "nobuild"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let include_path = if cfg!(feature = "bundled_proj") {
        eprintln!("feature flags specified source build");
        build_from_source()?
    } else {
        pkg_config::Config::new()
        .atleast_version(MINIMUM_PROJ_VERSION)
        .probe("proj")
        .map(|pk| {
            eprintln!("found acceptable libproj already installed at: {:?}", pk.link_paths[0]);
            if cfg!(feature = "network") {
                // Generally, system proj installations have been built with tiff support
                // allowing for network grid interaction. If this proves to be untrue
                // could we try to determine some kind of runtime check and fall back
                // to building from source?
                eprintln!("assuming existing system libproj installation has network (tiff) support");
            }
            if let Ok(val) = &env::var("_PROJ_SYS_TEST_EXPECT_BUILD_FROM_SRC") {
                if val != "0" {
                    panic!("for testing purposes: existing package was found, but should not have been");
                }
            }

            // Tell cargo to tell rustc to link the system proj
            // shared library.
            println!("cargo:rustc-link-search=native={:?}", pk.link_paths[0]);
            println!("cargo:rustc-link-lib=proj");

            pk.include_paths[0].clone()
        })
        .or_else(|err| {
            eprintln!("pkg-config unable to find existing libproj installation: {err}");
            build_from_source()
        })?
    };

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_path.to_string_lossy()))
        .trust_clang_mangling(false)
        .size_t_is_usize(true)
        .blocklist_type("max_align_t")
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}

// returns the path of "include" for the built proj
fn build_from_source() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    eprintln!("building libproj from source");
    println!("cargo:rustc-cfg=bundled_build");
    if let Ok(val) = &env::var("_PROJ_SYS_TEST_EXPECT_BUILD_FROM_SRC") {
        if val == "0" {
            panic!(
                "for testing purposes: package was building from source but should not have been"
            );
        }
    }

    let path = "PROJSRC/proj-9.4.0.tar.gz";
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let tar_gz = File::open(path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(out_path.join("PROJSRC/proj"))?;
    let mut config = cmake::Config::new(out_path.join("PROJSRC/proj/proj-9.4.0"));
    config.define("BUILD_SHARED_LIBS", "OFF");
    config.define("BUILD_TESTING", "OFF");
    config.define("BUILD_CCT", "OFF");
    config.define("BUILD_CS2CS", "OFF");
    config.define("BUILD_GEOD", "OFF");
    config.define("BUILD_GIE", "OFF");
    config.define("BUILD_PROJ", "OFF");
    config.define("BUILD_PROJINFO", "OFF");
    config.define("BUILD_PROJSYNC", "OFF");
    config.define("ENABLE_CURL", "OFF");

    // we check here whether or not these variables are set by cargo
    // if they are set, `libsqlite3-sys` was built with the bundled feature
    // enabled, which in turn allows us to rely on the built libsqlite3 version
    // and link it statically
    //
    // If these are not set, it's necessary that libsqlite3 exists on the build system
    // in a location accessible by cmake
    if let Ok(sqlite_include) = std::env::var("DEP_SQLITE3_INCLUDE") {
        config.define("SQLITE3_INCLUDE_DIR", sqlite_include);
    }
    if let Ok(sqlite_lib_dir) = std::env::var("DEP_SQLITE3_LIB_DIR") {
        config.define("SQLITE3_LIBRARY", format!("{sqlite_lib_dir}/libsqlite3.a",));
    }

    if cfg!(feature = "tiff") {
        eprintln!("enabling tiff support");
        config.define("ENABLE_TIFF", "ON");
    } else {
        eprintln!("disabling tiff support");
        config.define("ENABLE_TIFF", "OFF");
    }

    let proj = config.build();
    // Tell cargo to tell rustc to link libproj, and where to find it
    // libproj will be built in $OUT_DIR/lib

    //proj likes to create proj_d when configured as debug and on MSVC, so link to that one if it exists
    if proj.join("lib").join("proj_d.lib").exists() {
        println!("cargo:rustc-link-lib=static=proj_d");
    } else {
        println!("cargo:rustc-link-lib=static=proj");
    }
    println!(
        "cargo:rustc-link-search=native={}",
        proj.join("lib").display()
    );

    // This is producing a warning - this directory doesn't exist (on aarch64 anyway)
    println!(
        "cargo:rustc-link-search={}",
        &out_path.join("lib64").display()
    );
    println!(
        "cargo:rustc-link-search={}",
        &out_path.join("build/lib").display()
    );

    if cfg!(feature = "tiff") {
        // On platforms like apples aarch64, users are likely to have installed libtiff with homebrew,
        // which isn't in the default search path, so try to determine path from pkg-config
        match pkg_config::Config::new()
            .atleast_version("4.0")
            .probe("libtiff-4")
        {
            Ok(pk) => {
                eprintln!(
                    "found acceptable libtiff installed at: {:?}",
                    pk.link_paths[0]
                );
                println!("cargo:rustc-link-search=native={:?}", pk.link_paths[0]);
            }
            Err(err) => {
                // pkg-config might not even be installed. Let's try to stumble forward
                // to see if the build succeeds regardless, e.g. if libtiff is installed
                // in some default search path.
                eprintln!("Failed to find libtiff with pkg-config: {err}");
            }
        }
        println!("cargo:rustc-link-lib=dylib=tiff");
    }

    Ok(proj.join("include"))
}
