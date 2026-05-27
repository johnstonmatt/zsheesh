# zsheesh

A zsh-aware code formatter, written in Rust.

## Why

There is no working zsh formatter today. The de facto shell formatter, [shfmt](https://github.com/mvdan/sh), explicitly rejects zsh — its maintainer has closed every zsh-support issue because zsh's grammar diverges from bash/POSIX and has no formal spec. Every other shell formatter (beautysh, shellharden, prettier-plugin-sh) defers to bash semantics and breaks on the same constructs.

This matters because:

- macOS has shipped zsh as the default shell since 2019 (Catalina), driven by Apple's avoidance of GPLv3 bash. The tooling ecosystem did not follow.
- Real zsh configs use constructs bash parsers cannot represent: parameter-expansion flags (`${(k)var}`, `${(@)var}`), glob qualifiers (`*(.om[1])`), anonymous functions, `typeset -gA`, `setopt`, `print -r --`.
- The community workaround — "write interactive config in zsh, scripts in bash" — leaves interactive configs hand-formatted, stylistically divergent, and untooled.

## How

zsheesh = forked tree-sitter grammar + the Topiary formatter framework, glued together in Rust.

### Stack

| Layer | Tool | Why |
|---|---|---|
| Parser | `tree-sitter-zsh` (forked from `tree-sitter-bash`) | tree-sitter-bash already handles ~80% of the shared grammar; extend it for zsh-only constructs |
| Format rules | [Topiary](https://github.com/tweag/topiary) | Tree-sitter-native formatter framework; rules written as Scheme queries against the AST |
| CLI / glue | Rust | Owns IO, config loading, the parse-format-emit pipeline |

### Plan

1. **Build a corpus first.** Before extending the grammar, collect real zsh in `corpus/`: oh-my-zsh, prezto, zsh-users/* plugins, the zsh source tree's `Test/` directory, contributors' own dotfiles. Real input surfaces grammar gaps faster than reading the manual.
2. **Fork the grammar.** `tree-sitter-bash` → `tree-sitter-zsh`. Add nodes for zsh-only constructs. Cross-reference zsh's own parser (`Src/parse.c`, `Src/lex.c` in the [zsh source](https://sourceforge.net/p/zsh/code/ci/master/tree/Src/)) when ambiguities arise.
3. **Reach "parses corpus."** Milestone: every file in `corpus/` parses without error.
4. **Add formatting queries.** Start narrow — indentation, `case` arm structure, `if`/`while`/`for` blocks, function bodies, heredoc preservation. Emit expansion-flag bodies verbatim; do not try to format inside `${(...)…}`.
5. **Ship `# fmt: skip` and `# fmt: off`/`# fmt: on` early.** Real zsh files contain intentionally weird alignment. Opt-out has to exist before users get burned.
6. **Reuse shfmt's style defaults** (`-i 2 -ci -bn`). Do not reinvent style; people are calibrated to shfmt.

## Project structure

```
zsheesh/
  Cargo.toml             # crate manifest
  src/main.rs            # CLI entry (currently stubbed)
  tree-sitter-zsh/       # forked grammar              (TODO)
  queries/zsh/           # Topiary formatting rules    (TODO)
  corpus/                # test inputs                 (TODO)
  tests/snapshots/       # insta golden files          (TODO)
```

Today only `Cargo.toml` and `src/main.rs` exist. Dependencies are pulled (`tree-sitter`, `tree-sitter-bash`, `topiary-core`) and the project builds.

## Getting started

```sh
git clone <repo>
cd zsheesh
cargo build
```

First meaningful task for a new contributor: wire `src/main.rs` to take a path, read the file, parse it with `tree-sitter-bash` (placeholder until the fork exists), and pretty-print the AST. That's the smallest end-to-end that validates the toolchain before any grammar work.

## Success criteria

The tool ships when, against a corpus of real-world zsh (oh-my-zsh, prezto, zsh-users/* plugins):

1. **Parses 100% of the corpus** without errors.
2. **Roundtrips identically** — running zsheesh on its own output produces no diff (the formatter is a fixed point).
3. **Handles the zsh constructs that break shfmt**, specifically:
   - Parameter expansion flags: `${(k)v}`, `${(@)v}`, `${(P)v}`, `${(kv)hash}`
   - Glob qualifiers: `*(.om[1])`, `**/*(N)`
   - Anonymous functions: `() { ... } "$@"`
   - `typeset -gA`, `typeset -U`, `setopt`, `print -r --`
   - `[[ ... =~ ... ]]` with zsh regex-match semantics
4. **Has a tested escape hatch** (`# fmt: skip`, `# fmt: off`/`# fmt: on`).
5. **Formats `~/.zshrc` and `~/.config/zsh/*.zsh` cleanly** — the dogfooding bar.

### Out of scope for v1

Listed so the next maintainer doesn't get nerd-sniped:

- LSP server / editor integrations
- `--check` mode for CI
- Configurable style (shfmt-equivalent defaults are enough for v1)
- Performance optimization (correctness first)
