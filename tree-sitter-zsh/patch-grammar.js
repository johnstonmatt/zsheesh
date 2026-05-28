#!/usr/bin/env node
//
// patch-grammar.js — apply zsh-specific changes to a tree-sitter-bash grammar.js
//
// Called by sync-from-bash.sh. Reads the file path from argv[2], patches in
// place. Each patch targets a stable anchor in the bash grammar so that minor
// upstream edits don't break it.

"use strict";

const fs = require("fs");

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function replaceStr(src, find, replacement, patchName) {
  const idx = src.indexOf(find);
  if (idx === -1) {
    console.error(`PATCH FAILED [${patchName}]: string not found:\n  ${find.slice(0, 120)}...`);
    process.exit(1);
  }
  return src.slice(0, idx) + replacement + src.slice(idx + find.length);
}

function replaceRe(src, find, replacement, patchName) {
  const result = src.replace(find, replacement);
  if (result === src) {
    console.error(`PATCH FAILED [${patchName}]: regex not matched: ${find}`);
    process.exit(1);
  }
  return result;
}

function insertAfter(src, anchor, content, patchName) {
  const idx = src.indexOf(anchor);
  if (idx === -1) {
    console.error(`PATCH FAILED [${patchName}]: anchor not found:\n  ${anchor.slice(0, 120)}...`);
    process.exit(1);
  }
  const pos = idx + anchor.length;
  return src.slice(0, pos) + content + src.slice(pos);
}

function insertBefore(src, anchor, content, patchName) {
  const idx = src.indexOf(anchor);
  if (idx === -1) {
    console.error(`PATCH FAILED [${patchName}]: anchor not found:\n  ${anchor.slice(0, 120)}...`);
    process.exit(1);
  }
  return src.slice(0, idx) + content + src.slice(idx);
}

