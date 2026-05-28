use zsheesh::{ZshFormatter, protect_regions, restore_regions};

fn fmt(input: &str) -> String {
    let formatter = ZshFormatter::new().unwrap();
    let (protected, regions) = protect_regions(input);
    let formatted = formatter.format_str(&protected).unwrap();
    restore_regions(&formatted, &regions)
}

fn assert_idempotent(input: &str) {
    let first = fmt(input);
    let second = fmt(&first);
    assert_eq!(
        first, second,
        "Formatter is not idempotent.\nFirst pass:\n{first}\nSecond pass:\n{second}"
    );
}

// ---------------------------------------------------------------------------
// Basic formatting
// ---------------------------------------------------------------------------

#[test]
fn formats_simple_echo() {
    let input = "echo \"hello\"\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r#"
    echo "hello"
    "#);
}

#[test]
fn formats_multiple_commands() {
    let input = "echo \"hello\"\necho \"world\"\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r#"
    echo "hello"
    echo "world"
    "#);
}

#[test]
fn adds_trailing_newline() {
    let input = "echo \"hello\"";
    let result = fmt(input);
    assert!(result.ends_with('\n'), "Output must end with newline");
}

// ---------------------------------------------------------------------------
// Indentation
// ---------------------------------------------------------------------------

#[test]
fn indents_if_body() {
    let input = "if [ -f /tmp/x ]; then\necho found\nfi\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    if [ -f /tmp/x ]; then
      echo found
    fi
    ");
}

#[test]
fn indents_function_body() {
    let input = "greet(){\necho hello\necho world\n}\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    greet() {
      echo hello
      echo world
    }
    ");
}

#[test]
fn indents_for_loop() {
    let input = "for f in *.txt; do\necho $f\ndone\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    for f in *.txt; do
      echo ${f}
    done
    ");
}

#[test]
fn indents_while_loop() {
    let input = "while true; do\necho loop\nbreak\ndone\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    while true; do
      echo loop
      break
    done
    ");
}

#[test]
fn indents_case_statement() {
    let input = "case $1 in\nstart)\necho starting\n;;\nstop)\necho stopping\n;;\nesac\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    case $1 in
      start)
        echo starting
        ;;
      stop)
        echo stopping
        ;;
    esac
    ");
}

// ---------------------------------------------------------------------------
// If / elif / else
// ---------------------------------------------------------------------------

