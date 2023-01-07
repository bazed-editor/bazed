fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let svelte_dir = format!("{manifest_dir}/svelte");

    println!("cargo:rerun-if-changed={manifest_dir}/build.rs");
    println!("cargo:rerun-if-changed={svelte_dir}/src");
    println!("cargo:rerun-if-changed={svelte_dir}/package.json");
    println!("cargo:rerun-if-changed={svelte_dir}/svelte.config.js");

    let status = std::process::Command::new("npm")
        .args(&["--prefix", &svelte_dir, "ci"])
        .status()
        .expect("npm ci failed");
    if !status.success() {
        panic!("`npm ci` had non-zero exit status");
    }

    let status = std::process::Command::new("npm")
        .args(&["run", "--prefix", &svelte_dir, "build"])
        .status()
        .expect("npm run build failed");
    if !status.success() {
        panic!("`npm run build` had non-zero exit status");
    }
    tauri_build::build()
}
