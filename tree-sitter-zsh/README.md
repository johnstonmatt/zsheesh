# tree-sitter-zsh

Zsh grammar for [tree-sitter](https://tree-sitter.github.io/), mechanically
derived from [tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash).

## How it works

The grammar is 99% tree-sitter-bash with a small set of zsh-specific patches
applied on top:

| Patch | Purpose |
|---|---|
| `zsh_flags_expansion` rule | Parse `${(k)var}`, `${(v)var}`, `${(@)array}`, etc. |
| `zsh_expansion_flags` rule | Match flag characters and `key:sep:` patterns |
| `&!` terminator | Background + disown operator |
| `name: 'zsh'` | Grammar identity |

The patches live in `patch-grammar.js` and are applied by `sync-from-bash.sh`.

## Upgrading to a new tree-sitter-bash version

```bash
# Point at the new bash grammar source (npm, cargo, or a local checkout)
./sync-from-bash.sh /path/to/tree-sitter-bash

# Verify
cd .. && cargo test
```

The script:
1. Copies `grammar.js` from tree-sitter-bash
2. Runs `patch-grammar.js` to apply the zsh additions
3. Runs `tree-sitter generate` to rebuild the parser
4. Copies `scanner.c` and renames `tree_sitter_bash_*` symbols to `tree_sitter_zsh_*`

If the upstream grammar changed in a way that breaks a patch anchor (e.g.
`_expansion_body` was renamed), `patch-grammar.js` will fail with a clear
error message telling you which anchor was not found.

## Requirements

- [tree-sitter CLI](https://tree-sitter.github.io/tree-sitter/creating-parsers/tool-overview.html) (`cargo install tree-sitter-cli`)
- Node.js (for `patch-grammar.js`)