function verify(src, regex, patchName) {
  if (!regex.test(src)) {
    console.error(`PATCH VERIFY FAILED [${patchName}]`);
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Apply patches
// ---------------------------------------------------------------------------

const file = process.argv[2];
if (!file) {
  console.error("Usage: node patch-grammar.js <grammar.js>");
  process.exit(1);
}

let src = fs.readFileSync(file, "utf8");
const original = src;
let count = 0;

// ── 1. Header: bash → zsh ──────────────────────────────────────────────────
src = replaceRe(src, /\* @file Bash grammar for tree-sitter/,
  "* @file Zsh grammar for tree-sitter (forked from tree-sitter-bash)", "header");
verify(src, /Zsh grammar/, "header");
count++;

// ── 2. Grammar name ────────────────────────────────────────────────────────
src = replaceRe(src, /name: 'bash'/, "name: 'zsh'", "grammar name");
verify(src, /name: 'zsh'/, "grammar name");
count++;

// ── 3. Add zsh conflict ────────────────────────────────────────────────────
src = insertAfter(src, "[$.pipeline],",
  "\n    [$.zsh_flags_expansion, $._expansion_regex],", "conflicts");
verify(src, /zsh_flags_expansion.*_expansion_regex/, "conflicts");
count++;

// ── 4. Redirected statement: allow herestring after file redirect ───────────
src = replaceStr(src,
  `      ),
      seq(
        field('body', choice($.if_statement, $.while_statement)),
        $.herestring_redirect,`,
  `        // zsh: cmd &>/dev/null <<< "input" — herestring after other redirects
        optional($.herestring_redirect),
      ),
      seq(
        field('body', choice($.if_statement, $.while_statement)),
        $.herestring_redirect,`,
  "herestring after redirect");
verify(src, /herestring after other redirects/, "herestring after redirect");
count++;

// ── 5. For statement: multiple loop variables + (list) form ─────────────────
src = replaceStr(src,
  `    for_statement: $ => seq(
      choice('for', 'select'),
      field('variable', $._simple_variable_name),
      optional(seq(
        'in',
        field('value', repeat1($._literal)),
      )),
      $._terminator,
      field('body', $.do_group),
    ),`,
  `    for_statement: $ => choice(
      seq(
        choice('for', 'select'),
        // zsh: for k v in list — multiple loop variables
        repeat1(field('variable', $._simple_variable_name)),
        optional(seq(
          'in',
          field('value', repeat1($._literal)),
        )),
        $._terminator,
        field('body', $.do_group),
      ),
      // zsh: for var (list); do ... done
      prec(1, seq(
        'for',
        field('variable', $._simple_variable_name),
        '(',
        field('value', repeat($._literal)),
        ')',
        optional($._terminator),
        field('body', $.do_group),
      )),
    ),`,
  "for statement");
verify(src, /for k v in list/, "for statement");
count++;

// ── 6. If statement: allow unterminated condition ───────────────────────────
src = replaceStr(src,
  "      field('condition', $._terminated_statement),\n      'then',",
  "      field('condition', choice($._terminated_statement, $._statement)),\n      'then',",
  "if condition");
verify(src, /field\('condition', choice\(/, "if condition");
count++;

// ── 7. Elif clause: allow unterminated condition ────────────────────────────
src = replaceStr(src,
  "      'elif',\n      $._terminated_statement,\n      'then',",
  "      'elif',\n      choice($._terminated_statement, $._statement),\n      'then',",
  "elif condition");
verify(src, /elif.*\n.*choice\(\$\._terminated/, "elif condition");
count++;

// ── 8. Function definition: multi-name + anonymous ──────────────────────────
src = replaceStr(src,
  `    function_definition: $ => prec.right(seq(
      choice(
        seq(
          'function',
          field('name', $.word),
          optional(seq('(', ')')),
        ),
        seq(
          field('name', $.word),
          '(', ')',
        ),
      ),
      field(
        'body',
        choice(
          $.compound_statement,
          $.subshell,
          $.test_command,
          $.if_statement,
        ),
      ),
      field('redirect', optional($._redirect)),
    )),`,
  `    function_definition: $ => prec.right(choice(
      seq(
        choice(
          seq(
            'function',
            // zsh: function can define multiple names: function name1 name2 { }
            repeat1(field('name', $.word)),
            optional(seq('(', ')')),
          ),
          seq(
            field('name', $.word),
            '(', ')',
          ),
        ),
        field(
          'body',
          choice(
            $.compound_statement,
            $.subshell,
            $.test_command,
            $.if_statement,
          ),
        ),
        field('redirect', optional($._redirect)),
      ),
      // zsh: function { body } — anonymous function (no name)
      prec(-1, seq(
        'function',
        field(
          'body',
          choice(
            $.compound_statement,
            $.subshell,
          ),
        ),
        field('redirect', optional($._redirect)),
      )),
    )),`,
  "function definition");
verify(src, /function can define multiple names/, "function definition");
count++;

// ── 9. Compound statement: always block + (( )) form ────────────────────────
src = replaceStr(src,
  `    compound_statement: $ => seq(
      '{',
      optional($._terminated_statement),
      token(prec(-1, '}')),
    ),`,
  `    compound_statement: $ => choice(
      seq(
        '{',
        optional($._terminated_statement),
        token(prec(-1, '}')),
        // zsh: optional always block
        optional(seq(
          'always',
          '{',
          optional($._terminated_statement),
          token(prec(-1, '}')),
        )),
      ),
      seq(
        '((',
        repeat(
          seq(
            $._arithmetic_expression,
            ',',
          ),
        ),
        $._arithmetic_expression,
        '))'),
    ),`,
  "compound statement");
verify(src, /optional always block/, "compound statement");
count++;

// ── 10. Subscript: prec.left + optional flag ────────────────────────────────
src = replaceStr(src,
  `    subscript: $ => seq(
      field('name', $.variable_name),
      '[',
      field('index', choice($._literal, $.binary_expression, $.unary_expression, $.parenthesized_expression)),
      optional($._concat),
      ']',
      optional($._concat),
    ),`,
  `    subscript: $ => prec.left(seq(
      field('name', $.variable_name),
      '[',
      // zsh: optional subscript flag like (r), (i), (I), (k), (K), (R), (Ie), (Re)
      optional(/\\([a-zA-Z]+\\)/),
      field('index', choice($._literal, $.binary_expression, $.unary_expression, $.compound_statement, $.subshell)),
      optional($._concat),
      ']',
      optional($._concat),
    )),`,
  "subscript");
verify(src, /optional subscript flag/, "subscript");
count++;

// ── 11. File redirect: add >! ───────────────────────────────────────────────
src = replaceStr(src,
  "          choice('<', '>', '>>', '&>', '&>>', '<&', '>&', '>|'),",
  "          choice('<', '>', '>>', '&>', '&>>', '<&', '>&', '>|', '>!'),",
  ">! redirect");
verify(src, /'>!'/, ">! redirect");
count++;

// ── 12. Simple expansion: $+var and $#var ───────────────────────────────────
src = insertAfter(src,
  "        alias('#', $.special_variable_name),",
  `
        // zsh: $+var checks if var is set (returns 0/1)
        seq('+', choice($._simple_variable_name, $.variable_name, $.subscript)),
        // zsh: $#var gives length of var (higher prec so $#remotes is one unit)
        prec(1, seq('#', choice($._simple_variable_name, $.variable_name))),`,
  "$+var $#var");
verify(src, /\$\+var checks if var is set/, "$+var $#var");
count++;

// ── 13. Replace entire _expansion_body with zsh-extended version ────────────
src = replaceStr(src,
  `    _expansion_body: $ => choice(
      // \${!##} \${!#}
      repeat1(field(
        'operator',
        choice(
          alias($._external_expansion_sym_hash, '#'),
          alias($._external_expansion_sym_bang, '!'),
          alias($._external_expansion_sym_equal, '='),
        ),
      )),
      seq(
        optional(field('operator', token.immediate('!'))),
        choice($.variable_name, $._simple_variable_name, $._special_variable_name, $.subscript),
        choice(
          $._expansion_expression,
          $._expansion_regex,
          $._expansion_regex_replacement,
          $._expansion_regex_removal,
          $._expansion_max_length,
          $._expansion_operator,
        ),
      ),
      seq(
        field('operator', token.immediate('!')),
        choice($._simple_variable_name, $.variable_name),
        optional(field('operator', choice(
          token.immediate('@'),
          token.immediate('*'),
        ))),
      ),
      seq(
        optional(field('operator', immediateLiterals('#', '!', '='))),
        choice(
          $.subscript,
          $._simple_variable_name,
          $._special_variable_name,
          $.command_substitution,
        ),
        repeat(field(
          'operator',
          choice(
            alias($._external_expansion_sym_hash, '#'),
            alias($._external_expansion_sym_bang, '!'),
            alias($._external_expansion_sym_equal, '='),
          ),
        )),
      ),
    ),`,
  `    _expansion_body: $ => choice(
      // Zsh parameter expansion flags: \${(flags)name}
      $.zsh_flags_expansion,
      // \${!##} \${!#}
      repeat1(field(
        'operator',
        choice(
          alias($._external_expansion_sym_hash, '#'),
          alias($._external_expansion_sym_bang, '!'),
          alias($._external_expansion_sym_equal, '='),
        ),
      )),
      seq(
        // zsh: !, +, = operators on expansion body
        optional(field('operator', choice(token.immediate('!'), token.immediate('+'), token.immediate('=')))),
        choice($.variable_name, $._simple_variable_name, $._special_variable_name, $.subscript),
        choice(
          $._expansion_expression,
          $._expansion_regex,
          $._expansion_regex_replacement,
          $._expansion_regex_removal,
          $._expansion_max_length,
          $._expansion_operator,
          $._zsh_expansion_modifier,
        ),
      ),
      // zsh: \${@[idx]}, \${*[idx]} — special variable with subscript
      seq(
        $._special_variable_name,
        '[',
        field('index', choice($._literal, $.binary_expression, $.unary_expression)),
        ']',
        optional(choice(
          $._expansion_expression,
          $._expansion_regex,
          $._expansion_regex_replacement,
          $._expansion_regex_removal,
          $._expansion_max_length,
          $._expansion_operator,
          $._zsh_expansion_modifier,
        )),
      ),
      // zsh: \${#\${nested}} — length of nested expansion
      seq(
        alias($._external_expansion_sym_hash, '#'),
        choice($.expansion, $.command_substitution),
      ),
      // zsh: \${^var}, \${~var} — distribute/glob prefix operators
      seq(
        field('operator', choice(token.immediate('^'), token.immediate('~'))),
        choice($.variable_name, $._simple_variable_name, $._special_variable_name, $.subscript,
               $.expansion, $.command_substitution),
        optional(choice(
          $._expansion_expression,
          $._expansion_regex,
          $._expansion_regex_replacement,
          $._expansion_regex_removal,
          $._expansion_max_length,
          $._expansion_operator,
          $._zsh_expansion_modifier,
        )),
      ),
      // zsh: nested parameter expansion \${$\{VAR:-default}:-fallback}
      // also: \${$(cmd)#pattern}, \${"string"#pattern}, \${$\{(Az)VAR}[idx]}
      seq(
        choice($.expansion, $.command_substitution, $.string),
        // zsh: optional subscript on nested expansion like \${$\{var}[-1]}
        optional(seq('[', field('index', choice($._literal, $.binary_expression, $.unary_expression)), ']')),
        optional(choice(
          $._expansion_expression,
          $._expansion_regex,
          $._expansion_regex_replacement,
          $._expansion_regex_removal,
          $._expansion_max_length,
          $._expansion_operator,
          $._zsh_expansion_modifier,
        )),
      ),
      seq(
        field('operator', token.immediate('!')),
        choice($._simple_variable_name, $.variable_name),
        optional(field('operator', choice(
          token.immediate('@'),
          token.immediate('*'),
        ))),
      ),
      seq(
        // zsh: + added to operator set
        optional(field('operator', immediateLiterals('#', '!', '=', '+'))),
        choice(
          $.subscript,
          $._simple_variable_name,
          $._special_variable_name,
          $.command_substitution,
        ),
        repeat(field(
          'operator',
          choice(
            alias($._external_expansion_sym_hash, '#'),
            alias($._external_expansion_sym_bang, '!'),
            alias($._external_expansion_sym_equal, '='),
          ),
        )),
      ),
    ),`,
  "_expansion_body");
verify(src, /zsh_flags_expansion/, "_expansion_body");
verify(src, /_zsh_expansion_modifier/, "_expansion_body");
verify(src, /special variable with subscript/, "_expansion_body");
verify(src, /nested parameter expansion/, "_expansion_body");
verify(src, /distribute\/glob prefix/, "_expansion_body");
verify(src, /length of nested expansion/, "_expansion_body");
count++;

// ── 14. Expansion expression: add :| and :* operators ───────────────────────
src = replaceStr(src,
  "immediateLiterals('=', ':=', '-', ':-', '+', ':+', '?', ':?')",
  "immediateLiterals('=', ':=', '-', ':-', '+', ':+', '?', ':?', ':|', ':*')",
  "expansion operators :|:*");
verify(src, /':\|', ':\*'/, "expansion operators :|:*");
count++;

// ── 15. Expansion regex removal: add :# operator ───────────────────────────
src = replaceStr(src,
  "      field('operator', choice('#', alias($._immediate_double_hash, '##'), '%', '%%')),",
  "      field('operator', choice('#', alias($._immediate_double_hash, '##'), '%', '%%', ':#')),",
  "regex removal :#");
verify(src, /':#'/, "regex removal :#");
count++;

// ── 16. Insert zsh_flags_expansion rule before _expansion_expression ────────
src = insertBefore(src,
  "    _expansion_expression: $ =>",
  `    // \${(flags)name} — zsh parameter expansion flags
    zsh_flags_expansion: $ => seq(
      token.immediate('('),
      field('flags', $.zsh_expansion_flags),
      ')',
      optional(choice(
        $._simple_variable_name,
        $._special_variable_name,
        $.variable_name,
        $.subscript,
        $.string,
        $.command_substitution,
        $.expansion,
        // zsh: @ and * as target vars — use prec to beat _expansion_operator
        prec(2, alias(token.immediate('@'), $.special_variable_name)),
      )),
      optional(choice(
        $._expansion_expression,
        $._expansion_regex,
        $._expansion_regex_replacement,
        $._expansion_regex_removal,
        $._expansion_max_length,
        $._expansion_operator,
        $._zsh_expansion_modifier,
      )),
    ),

    // Flag characters (k, v, f, @, …) and delimited patterns like s:sep: or s/sep/
    zsh_expansion_flags: $ => repeat1(
      choice(
        /[a-zA-Z@#%^~0-9]/,
        /[a-zA-Z]:[^:]*:/,
        /[a-zA-Z]\\/[^/]*\\//,
      ),
    ),

    // zsh: expansion modifiers — :l :u :h :t :r :e :a :A :P :Q :N :gs/x/y/ :s/x/y/
    _zsh_expansion_modifier: $ => repeat1(choice(
      seq(':', choice('l', 'u', 'h', 't', 'r', 'e', 'a', 'A', 'P', 'Q', 'N')),
      seq(':',
        choice('gs', 's'),
        token.immediate('/'),
        optional(alias(/[^/]*/, $.regex)),
        '/',
        optional(alias(/[^/}]*/, $.regex)),
        optional('/'),
      ),
    )),

`,
  "zsh rules");
verify(src, /zsh_flags_expansion: \$ => seq/, "zsh rules");
verify(src, /zsh_expansion_flags: \$ => repeat1/, "zsh rules");
verify(src, /_zsh_expansion_modifier: \$ => repeat1/, "zsh rules");
count++;

// ── 17. Test command: remove (( )) form (handled by compound_statement)
src = replaceStr(src,
  "        seq('((', optional($._expression), '))'),\n      ),\n    ),",
  "      ),\n    ),",
  "test_command ((");
verify(src, /test_command/, "test_command ((");
count++;

// ── 18. Arithmetic expansion: (( moved to compound_statement
src = replaceStr(src,
  "      seq(choice('$((', '(('), commaSep1($._arithmetic_expression), '))'),",
  "      seq('$((', commaSep1($._arithmetic_expression), '))'),",
  "arithmetic_expansion");
verify(src, /arithmetic_expansion/, "arithmetic_expansion");
count++;

// ── 19. Arithmetic literal: add raw_string
src = replaceStr(src,
  `      $.variable_name,\n      $.string,\n    )),\n\n    _arithmetic_binary_expression`,
  `      $.variable_name,\n      $.string,\n      $.raw_string,\n    )),\n\n    _arithmetic_binary_expression`,
  "arithmetic literal raw_string");
verify(src, /raw_string.*\n.*\)\).*\n.*\n.*_arithmetic_binary/, "arithmetic literal raw_string");
count++;

// ── 20. Special variable name: remove '0'
src = replaceStr(src,
  "alias(choice('*', '@', '?', '!', '#', '-', '$', '0', '_'), $.special_variable_name)",
  "alias(choice('*', '@', '?', '!', '#', '-', '$', '_'), $.special_variable_name)",
  "special_variable_name remove 0");
verify(src, /'\$', '_'\)/, "special_variable_name remove 0");
count++;

// ── 21. &! and &| terminators ───────────────────────────────────────────────
src = replaceRe(src,
  /_terminator: _ => choice\(';', ';;', \/\\n\/, '&'\)/,
  "_terminator: _ => choice(';', ';;', /\\n/, '&', '&!', '&|')",
  "&! &| terminators");
verify(src, /'&\|'/, "&! &| terminators");
count++;

// ── Summary ─────────────────────────────────────────────────────────────────
if (src === original) {
  console.error("WARNING: no changes were made — grammar may already be patched");
}

fs.writeFileSync(file, src);
console.log(`Patched ${file} successfully (${count} patches applied)`);
