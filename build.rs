use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let home = env::var("HOME").expect("HOME not set");
    let lib_dir = PathBuf::from(&home).join(".local/lib");

    println!("cargo:rerun-if-changed=build.rs");

    // Set rpath so the binary finds libpdfium.dylib at runtime.
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        lib_dir.display()
    );

    // Copy libpdfium.dylib from the pdfium-bind-sys build output to ~/.local/lib/.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // OUT_DIR: target/<profile>/build/pdfyank-<hash>/out
    // out -> pdfyank-<hash> -> build
    let build_dir = out_dir.parent().and_then(|p| p.parent());

    if let Some(build_dir) = build_dir {
        for entry in fs::read_dir(build_dir).into_iter().flatten().flatten() {
            let name = entry.file_name();
            if !name.to_string_lossy().starts_with("pdfium-bind-sys-") {
                continue;
            }
            let dylib = entry.path().join("out/pdfium/lib/libpdfium.dylib");
            if dylib.exists() {
                fs::create_dir_all(&lib_dir).expect("failed to create ~/.local/lib");
                let dest = lib_dir.join("libpdfium.dylib");
                fs::copy(&dylib, &dest).expect("failed to copy libpdfium.dylib");
                println!(
                    "cargo:warning=Installed libpdfium.dylib to {}",
                    dest.display()
                );
                break;
            }
        }
    }
}
