use std::io::Write;
use std::process::Command;

fn zsheesh() -> Command {
    Command::new(env!("CARGO_BIN_EXE_zsheesh"))
}

fn run_stdin(input: &str) -> (String, String, bool) {
    let mut child = zsheesh()
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn zsheesh");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
        output.status.success(),
    )
}

fn run_stdin_check(input: &str) -> bool {
    let mut child = zsheesh()
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
        .write_all(input.as_bytes())
        .unwrap();
    drop(child.stdin.take());

    child.wait_with_output().unwrap().status.success()
}

// ---------------------------------------------------------------------------
// E2E: Full pipeline — unformatted input → formatted output → idempotent
// ---------------------------------------------------------------------------

#[test]
fn e2e_full_pipeline_simple_script() {
    let unformatted = r#"#!/bin/zsh
greet(){
echo "Hello, $1"
if [ -z "$1" ]; then
echo "No name given"
else
echo "Welcome"
fi
}
greet "world"
"#;

    // Step 1: Format
    let (formatted, _, ok) = run_stdin(unformatted);
    assert!(ok, "Formatting should succeed");

    // Step 2: Verify structure
    assert!(formatted.contains("greet()"), "Function parens normalized");
    assert!(formatted.contains("  echo"), "Body indented");
    assert!(
        formatted.contains("  if"),
        "Nested if indented: {formatted}"
    );

    // Step 3: Idempotence — formatting the output again should be identical
    let (second_pass, _, ok2) = run_stdin(&formatted);
    assert!(ok2, "Second pass should succeed");
    assert_eq!(
        formatted, second_pass,
        "Formatter must be idempotent (fixed point)"
    );

    // Step 4: --check should pass on already-formatted output
    assert!(
        run_stdin_check(&formatted),
        "--check should pass on formatted output"
    );

    // Step 5: --check should fail on unformatted input
    assert!(
        !run_stdin_check(unformatted),
        "--check should fail on unformatted input"
    );
}

#[test]
fn e2e_full_pipeline_complex_script() {
    let unformatted = r#"#!/bin/zsh
HISTFILE=~/.zsh_history
HISTSIZE=10000
SAVEHIST=10000
setopt SHARE_HISTORY
setopt HIST_IGNORE_DUPS

alias ll='ls -la'
alias la='ls -A'

mkcd(){
mkdir -p "$1"&&cd "$1"
}

extract(){
if [ -f "$1" ]; then
case "$1" in
*.tar.gz)
tar xzf "$1"
;;
*.zip)
unzip "$1"
;;
*.gz)
gunzip "$1"
;;
*)
echo "'$1' cannot be extracted"
;;
esac
else
echo "'$1' is not a valid file"
fi
}

export PATH="$HOME/.local/bin:$PATH"

if [ -f ~/.zshrc.local ]; then
source ~/.zshrc.local
fi
"#;

    let (formatted, _, ok) = run_stdin(unformatted);
    assert!(ok, "Formatting should succeed");

    // Verify indentation
    assert!(
        formatted.contains("  mkdir"),
        "Function body indented: {formatted}"
    );
    assert!(
        formatted.contains("    case"),
        "Case inside if indented: {formatted}"
    );

    // Idempotence
    let (second_pass, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second_pass, "Must be idempotent");

    // Check mode
    assert!(run_stdin_check(&formatted));
    assert!(!run_stdin_check(unformatted));
}

// ---------------------------------------------------------------------------
// E2E: File-based workflow (format → write → check)
// ---------------------------------------------------------------------------

