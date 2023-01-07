fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let npm_command = std::env::var("CARGO_NPM_COMMAND").unwrap_or("npm".to_string());
    let npm_workspace_root = format!("{manifest_dir}/../..");
    let svelte_dir = format!("{npm_workspace_root}/node_packages/bazed-svelte");

    println!("cargo:rerun-if-changed={manifest_dir}/build.rs");
    println!("cargo:rerun-if-changed={svelte_dir}/src");
    println!("cargo:rerun-if-changed={svelte_dir}/package.json");
    println!("cargo:rerun-if-changed={svelte_dir}/svelte.config.js");

    let status = std::process::Command::new(&npm_command)
        .args(&["--prefix", &npm_workspace_root, "ci", "-w", &svelte_dir])
        .status()
        .expect("npm ci failed");

    if !status.success() {
        panic!("`npm ci` had non-zero exit status");
    }

    let status = std::process::Command::new(&npm_command)
        .args(&[
            "run",
            "--prefix",
            &npm_workspace_root,
            "build",
            "-w",
            &svelte_dir,
        ])
        .status()
        .expect("npm run build failed");

    if !status.success() {
        panic!("`npm run build` had non-zero exit status");
    }

    tauri_build::build()
}
