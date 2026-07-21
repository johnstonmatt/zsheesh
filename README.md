# zsheesh

A zsh-aware code formatter, written in Rust.

## Install

### Shell (macOS / Linux)

```sh
curl -fsSL https://raw.githubusercontent.com/johnstonmatt/zsheesh/main/install.sh | sh
```

Downloads a pre-built binary from [GitHub Releases](https://github.com/johnstonmatt/zsheesh/releases) for your platform.

### Cargo (build from source)

```sh
cargo install --git https://github.com/johnstonmatt/zsheesh
```

## Usage

```sh
# Format a file to stdout
zsheesh ~/.zshrc

# Format in-place
zsheesh -w ~/.zshrc

# Format from stdin
echo 'if [ -f x ]; then\necho y\nfi' | zsheesh

# Check mode (CI-friendly, exits 1 if changes needed)
zsheesh --check .

# Custom indent (default: 2 spaces)
zsheesh --indent '\t' ~/.zshrc

# Dump the AST
zsheesh --dump-ast ~/.zshrc
```

Directories are searched recursively for `.zsh`, `.zshrc`, `.zshenv`, `.zprofile`, `.zlogin`, `.zlogout` files.

### GitHub Action

```yaml
- uses: johnstonmatt/zsheesh@main
  with:
    path: .           # file(s)/dir(s) to check (default: .)
    mode: check       # 'check' (default, fails on unformatted files) or 'write'
    args: ''          # extra args, e.g. --indent '\t' or --force
    version: latest   # zsheesh release to install (default: latest)
```

Installs a released binary if one is available, otherwise builds from source. `mode: check` fails the step if any file would be reformatted; `mode: write` formats in place (e.g. to auto-fix and commit).

### Escape hatches

```zsh
# fmt: skip
weird_alignment_line_preserved_as_is

# fmt: off
this block
  is not
    formatted
# fmt: on
```

### Safe by default

Files with parse errors are skipped to prevent destructive formatting. Use `--force` to override.

## Why

There is no working zsh formatter today. The de facto shell formatter, [shfmt](https://github.com/mvdan/sh), explicitly rejects zsh — its maintainer has closed every zsh-support issue because zsh's grammar diverges from bash/POSIX and has no formal spec. Every other shell formatter (beautysh, shellharden, prettier-plugin-sh) defers to bash semantics and breaks on the same constructs.

This matters because:

- macOS has shipped zsh as the default shell since 2019 (Catalina), driven by Apple's avoidance of GPLv3 bash. The tooling ecosystem did not follow.
- Real zsh configs use constructs bash parsers cannot represent: parameter-expansion flags (`${(k)var}`, `${(@)var}`), glob qualifiers (`*(.om[1])`), anonymous functions, `typeset -gA`, `setopt`, `print -r --`.
- The community workaround — "write interactive config in zsh, scripts in bash" — leaves interactive configs hand-formatted, stylistically divergent, and untooled.

## How it works

zsheesh = a dedicated tree-sitter grammar + the Topiary formatter framework, glued together in Rust.

| Layer | Tool | Why |
|---|---|---|
| Parser | [`tree-sitter-zsh`](tree-sitter-zsh/) | Forked from tree-sitter-bash, extended for zsh-only constructs |
| Format rules | [Topiary](https://github.com/tweag/topiary) | Tree-sitter-native formatter; rules written as Scheme queries |
| CLI | Rust + [clap](https://docs.rs/clap) | IO, config, the parse → format → emit pipeline |

### tree-sitter-zsh

The grammar is 99% tree-sitter-bash with a small set of zsh-specific patches applied mechanically. See [`tree-sitter-zsh/README.md`](tree-sitter-zsh/README.md) for details on upgrading when tree-sitter-bash releases a new version.

## Project structure

```
zsheesh/
├── src/
│   ├── main.rs              CLI entry point
│   ├── lib.rs               public API
│   ├── format.rs            ZshFormatter (tree-sitter + Topiary)
│   └── escape.rs            # fmt: skip / off / on directives
├── queries/zsh/
│   └── formatting.scm       Topiary formatting rules
├── corpus/                   test fixture zsh files
├── tests/
│   ├── format_test.rs        unit tests (formatting correctness)
│   ├── cli_test.rs           CLI integration tests
│   └── e2e_test.rs           end-to-end tests
└── tree-sitter-zsh/
    ├── grammar.js            generated from tree-sitter-bash + patches
    ├── patch-grammar.js      zsh-specific grammar patches
    ├── sync-from-bash.sh     rebuild grammar from a new bash version
    └── src/                  generated C parser
```

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
