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
    // The 'function' keyword is preserved (valid zsh form)
    let input = "function greet {\necho hi\n}\n";
    let result = fmt(input);
    insta::assert_snapshot!(result, @r"
    function greet {
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
// Zsh grammar extensions
// ---------------------------------------------------------------------------

#[test]
fn zsh_background_disown() {
    let input = "sleep 10 &|\n";
    let result = fmt(input);
    assert!(result.contains("&|"), "Should preserve &| operator: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_force_clobber_redirect() {
    let input = "echo data >! /tmp/output\n";
    let result = fmt(input);
    assert!(result.contains(">!"), "Should preserve >! redirect: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_dollar_plus_var_set_check() {
    let input = "if (( $+commands[git] )); then\necho yes\nfi\n";
    let result = fmt(input);
    assert!(result.contains("$+commands"), "Should parse $+var: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_expansion_plus_operator() {
    let input = "echo ${+PATH}\n";
    let result = fmt(input);
    assert!(result.contains("${+PATH}"), "Should parse ${{+var}}: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_expansion_plus_subscript() {
    let input = "echo ${+commands[git]}\n";
    let result = fmt(input);
    assert!(result.contains("${+commands[git]}"), "Should parse ${{+var[key]}}: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_nested_expansion() {
    let input = "0=\"${${ZERO:-foo}:-bar}\"\n";
    let result = fmt(input);
    assert!(result.contains("${${ZERO:-foo}:-bar}"), "Should handle nested expansion: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_triple_nested_expansion() {
    let input = "0=\"${${ZERO:-${0:#$ZSH_ARGZERO}}:-${(%):-%N}}\"\n";
    let result = fmt(input);
    assert!(result.contains("${${ZERO:-"), "Should handle triple-nested: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_colon_hash_operator() {
    let input = "echo \"${(M)0:#/*}\"\n";
    let result = fmt(input);
    assert!(result.contains(":#"), "Should handle :# operator: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_for_list_short_form() {
    let input = "for v (a b c); do\necho $v\ndone\n";
    let result = fmt(input);
    assert!(result.contains("for"), "Should handle for (list): got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_function_anonymous() {
    let input = "function {\necho anonymous\n}\n";
    let result = fmt(input);
    assert!(result.contains("echo anonymous"), "Should parse anonymous function: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_try_always() {
    let input = "{\necho try\n} always {\necho cleanup\n}\n";
    let result = fmt(input);
    assert!(result.contains("always"), "Should handle try/always: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_flags_with_string_target() {
    let input = "lines=(${(f)\"$(git status)\"})\n";
    let result = fmt(input);
    assert!(result.contains("${(f)"), "Should handle flags with string target: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_flags_with_command_substitution() {
    let input = "a=(${(@f)\"$(cmd)\"})\n";
    let result = fmt(input);
    assert!(result.contains("${(@f)"), "Should handle flags with cmd sub target: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_prompt_expansion_empty_target() {
    let input = "echo \"${(%):-%N}\"\n";
    let result = fmt(input);
    assert!(result.contains("${(%):-%N}"), "Should handle empty target flags: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_gs_modifier() {
    let input = "echo \"${rvm_prompt:gs/%/%%}\"\n";
    let result = fmt(input);
    assert!(result.contains(":gs/%/%%"), "Should handle :gs modifier: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_lowercase_modifier() {
    let input = "echo ${issue_arg:l}\n";
    let result = fmt(input);
    assert!(result.contains(":l"), "Should handle :l modifier: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_flag_separator() {
    let input = "parts=(${(s:.:)HOST})\n";
    let result = fmt(input);
    assert!(result.contains("${(s:.:)HOST}"), "Should handle flag separator: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_dollar_hash_var() {
    let input = "if (( $#remotes > 0 )); then\necho yes\nfi\n";
    let result = fmt(input);
    assert!(result.contains("$#"), "Should handle $#var length: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_equals_flag_expansion() {
    let input = "echo ${=icon:+--icon \"$icon\"}\n";
    let result = fmt(input);
    assert!(result.contains("${=icon"), "Should handle ${{=var}} word split: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_subscript_flag() {
    let input = "echo ${available_profiles[(r)$1]}\n";
    let result = fmt(input);
    assert!(result.contains("[(r)"), "Should handle subscript flag: got {result}");
    assert_idempotent(&result);
}

// ---------------------------------------------------------------------------
// Zsh grammar extensions — session 2
// ---------------------------------------------------------------------------

#[test]
fn zsh_caret_distribute_prefix() {
    let input = "echo ${^PATH}\n";
    let result = fmt(input);
    assert!(result.contains("${^PATH}"), "Should handle ${{^var}} distribute: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_caret_distribute_subscript() {
    let input = "for file in \"${^PYTHON_VENV_NAMES[@]}\"/bin/activate; do\necho $file\ndone\n";
    let result = fmt(input);
    assert!(result.contains("${^PYTHON_VENV_NAMES[@]}"), "Should handle ${{^var[@]}}: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_tilde_glob_prefix() {
    let input = "echo ${~var}\n";
    let result = fmt(input);
    assert!(result.contains("${~var}"), "Should handle ${{~var}} glob: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_flag_separator_slash_delimiter() {
    let input = "echo ${(@s/:/)var}\n";
    let result = fmt(input);
    assert!(result.contains("${(@s/:/)var}"), "Should handle s/:/ with / delimiter: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_multichar_subscript_flag() {
    let input = "if [[ ${tools[(Ie)$TOOL]} -eq 0 ]]; then\necho missing\nfi\n";
    let result = fmt(input);
    assert!(result.contains("[(Ie)"), "Should handle multi-char subscript flag (Ie): got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_array_difference_operator() {
    let input = "bundled=(${bundled:|UNBUNDLED})\n";
    let result = fmt(input);
    assert!(result.contains(":|"), "Should handle :| array difference: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_flags_at_target() {
    let input = "local query=\"${(j:,:)@}\"\n";
    let result = fmt(input);
    assert!(result.contains("${(j:,:)@}"), "Should handle flags with @ target: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_nested_expansion_subscript() {
    let input = "local word=${${(Az)LBUFFER}[-1]}\n";
    let result = fmt(input);
    assert!(result.contains("[-1]"), "Should handle nested expansion with subscript: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_string_in_expansion() {
    let input = "local fzf_ver=${\"$(fzf --version)\"#fzf }\n";
    let result = fmt(input);
    assert!(result.contains("${\""), "Should handle string in expansion body: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_command_sub_in_expansion() {
    let input = "local nvm_prompt=${$(nvm current)#v}\n";
    let result = fmt(input);
    assert!(result.contains("${$(nvm current)#v}"), "Should handle cmd sub in expansion: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_length_of_nested_expansion() {
    let input = "echo ${#${var}}\n";
    let result = fmt(input);
    assert!(result.contains("${#${var}}"), "Should handle ${{#${{nested}}}}: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_special_variable_subscript() {
    let input = "echo ${@[2,-1]}\n";
    let result = fmt(input);
    assert!(result.contains("${@[2,-1]}"), "Should handle ${{@[idx]}}: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_for_multiple_variables() {
    let input = "for k v in a b c d; do\necho $k $v\ndone\n";
    let result = fmt(input);
    assert!(result.contains("for k v in"), "Should handle multi-var for loop: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_if_no_semicolon_before_then() {
    let input = "if [[ -z \"$1\" ]] then\necho hi\nfi\n";
    let result = fmt(input);
    assert!(result.contains("then"), "Should handle if [[ ]] then (no ;): got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_function_multi_name() {
    let input = "function man foo {\necho ok\n}\n";
    let result = fmt(input);
    assert!(result.contains("man") && result.contains("foo"), "Should handle multi-name function: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_modifier_uppercase_p() {
    let input = "echo ${commands[aws]:P}\n";
    let result = fmt(input);
    assert!(result.contains(":P"), "Should handle :P modifier: got {result}");
    assert_idempotent(&result);
}

#[test]
fn zsh_redirect_with_herestring() {
    let input = "command grep -E 'test' &>/dev/null <<< \"$status\"\n";
    let result = fmt(input);
    assert!(result.contains("&>/dev/null"), "Should handle &> redirect: got {result}");
    assert!(result.contains("<<<"), "Should handle <<< herestring: got {result}");
    assert_idempotent(&result);
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

// ---------------------------------------------------------------------------
// Redirects on compound statements
//
// A break between the closing keyword and its redirect is not a style choice:
// zsh reads the redirect as a separate command and leaves the loop reading its
// parent's stdin, which hangs on a terminal. Losing the space in `< <(cmd)` is
// the same class of damage — `<<(` lexes as a heredoc.
// ---------------------------------------------------------------------------

#[test]
fn keeps_process_substitution_redirect_spaced() {
    let result = fmt("cat < <(cmd)\n");
    assert!(
        result.contains("< <("),
        "`< <(` must keep its space or it lexes as a heredoc: got {result}"
    );
    assert_idempotent(&result);
}

#[test]
fn keeps_output_process_substitution_redirect_spaced() {
    let result = fmt("cmd > >(tee log)\n");
    assert!(
        result.contains("> >("),
        "`> >(` must keep its space or it appends to a file named \"(tee log)\": got {result}"
    );
    assert_idempotent(&result);
}

#[test]
fn keeps_redirect_on_done_line() {
    let result = fmt("while read -r l; do\nprint \"$l\"\ndone < <(cmd)\n");
    assert!(
        result.contains("done < <(cmd)"),
        "Redirect must stay on the `done` line: got {result}"
    );
    assert_idempotent(&result);
}

#[test]
fn keeps_redirect_on_done_line_for_file() {
    let result = fmt("while read -r x; do\nprint \"$x\"\ndone <\"$file\"\n");
    assert!(
        result.contains("done <\"$file\""),
        "Redirect must stay on the `done` line: got {result}"
    );
    assert_idempotent(&result);
}

#[test]
fn keeps_redirect_on_fi_line() {
    let result = fmt("if true; then\nprint x\nfi >out.txt\n");
    assert!(
        result.contains("fi >out.txt"),
        "Redirect must stay on the `fi` line: got {result}"
    );
    assert_idempotent(&result);
}

#[test]
fn keeps_redirect_on_esac_line() {
    let result = fmt("case $x in\na) print y ;;\nesac >out.txt\n");
    assert!(
        result.contains("esac >out.txt"),
        "Redirect must stay on the `esac` line: got {result}"
    );
    assert_idempotent(&result);
}

#[test]
fn keeps_redirect_on_function_body_line() {
    let result = fmt("f() { print a; } >out.txt\n");
    assert!(
        result.contains("} >out.txt"),
        "Redirect must stay on the function body's closing line: got {result}"
    );
    assert_idempotent(&result);
}

#[test]
fn keeps_redirect_on_for_done_line() {
    let result = fmt("for f in a b; do\nprint \"$f\"\ndone >out.txt\n");
    assert!(
        result.contains("done >out.txt"),
        "Redirect must stay on the `done` line: got {result}"
    );
    assert_idempotent(&result);
}

// ---------------------------------------------------------------------------
// Comment placement
// ---------------------------------------------------------------------------

#[test]
fn keeps_own_line_comment_after_done() {
    let result = fmt("while true; do\nx\ndone\n# own line\ny\n");
    assert!(
        result.contains("done\n# own line"),
        "A comment on its own line must not be pulled onto the `done` line: got {result}"
    );
    assert_idempotent(&result);
}

#[test]
fn keeps_trailing_comment_after_done() {
    let result = fmt("while true; do x; done # trailing\n");
    assert!(
        result.contains("done # trailing"),
        "A trailing comment must stay on the `done` line: got {result}"
    );
    assert_idempotent(&result);
}
