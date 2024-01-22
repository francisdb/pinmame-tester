use std::env;
use std::path::{Path, PathBuf};

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let pinmame_dir = Path::new(&dir)
        .join("pinmame")
        .join("build")
        .join("Release");
    println!("cargo:rustc-link-search=native={}", pinmame_dir.display());

    // TODO get rid of this part and have libpinmame compile as a static library on linux
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=pinmame");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}/", pinmame_dir.display());
    } else {
        println!("cargo:rustc-link-lib=static=pinmame");
    }

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg("-x")
        .clang_arg("c++")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
