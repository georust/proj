use flate2::read::GzDecoder;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use tar::Archive;

const MINIMUM_PROJ_VERSION: &str = "9.6.2";

#[cfg(feature = "nobuild")]
fn main() {} // Skip the build script on docs.rs

#[cfg(not(feature = "nobuild"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let include_path = if cfg!(feature = "bundled_proj") {
        eprintln!("feature flags specified source build");
        build_from_source()?
    } else if env::var("CARGO_CFG_TARGET_OS")? == "emscripten" {
        eprintln!("no system libproj exists for emscripten: building from source");
        build_from_source()?
    } else {
        pkg_config::Config::new()
        .atleast_version(MINIMUM_PROJ_VERSION)
        .probe("proj")
        .map(|pk| {
            eprintln!("found acceptable libproj already installed at: {:?}", pk.link_paths);
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

            // pkg-config prints the link directives for the system libproj.
            pk.include_paths[0].clone()
        })
        .or_else(|err| {
            eprintln!("pkg-config unable to find existing libproj installation: {err}");
            build_from_source()
        })?
    };

    #[cfg(feature = "buildtime_bindgen")]
    generate_bindings(include_path)?;
    #[cfg(not(feature = "buildtime_bindgen"))]
    let _ = include_path;

    Ok(())
}

#[cfg(feature = "buildtime_bindgen")]
fn generate_bindings(include_path: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    // If you update the configuration here you also
    // need to update the corresponding bindgen command in
    // `DEVELOPMENT.md`
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_path.to_string_lossy()))
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

    // A build script is compiled for the host, so cfg! gives the host
    // configuration. Cargo sets these variables for the target.
    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    let target_env = env::var("CARGO_CFG_TARGET_ENV")?;

    let path = format!("PROJSRC/proj-{MINIMUM_PROJ_VERSION}.tar.gz");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let tar_gz = File::open(path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(out_path.join("PROJSRC/proj"))?;
    let mut config =
        cmake::Config::new(out_path.join(format!("PROJSRC/proj/proj-{MINIMUM_PROJ_VERSION}")));
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
        // The cc crate gives the static library an MSVC or a Unix-style name.
        let sqlite_lib = if target_env == "msvc" {
            "sqlite3.lib"
        } else {
            "libsqlite3.a"
        };
        config.define("SQLITE3_LIBRARY", format!("{sqlite_lib_dir}/{sqlite_lib}"));
    }

    if cfg!(feature = "tiff") {
        eprintln!("enabling tiff support");
        config.define("ENABLE_TIFF", "ON");
    } else {
        eprintln!("disabling tiff support");
        config.define("ENABLE_TIFF", "OFF");
    }

    if target_os == "emscripten" {
        // The libsqlite3-sys build supplies the only wasm32 SQLite.
        if env::var("DEP_SQLITE3_LIB_DIR").is_err() {
            return Err("the emscripten target requires the bundled_proj_sqlite feature".into());
        }
        // No libtiff exists for this target, and the network feature
        // uses sockets.
        if cfg!(feature = "tiff") {
            return Err(
                "the network and tiff features do not support the emscripten target".into(),
            );
        }
        // PROJ uses C++ exceptions, and all objects must use the same
        // exception model as the Rust code. From Rust 1.93, the emscripten
        // target uses wasm exceptions. Before 1.93, it uses JS exceptions.
        // The cc defaults add -fno-exceptions for all wasm targets, and the
        // emscripten toolchain file supplies the other flags.
        let exceptions = if rustc_minor_version()? >= 93 {
            "-fwasm-exceptions"
        } else {
            "-fexceptions"
        };
        config.no_default_flags(true);
        config.cflag(exceptions);
        config.cxxflag(exceptions);
        // emcmake adds the toolchain file, but cmake-rs cannot see it and
        // also sets CMAKE_<LANG>_COMPILER. The toolchain file sets a different
        // compiler path, and the next configure then fails. With a defined
        // toolchain file, cmake-rs does not set the compilers.
        if emscripten_toolchain_file_from_env().is_none() {
            config.define("CMAKE_TOOLCHAIN_FILE", emscripten_toolchain_file()?);
        }
        // There is no file system for proj.db at runtime.
        config.define("EMBED_RESOURCE_FILES", "ON");
        config.define("USE_ONLY_EMBEDDED_RESOURCE_FILES", "ON");
    }

    if target_env == "msvc" {
        // rust links the release MVSC runtime
        // also for debug builds. If we let
        // cmake choose debug/release builds
        // based on the underlying cargo build
        // version that results in linker errors
        config.profile("Release");
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
    // The static libproj does not carry its link dependencies. On Windows,
    // filemanager.cpp uses shell32 (SHGetFolderPathW, SHGetKnownFolderPath)
    // and ole32 (CoTaskMemFree).
    if target_os == "windows" {
        println!("cargo:rustc-link-lib=dylib=shell32");
        println!("cargo:rustc-link-lib=dylib=ole32");
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
                // pkg-config prints the link search path for libtiff.
                eprintln!("found acceptable libtiff installed at: {:?}", pk.link_paths);
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

// Returns the minor version of the rustc that compiles the target code.
#[cfg(not(feature = "nobuild"))]
fn rustc_minor_version() -> Result<u32, Box<dyn std::error::Error>> {
    let rustc = env::var("RUSTC")?;
    let output = std::process::Command::new(rustc)
        .arg("--version")
        .output()?;
    let version = String::from_utf8(output.stdout)?;
    // The output has the form "rustc 1.93.0 (... )".
    version
        .split_whitespace()
        .nth(1)
        .and_then(|v| v.split('.').nth(1))
        .and_then(|minor| minor.parse().ok())
        .ok_or_else(|| format!("cannot parse the rustc version: {version}").into())
}

// Returns a toolchain file from the environment variables that cmake-rs reads.
#[cfg(not(feature = "nobuild"))]
fn emscripten_toolchain_file_from_env() -> Option<String> {
    let target = env::var("TARGET").ok()?;
    [
        format!("CMAKE_TOOLCHAIN_FILE_{target}"),
        format!("CMAKE_TOOLCHAIN_FILE_{}", target.replace('-', "_")),
        "TARGET_CMAKE_TOOLCHAIN_FILE".to_string(),
        "CMAKE_TOOLCHAIN_FILE".to_string(),
    ]
    .iter()
    .find_map(|var| env::var(var).ok())
}

// Returns the CMake toolchain file of the emscripten installation that em-config finds.
#[cfg(not(feature = "nobuild"))]
fn emscripten_toolchain_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("em-config")
        .arg("EMSCRIPTEN_ROOT")
        .output()
        .map_err(|err| format!("cannot run em-config: {err}"))?;
    if !output.status.success() {
        return Err("em-config cannot find the emscripten root".into());
    }
    let root = String::from_utf8(output.stdout)?;
    Ok(PathBuf::from(root.trim()).join("cmake/Modules/Platform/Emscripten.cmake"))
}
