use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

fn all_files_with_extension<P: AsRef<Path>>(
    path: P,
    extension: &str,
) -> impl Iterator<Item = PathBuf> + '_ {
    WalkDir::new(path).into_iter().filter_map(move |entry| {
        entry
            .ok()
            .map(|entry| entry.into_path())
            .filter(|path| path.extension() == Some(&OsString::from(extension)))
    })
}

fn main() {
    println!("cargo:rerun-if-changed=src/ffi.cpp");

    let rive_cpp_path = env::var("RIVE_CPP_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../submodules/rive-cpp"));

    cc::Build::new()
        .cpp(true)
        .include(rive_cpp_path.join("include"))
        .file("src/ffi.cpp")
        .flag("-std=c++14")
        .warnings(false)
        .compile("rive-ffi");

    if cfg!(feature = "layout") {
        cc::Build::new()
            .cpp(true)
            .flag("-std=c++11")
            .files(all_files_with_extension("../submodules/yoga/yoga", "cpp"))
            .include("../submodules/yoga")
            .define("YOGA_EXPORT=", None)
            .warnings(false)
            .compile("yoga");
    }

    if cfg!(feature = "text") {
        let target = env::var("TARGET").unwrap();
        let profile = env::var("PROFILE").unwrap();

        let mut cfg = cc::Build::new();
        cfg.cpp(true)
            .flag_if_supported("-std=c++11") // for unix
            .warnings(false)
            .file("../submodules/harfbuzz/src/harfbuzz.cc");

        if !target.contains("windows") {
            cfg.define("HAVE_PTHREAD", "1");
        }

        if target.contains("apple") && profile.contains("release") {
            cfg.define("HAVE_CORETEXT", "1");
        }

        if target.contains("windows") {
            cfg.define("HAVE_DIRECTWRITE", "1");
        }

        if target.contains("windows-gnu") {
            cfg.flag("-Wa,-mbig-obj");
        }

        cfg.compile("harfbuzz");

        cc::Build::new()
            .files(all_files_with_extension(
                "../submodules/SheenBidi/Source",
                "c",
            ))
            .include("../submodules/SheenBidi/Headers")
            .warnings(false)
            .compile("sheenbidi");
    }

    let mut cfg = cc::Build::new();
    cfg.cpp(true)
        .include(rive_cpp_path.join("include"))
        .files(all_files_with_extension(rive_cpp_path.join("src"), "cpp"))
        .flag("-std=c++14")
        .define("_RIVE_INTERNAL_", None)
        .warnings(false);

    if cfg!(feature = "text") {
        cfg.include("../submodules/harfbuzz/src")
            .include("../submodules/SheenBidi/Headers")
            .flag_if_supported("-Wno-deprecated-declarations")
            .define("WITH_RIVE_TEXT", None);
    }
    if cfg!(feature = "layout") {
        cfg.include("../submodules/yoga")
            .flag_if_supported("-Wno-deprecated-declarations")
            .define("WITH_RIVE_LAYOUT", None)
            .define("YOGA_EXPORT=", None);
    }

    cfg.compile("rive");
}