#[test]
fn e2e_file_workflow() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("script.zsh");

    let unformatted = "#!/bin/zsh\nif [ -f /tmp/x ]; then\necho found\nfi\n";
    std::fs::write(&file, unformatted).unwrap();

    // Step 1: --check fails on unformatted file
    let output = zsheesh()
        .arg("--check")
        .arg(file.to_str().unwrap())
        .output()
        .unwrap();
    assert!(!output.status.success(), "--check should fail");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("script.zsh"));

    // Step 2: Format in-place with -w
    let output = zsheesh()
        .arg("-w")
        .arg(file.to_str().unwrap())
        .output()
        .unwrap();
    assert!(output.status.success(), "-w should succeed");

    // Step 3: --check now passes
    let output = zsheesh()
        .arg("--check")
        .arg(file.to_str().unwrap())
        .output()
        .unwrap();
    assert!(output.status.success(), "--check should pass after -w");

    // Step 4: File content is correctly formatted
    let content = std::fs::read_to_string(&file).unwrap();
    assert!(content.contains("  echo found"), "Body should be indented");
}

#[test]
fn e2e_multi_file_workflow() {
    let dir = tempfile::tempdir().unwrap();

    let files: Vec<_> = (0..5)
        .map(|i| {
            let p = dir.path().join(format!("script_{i}.zsh"));
            std::fs::write(&p, format!("if [ -f /tmp/{i} ]; then\necho {i}\nfi\n")).unwrap();
            p
        })
        .collect();

    // Step 1: --check fails on all files
    let mut cmd = zsheesh();
    cmd.arg("--check");
    for f in &files {
        cmd.arg(f.to_str().unwrap());
    }
    let output = cmd.output().unwrap();
    assert!(!output.status.success());

    // Step 2: Format all in-place
    let mut cmd = zsheesh();
    cmd.arg("-w");
    for f in &files {
        cmd.arg(f.to_str().unwrap());
    }
    let output = cmd.output().unwrap();
    assert!(output.status.success());

    // Step 3: --check passes on all files
    let mut cmd = zsheesh();
    cmd.arg("--check");
    for f in &files {
        cmd.arg(f.to_str().unwrap());
    }
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "--check should pass after -w on all files"
    );
}

// ---------------------------------------------------------------------------
// E2E: fmt: skip / off / on through the full pipeline
// ---------------------------------------------------------------------------

#[test]
fn e2e_fmt_skip_end_to_end() {
    let input = r#"#!/bin/zsh
echo "normal"
# fmt: skip
echo    "this    is   preserved    exactly"
echo "also normal"
"#;

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    // The skipped line must be verbatim
    assert!(
        formatted.contains("echo    \"this    is   preserved    exactly\""),
        "fmt: skip line must be preserved verbatim: {formatted}"
    );

    // Idempotence
    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second, "Must be idempotent with fmt: skip");
}

#[test]
fn e2e_fmt_off_on_end_to_end() {
    let input = r#"#!/bin/zsh
echo "before"
# fmt: off
if   [   -f  /tmp/x   ];   then
echo    "weird    spacing"
fi
# fmt: on
echo "after"
"#;

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    // The off region must be preserved
    assert!(
        formatted.contains("if   [   -f  /tmp/x   ];   then"),
        "fmt: off region must be preserved: {formatted}"
    );
    assert!(
        formatted.contains("echo    \"weird    spacing\""),
        "fmt: off region must be preserved: {formatted}"
    );

    // Idempotence
    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second, "Must be idempotent with fmt: off/on");
}

#[test]
fn e2e_fmt_off_without_on() {
    let input = "echo \"before\"\n# fmt: off\necho    \"preserved\"\necho    \"also preserved\"\n";

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    assert!(formatted.contains("echo    \"preserved\""));
    assert!(formatted.contains("echo    \"also preserved\""));
}

// ---------------------------------------------------------------------------
// E2E: Specific zsh constructs
// ---------------------------------------------------------------------------

#[test]
fn e2e_nested_functions() {
    let input = r#"outer(){
inner(){
echo "deep"
}
inner
echo "outer"
}
"#;
    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);
    assert!(formatted.contains("  inner()"), "Inner function indented");
    assert!(
        formatted.contains("    echo \"deep\""),
        "Deeply nested body: {formatted}"
    );
    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second);
}

