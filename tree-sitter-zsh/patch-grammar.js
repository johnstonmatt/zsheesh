#!/usr/bin/env node
//
// patch-grammar.js — apply zsh-specific changes to a tree-sitter-bash grammar.js
//
// Called by sync-from-bash.sh. Reads the file path from argv[2], patches in
// place. Each transformation targets a specific, stable anchor in the bash
// grammar so that minor upstream edits (new rules, comments, whitespace) don't
// break the patch.

"use strict";

const fs = require("fs");
const path = require("path");

const file = process.argv[2];
if (!file) {
  console.error("Usage: node patch-grammar.js <grammar.js>");
  process.exit(1);
}

let src = fs.readFileSync(file, "utf8");
const original = src;

// ---------------------------------------------------------------------------
// 1. Header: Bash → Zsh
// ---------------------------------------------------------------------------
src = src.replace(
  /\* @file Bash grammar for tree-sitter/,
  "* @file Zsh grammar for tree-sitter (forked from tree-sitter-bash)"
);

// ---------------------------------------------------------------------------
// 2. Grammar name: 'bash' → 'zsh'
// ---------------------------------------------------------------------------
src = src.replace(/name: 'bash'/, "name: 'zsh'");

// ---------------------------------------------------------------------------
// 3. Add $.zsh_flags_expansion as first choice in _expansion_body
//    Anchor: "_expansion_body: $ => choice("
// ---------------------------------------------------------------------------
const expansionBodyAnchor = "_expansion_body: $ => choice(";
const idx3 = src.indexOf(expansionBodyAnchor);
if (idx3 === -1) {
  console.error("PATCH FAILED: could not find _expansion_body anchor");
  process.exit(1);
}
const insertAt = idx3 + expansionBodyAnchor.length;
src =
  src.slice(0, insertAt) +
  "\n      // Zsh parameter expansion flags: ${(flags)name}\n" +
  "      $.zsh_flags_expansion," +
  src.slice(insertAt);

// ---------------------------------------------------------------------------
// 4. Add zsh_flags_expansion and zsh_expansion_flags rules
//    Anchor: the line "_expansion_expression: $ =>"
//    We insert the new rules right before that line.
// ---------------------------------------------------------------------------
const expansionExprAnchor = "    _expansion_expression: $ =>";
const idx4 = src.indexOf(expansionExprAnchor);
if (idx4 === -1) {
  console.error("PATCH FAILED: could not find _expansion_expression anchor");
  process.exit(1);
}

const zshRules = `    // \${(flags)name} — zsh parameter expansion flags
    // flags: single letters like k, v, f, o, O, U, L, C, @
    //        or with separators like j:sep: or s:sep:
    zsh_flags_expansion: $ => seq(
      token.immediate('('),
      field('flags', $.zsh_expansion_flags),
      ')',
      choice(
        $._simple_variable_name,
        $._special_variable_name,
        $.variable_name,
        $.subscript,
      ),
      optional(choice(
        $._expansion_expression,
        $._expansion_regex,
        $._expansion_regex_replacement,
        $._expansion_regex_removal,
        $._expansion_max_length,
        $._expansion_operator,
      )),
    ),

    zsh_expansion_flags: $ => repeat1(
      choice(
        /[a-zA-Z@#%^~]/,
        seq(/[a-zA-Z]/, ':', /[^:]*/, ':'),
      ),
    ),

`;

src = src.slice(0, idx4) + zshRules + src.slice(idx4);

// ---------------------------------------------------------------------------
// 5. Add '&!' to _terminator
//    Anchor: "_terminator: _ => choice(';', ';;', /\\n/, '&')"
// ---------------------------------------------------------------------------
src = src.replace(
  /_terminator: _ => choice\(';', ';;', \/\\n\/, '&'\)/,
  "_terminator: _ => choice(';', ';;', /\\n/, '&', '&!')"
);

// ---------------------------------------------------------------------------
// Verify all patches applied
// ---------------------------------------------------------------------------
const checks = [
  ["header", /Zsh grammar for tree-sitter/],
  ["name", /name: 'zsh'/],
  ["zsh_flags_expansion ref", /\$\.zsh_flags_expansion/],
  ["zsh_flags_expansion rule", /zsh_flags_expansion: \$ => seq/],
  ["zsh_expansion_flags rule", /zsh_expansion_flags: \$ => repeat1/],
  ["&! terminator", /'&!'/],
];

let ok = true;
for (const [label, re] of checks) {
  if (!re.test(src)) {
    console.error(`PATCH VERIFY FAILED: ${label} not found after patching`);
    ok = false;
  }
}

if (!ok) {
  process.exit(1);
}

if (src === original) {
  console.error("WARNING: no changes were made — grammar may already be patched");
}

fs.writeFileSync(file, src);

const added = src.split("\n").length - original.split("\n").length;
console.log(`Patched ${file} (+${added} lines, ${checks.length} checks passed)`);
