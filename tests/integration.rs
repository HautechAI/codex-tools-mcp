use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn prints_version() {
    let mut cmd = Command::cargo_bin("codex-tools-mcp").expect("binary exists");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn applies_patch_and_creates_file() {
    let dir = tempdir().expect("create temp dir");

    let mut cmd = Command::cargo_bin("codex-tools-mcp").expect("binary exists");
    cmd.arg("--log-level").arg("error");
    cmd.current_dir(dir.path());

    let input = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"test","version":"0"},"capabilities":{}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"apply_patch","arguments":{"input":"*** Begin Patch\n*** Add File: hello.txt\n+hello world!\n*** End Patch\n"}}}
"#;
    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn server");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        use std::io::Write;
        stdin.write_all(input.as_bytes()).expect("write stdin");
    }

    let output = child.wait_with_output().expect("collect output");
    assert!(
        output.status.success(),
        "process exited with failure: {:?}",
        output
    );

    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("\"result\""),
        "expected responses in stdout: {stdout}"
    );

    let hello_path = dir.path().join("hello.txt");
    let contents = std::fs::read_to_string(&hello_path).expect("hello.txt created");
    assert_eq!(contents.trim(), "hello world!");
}