#[test]
fn e2e_case_with_multiple_commands() {
    let input = r#"case "$1" in
start)
echo "starting"
start_service
log "started"
;;
stop)
echo "stopping"
stop_service
;;
*)
echo "Usage: $0 {start|stop}"
exit 1
;;
esac
"#;

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    // Case body should be double-indented
    assert!(
        formatted.contains("    echo \"starting\""),
        "Case body indented: {formatted}"
    );
    assert!(
        formatted.contains("    start_service"),
        "Multi-command case: {formatted}"
    );

    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second);
}

#[test]
fn e2e_pipeline_formatting() {
    let input = "cat /etc/passwd|grep root|awk '{print $1}'|sort|uniq\n";

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    // Each pipe should have surrounding spaces
    assert!(
        formatted.contains(" | "),
        "Pipes should be spaced: {formatted}"
    );

    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second);
}

#[test]
fn e2e_command_lists() {
    let input = "mkdir -p /tmp/test&&cd /tmp/test&&echo success||echo fail\n";

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    assert!(
        formatted.contains(" && "),
        "&& should be spaced: {formatted}"
    );
    assert!(
        formatted.contains(" || "),
        "|| should be spaced: {formatted}"
    );

    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second);
}

#[test]
fn e2e_heredoc_preservation() {
    let input = r#"cat <<EOF
Hello, $USER
This is a heredoc with   weird   spacing
  and indentation preserved
EOF
"#;

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    // Heredoc body must be preserved exactly
    assert!(
        formatted.contains("This is a heredoc with   weird   spacing"),
        "Heredoc body must be preserved: {formatted}"
    );
    assert!(
        formatted.contains("  and indentation preserved"),
        "Heredoc indent must be preserved: {formatted}"
    );

    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second);
}

#[test]
fn e2e_while_loop_with_read() {
    let input = "while read -r line; do\nprocess \"$line\"\nlog \"$line\"\ndone < input.txt\n";

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    assert!(
        formatted.contains("  process"),
        "While body indented: {formatted}"
    );
    assert!(
        formatted.contains("  log"),
        "Multiple commands indented: {formatted}"
    );

    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second);
}

#[test]
fn e2e_nested_if_else() {
    let input = r#"if [ -d /tmp ]; then
if [ -w /tmp ]; then
if [ -x /tmp ]; then
echo "all good"
else
echo "not executable"
fi
fi
fi
"#;

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    // Three levels of nesting
    assert!(
        formatted.contains("  if [ -w"),
        "Level 1 indent: {formatted}"
    );
    assert!(
        formatted.contains("    if [ -x"),
        "Level 2 indent: {formatted}"
    );
    assert!(
        formatted.contains("      echo \"all good\""),
        "Level 3 indent: {formatted}"
    );

    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second);
}

#[test]
fn e2e_function_keyword_preserved() {
    let input = "function my_func {\necho hello\n}\n";

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    // 'function' keyword should be preserved (valid zsh form)
    assert!(
        formatted.contains("function my_func"),
        "function keyword should be preserved: {formatted}"
    );

    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second);
}

#[test]
fn e2e_variable_expansion_normalization() {
    let input = "echo $foo\n";

    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);

    // $foo should be normalized to ${foo}
    assert!(
        formatted.contains("${foo}"),
        "$var should become ${{var}}: {formatted}"
    );

    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second);
}

// ---------------------------------------------------------------------------
// E2E: Custom indent settings
// ---------------------------------------------------------------------------

#[test]
fn e2e_tab_indent() {
    let input = "if [ -f x ]; then\necho found\nfi\n";

    let mut child = zsheesh()
        .arg("--indent")
        .arg("\t")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\techo"), "Tab indent: {stdout}");
}

#[test]
fn e2e_four_space_indent() {
    let input = "if [ -f x ]; then\necho found\nfi\n";

    let mut child = zsheesh()
        .arg("--indent")
        .arg("    ")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("    echo"), "4-space indent: {stdout}");
}

// ---------------------------------------------------------------------------
// E2E: AST dump mode
// ---------------------------------------------------------------------------

