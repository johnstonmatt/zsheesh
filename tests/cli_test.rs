use std::io::Write;
use std::process::Command;

fn zsheesh_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_zsheesh"))
}

#[test]
fn stdin_stdout_formatting() {
    let mut child = zsheesh_bin()
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn zsheesh");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"if [ -f x ]; then\necho y\nfi\n")
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("  echo y"), "Should indent: got {stdout}");
}

#[test]
fn check_mode_exits_zero_when_formatted() {
    let mut child = zsheesh_bin()
        .arg("--check")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn zsheesh");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"echo hello\n")
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "Already-formatted input should pass --check"
    );
}

#[test]
fn check_mode_exits_nonzero_when_unformatted() {
    let mut child = zsheesh_bin()
        .arg("--check")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn zsheesh");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"if [ -f x ]; then\necho y\nfi\n")
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert!(
        !output.status.success(),
        "Unformatted input should fail --check"
    );
}

#[test]
fn file_argument_formatting() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.zsh");
    std::fs::write(&file, "if [ -f x ]; then\necho y\nfi\n").unwrap();

    let output = zsheesh_bin()
        .arg(file.to_str().unwrap())
        .output()
        .expect("failed to spawn zsheesh");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("  echo y"),
        "Should format file: got {stdout}"
    );
}

#[test]
fn write_mode_modifies_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.zsh");
    std::fs::write(&file, "if [ -f x ]; then\necho y\nfi\n").unwrap();

    let output = zsheesh_bin()
        .arg("-w")
        .arg(file.to_str().unwrap())
        .output()
        .expect("failed to spawn zsheesh");

    assert!(output.status.success());
    let content = std::fs::read_to_string(&file).unwrap();
    assert!(
        content.contains("  echo y"),
        "File should be modified in-place: got {content}"
    );
}

#[test]
fn check_mode_with_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.zsh");
    std::fs::write(&file, "if [ -f x ]; then\necho y\nfi\n").unwrap();

    let output = zsheesh_bin()
        .arg("--check")
        .arg(file.to_str().unwrap())
        .output()
        .expect("failed to spawn zsheesh");

    assert!(!output.status.success(), "Unformatted file should fail");

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("would reformat"),
        "Should report file: got {stderr}"
    );
}

#[test]
fn dump_ast_mode() {
    let mut child = zsheesh_bin()
        .arg("--dump-ast")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn zsheesh");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"echo hello\n")
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("command") || stdout.contains("program"),
        "AST dump should contain node types"
    );
}

#[test]
fn custom_indent_via_cli() {
    let mut child = zsheesh_bin()
        .arg("--indent")
        .arg("\t")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn zsheesh");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"if [ -f x ]; then\necho y\nfi\n")
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("\techo"),
        "Should indent with tab: got {stdout}"
    );
}

#[test]
fn version_flag() {
    let output = zsheesh_bin()
        .arg("--version")
        .output()
        .expect("failed to spawn zsheesh");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("zsheesh"));
}

#[test]
fn help_flag() {
    let output = zsheesh_bin()
        .arg("--help")
        .output()
        .expect("failed to spawn zsheesh");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("zsheesh"));
}

#[test]
fn multiple_files() {
    let dir = tempfile::tempdir().unwrap();
    let f1 = dir.path().join("a.zsh");
    let f2 = dir.path().join("b.zsh");
    std::fs::write(&f1, "echo a\n").unwrap();
    std::fs::write(&f2, "echo b\n").unwrap();

    let output = zsheesh_bin()
        .arg("-w")
        .arg(f1.to_str().unwrap())
        .arg(f2.to_str().unwrap())
        .output()
        .expect("failed to spawn zsheesh");

    assert!(output.status.success());
}
