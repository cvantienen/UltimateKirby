//! The optional ImGui client needs the matching upstream static API shim.
//! GUI builds use the pinned shim in lib/ (or IMGUI_SMASH_LIB_DIR).
//! A headless --no-default-features build does not link the external GUI.
fn main() {
    println!("cargo:rerun-if-env-changed=IMGUI_SMASH_LIB_DIR");
    if std::env::var_os("CARGO_FEATURE_IMGUI_MENU").is_none() {
        return;
    }
    let directory = std::env::var_os("IMGUI_SMASH_LIB_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("lib"));
    let library = directory.join("libimgui_smash.a");
    println!("cargo:rerun-if-changed={}", library.display());
    assert!(library.is_file(), "imgui-menu requires the matching imgui-smash libimgui_smash.a; set IMGUI_SMASH_LIB_DIR. See docs/CUSTOM_COMBOS.md");
    println!("cargo:rustc-link-search=native={}", directory.display());
    println!("cargo:rustc-link-lib=static=imgui_smash");
}