#[test]
fn e2e_dump_ast_produces_valid_json() {
    let input = "echo hello\nif [ -f x ]; then\necho found\nfi\n";

    let mut child = zsheesh()
        .arg("--dump-ast")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    // Should be valid JSON containing tree-sitter node types
    assert!(stdout.contains("program"), "AST root: {stdout}");
    assert!(stdout.contains("command"), "Command node: {stdout}");
    assert!(
        stdout.contains("if_statement"),
        "If statement node: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// E2E: Corpus files — format each and verify idempotence
// ---------------------------------------------------------------------------

macro_rules! corpus_e2e {
    ($name:ident, $file:expr) => {
        #[test]
        fn $name() {
            let input = include_str!(concat!("../corpus/", $file));
            let (formatted, stderr, ok) = run_stdin(input);
            assert!(ok, "Formatting {}: stderr={}", $file, stderr);

            let (second, _, ok2) = run_stdin(&formatted);
            assert!(ok2, "Second pass should succeed");
            assert_eq!(formatted, second, "Idempotence failed for {}", $file);

            assert!(
                run_stdin_check(&formatted),
                "--check should pass for {}",
                $file
            );
        }
    };
}

corpus_e2e!(e2e_corpus_basic, "basic.zsh");
corpus_e2e!(e2e_corpus_functions, "functions.zsh");
corpus_e2e!(e2e_corpus_conditionals, "conditionals.zsh");
corpus_e2e!(e2e_corpus_loops, "loops.zsh");
corpus_e2e!(e2e_corpus_case_statement, "case_statement.zsh");
corpus_e2e!(e2e_corpus_pipelines, "pipelines.zsh");
corpus_e2e!(e2e_corpus_variables, "variables.zsh");
corpus_e2e!(e2e_corpus_heredoc, "heredoc.zsh");
corpus_e2e!(e2e_corpus_redirections, "redirections.zsh");
// zsh_specific.zsh uses parameter expansion flags parsed by tree-sitter-zsh.
corpus_e2e!(e2e_corpus_zsh_specific, "zsh_specific.zsh");
corpus_e2e!(e2e_corpus_real_zshrc, "real_zshrc.zsh");
corpus_e2e!(e2e_corpus_fmt_directives, "fmt_directives.zsh");

// ---------------------------------------------------------------------------
// E2E: Edge cases
// ---------------------------------------------------------------------------

#[test]
fn e2e_empty_input() {
    let (_, _, ok) = run_stdin("");
    assert!(ok, "Empty input should not crash");
}

#[test]
fn e2e_shebang_only() {
    let (formatted, _, ok) = run_stdin("#!/bin/zsh\n");
    assert!(ok);
    assert!(formatted.contains("#!/bin/zsh"));
}

#[test]
fn e2e_comments_only() {
    let input = "# This is a comment\n# Another comment\n";
    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);
    assert!(formatted.contains("# This is a comment"));
    assert!(formatted.contains("# Another comment"));
}

#[test]
fn e2e_nonexistent_file_errors_gracefully() {
    let output = zsheesh().arg("/nonexistent/file.zsh").output().unwrap();
    assert!(!output.status.success(), "Should fail for nonexistent file");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("zsheesh:") || stderr.contains("No such file"),
        "Should report error: {stderr}"
    );
}

#[test]
fn e2e_blank_lines_preserved() {
    let input = "echo a\n\necho b\n\n\necho c\n";
    let (formatted, _, ok) = run_stdin(input);
    assert!(ok);
    // At least one blank line between commands should be preserved
    assert!(
        formatted.contains("\n\n"),
        "Blank lines should be preserved: {formatted}"
    );
}

#[test]
fn e2e_large_script() {
    // Generate a reasonably large script to verify performance
    let mut script = String::from("#!/bin/zsh\n\n");
    for i in 0..100 {
        script.push_str(&format!(
            "func_{i}() {{\n  echo \"function {i}\"\n  if [ $1 -eq {i} ]; then\n    echo \"match\"\n  fi\n}}\n\n"
        ));
    }

    let (formatted, _, ok) = run_stdin(&script);
    assert!(ok, "Large script should format successfully");

    let (second, _, _) = run_stdin(&formatted);
    assert_eq!(formatted, second, "Large script must be idempotent");
}
