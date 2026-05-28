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
// Patch definitions
// ---------------------------------------------------------------------------
// Each patch is one of:
//   { type: "replace", find: <string|RegExp>, replacement: <string> }
//   { type: "insert_after", anchor: <string>, content: <string> }
//   { type: "insert_before", anchor: <string>, content: <string> }
//
// verify: regex that must match after the patch is applied.

const ZSH_RULES = `\
    // \${(flags)name} — zsh parameter expansion flags
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

const patches = [
  {
    name: "header",
    type: "replace",
    find: /\* @file Bash grammar for tree-sitter/,
    replacement: "* @file Zsh grammar for tree-sitter (forked from tree-sitter-bash)",
    verify: /Zsh grammar for tree-sitter/,
  },
  {
    name: "grammar name",
    type: "replace",
    find: /name: 'bash'/,
    replacement: "name: 'zsh'",
    verify: /name: 'zsh'/,
  },
  {
    name: "zsh_flags_expansion choice",
    type: "insert_after",
    anchor: "_expansion_body: $ => choice(",
    content:
      "\n      // Zsh parameter expansion flags: ${(flags)name}\n" +
      "      $.zsh_flags_expansion,",
    verify: /\$\.zsh_flags_expansion/,
  },
  {
    name: "zsh grammar rules",
    type: "insert_before",
    anchor: "    _expansion_expression: $ =>",
    content: ZSH_RULES,
    verify: /zsh_flags_expansion: \$ => seq/,
  },
  {
    name: "&! terminator",
    type: "replace",
    find: /_terminator: _ => choice\(';', ';;', \/\\n\/, '&'\)/,
    replacement: "_terminator: _ => choice(';', ';;', /\\n/, '&', '&!')",
    verify: /'&!'/,
  },
];

// ---------------------------------------------------------------------------
// Apply
// ---------------------------------------------------------------------------

const file = process.argv[2];
if (!file) {
  console.error("Usage: node patch-grammar.js <grammar.js>");
  process.exit(1);
}

let src = fs.readFileSync(file, "utf8");
const original = src;

for (const patch of patches) {
  switch (patch.type) {
    case "replace": {
      const before = src;
      src = src.replace(patch.find, patch.replacement);
      if (src === before) {
        console.error(`PATCH FAILED [${patch.name}]: pattern not found`);
        process.exit(1);
      }
      break;
    }
    case "insert_after": {
      const idx = src.indexOf(patch.anchor);
      if (idx === -1) {
        console.error(`PATCH FAILED [${patch.name}]: anchor not found: ${patch.anchor}`);
        process.exit(1);
      }
      const pos = idx + patch.anchor.length;
      src = src.slice(0, pos) + patch.content + src.slice(pos);
      break;
    }
    case "insert_before": {
      const idx = src.indexOf(patch.anchor);
      if (idx === -1) {
        console.error(`PATCH FAILED [${patch.name}]: anchor not found: ${patch.anchor}`);
        process.exit(1);
      }
      src = src.slice(0, idx) + patch.content + src.slice(idx);
      break;
    }
    default:
      console.error(`Unknown patch type: ${patch.type}`);
      process.exit(1);
  }

  if (!patch.verify.test(src)) {
    console.error(`PATCH VERIFY FAILED [${patch.name}]`);
    process.exit(1);
  }
}

if (src === original) {
  console.error("WARNING: no changes were made — grammar may already be patched");
}

fs.writeFileSync(file, src);

const added = src.split("\n").length - original.split("\n").length;
console.log(`Patched ${file} (+${added} lines, ${patches.length} patches applied)`);
