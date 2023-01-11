//! This module tests the project strurcture with external tools like cargo fmt or cargo deny.

use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio},
};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

#[test]
fn cargo_fmt_check() {
    if env::var_os("CI").is_some() {
        // we are checking formatting separately in CI
        return;
    }

    let output = Command::new("cargo")
        .args(["fmt", "--", "--check"])
        .current_dir(project_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    if !output.status.success() {
        Command::new("cargo")
            .arg("fmt")
            .current_dir(project_root())
            .output()
            .unwrap();
        panic!("code wasn't formatted");
    }
}

#[test]
fn cargo_deny_check() {
    if env::var_os("CI").is_some() {
        // we are already checking cargo deny via EmbarkStudios/cargo-deny-action@v1
        return;
    }

    let output = Command::new("cargo")
        .args(["deny", "check"])
        .current_dir(project_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[test]
fn cargo_doc_check() {
    if env::var_os("CI").is_some() {
        // we are checking docs separately in CI
        return;
    }

    let output = Command::new("cargo")
        .args(["doc", "--no-deps", "--document-private-items"])
        .current_dir(project_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    assert!(output.status.success());
}

#[test]
fn cargo_clippy_check() {
    if env::var_os("CI").is_some() {
        // we are checking clippy separately in CI
        return;
    }

    let output = Command::new("cargo")
        .args([
            "clippy",
            "--workspace",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .current_dir(project_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    assert!(output.status.success());
}
