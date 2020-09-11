use bindgen;
use cmake;
use flate2::read::GzDecoder;
use std::fs::File;

use pkg_config;
use std::env;
use std::path::PathBuf;
use tar::Archive;

const MINIMUM_PROJ_VERSION: &str = "7.1.0";

#[cfg(feature = "nobuild")]
fn main() {} // Skip the build script on docs.rs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let include_path = if cfg!(feature = "bundled_proj") {
        println!("feature flags specified source build");
        build_from_source()?
    } else {
        pkg_config::Config::new()
        .atleast_version(MINIMUM_PROJ_VERSION)
        .probe("proj")
        .and_then(|pk| {
            println!("found acceptable libproj already installed at: {:?}", pk.link_paths[0]);
            if let Ok(val) = &env::var("_PROJ_SYS_TEST_EXPECT_BUILD_FROM_SRC") {
                if val != "0" {
                    panic!("for testing purposes: existing package was found, but should not have been");
                }
            }

            // Tell cargo to tell rustc to link the system proj
            // shared library.
            println!("cargo:rustc-link-search=native={:?}", pk.link_paths[0]);
            println!("cargo:rustc-link-lib=proj");

            Ok(pk.include_paths[0].clone())
        })
        .or_else(|err| {
            println!("pkg-config unable to find existing libproj installation: {}", err);
            build_from_source()
        })?
    };

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_path.to_string_lossy()))
        .trust_clang_mangling(false)
        .blacklist_type("max_align_t")
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

// returns the path of "inlude" for the built proj
fn build_from_source() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    println!("building libproj from source");
    if let Ok(val) = &env::var("_PROJ_SYS_TEST_EXPECT_BUILD_FROM_SRC") {
        if val == "0" {
            panic!(
                "for testing purposes: package was building from source but should not have been"
            );
        }
    }

    // NOTE: The PROJ build expects Sqlite3 to be present on the system.
    let path = "PROJSRC/proj-7.1.0.tar.gz";
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let tar_gz = File::open(path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack("PROJSRC/proj")?;
    let mut config = cmake::Config::new("PROJSRC/proj/proj-7.1.0");
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
    config.define("ENABLE_TIFF", "ON");
    let proj = config.build();
    // Tell cargo to tell rustc to link libproj, and where to find it
    // libproj will be built in $OUT_DIR/lib
    println!("cargo:rustc-link-lib=static=proj");
    println!(
        "cargo:rustc-link-search=native={}",
        proj.join("lib").display()
    );
    println!(
        "cargo:rustc-link-search={}",
        &out_path.join("lib64").display()
    );
    println!(
        "cargo:rustc-link-search={}",
        &out_path.join("build/lib").display()
    );
    // The PROJ library needs SQLite and the C++ standard library.
    println!("cargo:rustc-link-lib=dylib=sqlite3");
    println!("cargo:rustc-link-lib=dylib=tiff");
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        println!("cargo:warning=proj-sys: Not configuring an explicit C++ standard library on this target.");
    }

    Ok(proj.join("include"))
}