#[test]
fn formats_if_elif_else() {
    let input = r#"if [ "$1" = "a" ]; then
echo a
elif [ "$1" = "b" ]; then
echo b
else
echo other
fi
"#;
    let result = fmt(input);
    insta::assert_snapshot!(result, @r#"
    if [ "$1" = "a" ]; then
      echo a
    elif [ "$1" = "b" ]; then
      echo b
    else
      echo other
    fi
    "#);
}

#[test]
fn formats_nested_if() {
    let input = "if [ -d /tmp ]; then\nif [ -w /tmp ]; then\necho writable\nfi\nfi\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    if [ -d /tmp ]; then
      if [ -w /tmp ]; then
        echo writable
      fi
    fi
    ");
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

#[test]
fn normalizes_function_parens() {
    let input = "greet  (  ) {\necho hi\n}\n";
    let result = fmt(input);
    assert!(result.contains("greet()"), "Should normalize to greet()");
}

#[test]
fn formats_function_keyword_style() {
    // The 'function' keyword should be removed per the query rules
    let input = "function greet {\necho hi\n}\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    greet() {
      echo hi
    }
    ");
}

// ---------------------------------------------------------------------------
// Pipelines and lists
// ---------------------------------------------------------------------------

#[test]
fn formats_pipeline() {
    let input = "cat file|grep pattern|sort\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    cat file | grep pattern | sort
    ");
}

#[test]
fn formats_command_list_and() {
    let input = "mkdir -p /tmp/test&&echo created||echo failed\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    mkdir -p /tmp/test && echo created || echo failed
    ");
}

// ---------------------------------------------------------------------------
// Variables
// ---------------------------------------------------------------------------

#[test]
fn formats_export() {
    let input = "export  PATH=\"/usr/local/bin:$PATH\"\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r#"
    export PATH="/usr/local/bin:$PATH"
    "#);
}

#[test]
fn formats_local_var() {
    let input = "local  my_var=\"hello\"\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r#"
    local my_var="hello"
    "#);
}

// ---------------------------------------------------------------------------
// Redirections
// ---------------------------------------------------------------------------

#[test]
fn formats_output_redirect() {
    let input = "echo hello>output.txt\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    echo hello >output.txt
    ");
}

#[test]
fn formats_stderr_redirect() {
    let input = "command 2>/dev/null\n";
    let result = fmt(input);
    assert!(result.contains("2>"));
}

// ---------------------------------------------------------------------------
// Compound statements and subshells
// ---------------------------------------------------------------------------

#[test]
fn formats_compound_statement() {
    let input = "{ echo a; echo b; }\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    { echo a; echo b; }
    ");
}

#[test]
fn formats_subshell() {
    let input = "(echo a; echo b)\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    ( echo a; echo b )
    ");
}

// ---------------------------------------------------------------------------
// Test commands
// ---------------------------------------------------------------------------

#[test]
fn formats_single_bracket_test() {
    let input = "[ -f /tmp/x ]\n";
    let result = fmt(input);
    assert!(result.contains("[ -f /tmp/x ]"), "Single bracket spacing");
}

#[test]
fn formats_double_bracket_test() {
    let input = "[[ -n \"${VAR}\" ]]\n";
    let result = fmt(input);
    assert!(
        result.contains("[[ -n"),
        "Double bracket spacing: got {result}"
    );
}

// ---------------------------------------------------------------------------
// Arithmetic
// ---------------------------------------------------------------------------

#[test]
fn formats_arithmetic_expansion() {
    let input = "echo $(( 1 + 2 ))\n";
    let result = fmt(input);
    assert!(result.contains("$(( 1 + 2 ))") || result.contains("$((1 + 2))"));
}

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

#[test]
fn preserves_shebang() {
    let input = "#!/bin/zsh\necho hello\n";
    let result = fmt(input);
    assert!(result.starts_with("#!/bin/zsh\n"));
}

#[test]
fn preserves_comment_content() {
    let input = "# This is a comment\necho hello\n";
    let result = fmt(input);
    assert!(result.contains("# This is a comment"));
}

// ---------------------------------------------------------------------------
// fmt: skip / off / on
// ---------------------------------------------------------------------------

#[test]
fn fmt_skip_preserves_next_line() {
    let input = "echo before\n# fmt: skip\necho    this   is   preserved\necho after\n";
    let result = fmt(input);
    assert!(
        result.contains("echo    this   is   preserved"),
        "fmt: skip should preserve the next line"
    );
}

#[test]
fn fmt_off_on_preserves_region() {
    let input = "echo before\n# fmt: off\necho   a\necho   b\n# fmt: on\necho after\n";
    let result = fmt(input);
    assert!(
        result.contains("echo   a"),
        "fmt: off/on should preserve region"
    );
    assert!(
        result.contains("echo   b"),
        "fmt: off/on should preserve region"
    );
}

#[test]
fn fmt_off_without_on_preserves_rest() {
    let input = "echo before\n# fmt: off\necho   a\necho   b\n";
    let result = fmt(input);
    assert!(
        result.contains("echo   a"),
        "fmt: off without on should preserve to end"
    );
}

// ---------------------------------------------------------------------------
// Idempotence
// ---------------------------------------------------------------------------

#[test]
fn idempotent_basic() {
    assert_idempotent("echo hello\n");
}

#[test]
fn idempotent_function() {
    assert_idempotent("greet() {\n  echo hello\n}\n");
}

#[test]
fn idempotent_if_statement() {
    assert_idempotent("if [ -f /tmp/x ]; then\n  echo found\nfi\n");
}

#[test]
fn idempotent_case() {
    assert_idempotent("case $1 in\n  start)\n    echo go\n    ;;\nesac\n");
}

#[test]
fn idempotent_for_loop() {
    assert_idempotent("for f in *.txt; do\n  echo ${f}\ndone\n");
}

#[test]
fn idempotent_pipeline() {
    assert_idempotent("cat file | grep pattern | sort\n");
}

// ---------------------------------------------------------------------------
// Corpus files — parse and format without error
// ---------------------------------------------------------------------------

#[test]
fn corpus_basic() {
    let input = include_str!("../corpus/basic.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

#[test]
fn corpus_functions() {
    let input = include_str!("../corpus/functions.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

#[test]
fn corpus_conditionals() {
    let input = include_str!("../corpus/conditionals.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

#[test]
fn corpus_loops() {
    let input = include_str!("../corpus/loops.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

#[test]
fn corpus_case_statement() {
    let input = include_str!("../corpus/case_statement.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

#[test]
fn corpus_pipelines() {
    let input = include_str!("../corpus/pipelines.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

#[test]
fn corpus_variables() {
    let input = include_str!("../corpus/variables.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

#[test]
fn corpus_heredoc() {
    let input = include_str!("../corpus/heredoc.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

#[test]
fn corpus_redirections() {
    let input = include_str!("../corpus/redirections.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

#[test]
fn corpus_zsh_specific() {
    let input = include_str!("../corpus/zsh_specific.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

#[test]
fn corpus_real_zshrc() {
    let input = include_str!("../corpus/real_zshrc.zsh");
    let _ = fmt(input);
    assert_idempotent(&fmt(input));
}

// ---------------------------------------------------------------------------
// Check mode
// ---------------------------------------------------------------------------

#[test]
fn check_returns_true_for_formatted() {
    let formatter = ZshFormatter::new().unwrap();
    let input = "echo hello\n";
    let formatted = formatter.format_str(input).unwrap();
    assert!(formatter.check_str(&formatted).unwrap());
}

#[test]
fn check_returns_false_for_unformatted() {
    let formatter = ZshFormatter::new().unwrap();
    let input = "if [ -f x ]; then\necho y\nfi\n";
    assert!(!formatter.check_str(input).unwrap());
}

// ---------------------------------------------------------------------------
// Custom indent
// ---------------------------------------------------------------------------

#[test]
fn custom_indent_tab() {
    let formatter = ZshFormatter::with_indent("\t").unwrap();
    let input = "if [ -f x ]; then\necho found\nfi\n";
    let result = formatter.format_str(input).unwrap();
    assert!(
        result.contains("\techo"),
        "Should indent with tab: got {result}"
    );
}

#[test]
fn custom_indent_four_spaces() {
    let formatter = ZshFormatter::with_indent("    ").unwrap();
    let input = "if [ -f x ]; then\necho found\nfi\n";
    let result = formatter.format_str(input).unwrap();
    assert!(
        result.contains("    echo"),
        "Should indent with 4 spaces: got {result}"
    );
}

// ---------------------------------------------------------------------------
// AST dump
// ---------------------------------------------------------------------------

#[test]
fn dump_ast_produces_json() {
    let formatter = ZshFormatter::new().unwrap();
    let result = formatter.dump_ast("echo hello\n").unwrap();
    assert!(
        result.contains("program") || result.contains("command"),
        "AST dump should contain tree-sitter node types"
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_input() {
    let result = fmt("");
    assert!(!result.is_empty() || result.is_empty()); // should not crash
}

#[test]
fn only_comments() {
    let input = "# just a comment\n";
    let result = fmt(input);
    assert!(result.contains("# just a comment"));
}

#[test]
fn only_shebang() {
    let input = "#!/bin/zsh\n";
    let result = fmt(input);
    assert!(result.contains("#!/bin/zsh"));
}

#[test]
fn preserves_blank_lines_between_blocks() {
    let input = "echo a\n\necho b\n";
    let result = fmt(input);
    assert!(
        result.contains("\n\n"),
        "Should preserve blank lines between commands"
    );
}
