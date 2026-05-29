use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use zsheesh::{ZshFormatter, ZshFormatterError, protect_regions, restore_regions};

#[derive(Parser)]
#[command(
    name = "zsheesh",
    version,
    about = "A zsh-aware code formatter",
    long_about = "zsheesh formats zsh scripts using tree-sitter parsing and \
                  Topiary-based formatting rules. It handles constructs that \
                  break other shell formatters: parameter expansion flags, \
                  glob qualifiers, anonymous functions, and more."
)]
struct Cli {
    /// Files or directories to format. Defaults to current directory when
    /// run interactively; reads from stdin when piped.
    #[arg()]
    paths: Vec<PathBuf>,

    /// Check if files are formatted without modifying them.
    /// Exits with code 1 if any file would change.
    #[arg(long)]
    check: bool,

    /// Write formatted output back to the file(s) in place.
    /// Without this flag, formatted output goes to stdout.
    #[arg(short = 'w', long)]
    write: bool,

    /// Dump the parsed AST as JSON instead of formatting.
    #[arg(long)]
    dump_ast: bool,

    /// Indentation string (default: two spaces).
    #[arg(long, default_value = "  ")]
    indent: String,

    /// Format even when parse errors are detected.
    /// By default, files with parse errors are skipped to prevent destructive formatting.
    #[arg(long)]
    force: bool,
}

const GREY: &str = "\x1b[90m";
const WHITE: &str = "\x1b[97m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

fn collect_zsh_files(path: &PathBuf) -> Result<Vec<PathBuf>, ZshFormatterError> {
    let mut files = Vec::new();
    if path.is_file() {
        files.push(path.clone());
    } else if path.is_dir() {
        collect_zsh_files_recursive(path, &mut files)?;
        files.sort();
    } else {
        return Err(ZshFormatterError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{}: No such file or directory", path.display()),
        )));
    }
    Ok(files)
}

fn collect_zsh_files_recursive(
    dir: &PathBuf,
    files: &mut Vec<PathBuf>,
) -> Result<(), ZshFormatterError> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            continue;
        }
        let path = entry.path();
        if ft.is_dir() {
            let skip = path
                .file_name()
                .is_some_and(|n| matches!(n.to_str(), Some(".git" | ".svn" | ".hg")));
            if skip {
                continue;
            }
            collect_zsh_files_recursive(&path, files)?;
        } else if ft.is_file() {
            let ext = path.extension().and_then(|e| e.to_str());
            let name = path.file_name().and_then(|n| n.to_str());
            if ext == Some("zsh")
                || name == Some(".zshrc")
                || name == Some(".zshenv")
                || name == Some(".zprofile")
                || name == Some(".zlogin")
                || name == Some(".zlogout")
            {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn format_input(
    formatter: &ZshFormatter,
    input: &str,
    force: bool,
) -> Result<Option<String>, ZshFormatterError> {
    let (protected_input, protected_regions) = protect_regions(input);

    if !force {
        let errors = formatter.check_parse_errors(&protected_input);
        if !errors.is_empty() {
            return Err(ZshFormatterError::ParseError(errors));
        }
    }

    let formatted = formatter.format_str(&protected_input)?;
    let restored = restore_regions(&formatted, &protected_regions);
    Ok(Some(restored))
}

fn run() -> Result<ExitCode, ZshFormatterError> {
    let cli = Cli::parse();
    let use_color = io::stderr().is_terminal();

    let formatter = ZshFormatter::with_indent(&cli.indent)?;

    if cli.paths.is_empty() && io::stdin().is_terminal() {
        // No paths and stdin is a terminal: default to current directory
        let all_files = collect_zsh_files(&PathBuf::from("."))?;
        if all_files.is_empty() {
            eprintln!("No zsh files found in current directory.");
            return Ok(ExitCode::SUCCESS);
        }
        // Fall through to the file-processing loop below
        return process_files(&formatter, &all_files, &cli, use_color);
    }

    if cli.paths.is_empty() {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(ZshFormatterError::from)?;

        if cli.dump_ast {
            let ast = formatter.dump_ast(&input)?;
            io::stdout()
                .write_all(ast.as_bytes())
                .map_err(ZshFormatterError::from)?;
            return Ok(ExitCode::SUCCESS);
        }

        if cli.check {
            match format_input(&formatter, &input, cli.force) {
                Ok(Some(restored)) => {
                    if input != restored {
                        return Ok(ExitCode::FAILURE);
                    }
                }
                Err(ZshFormatterError::ParseError(errors)) => {
                    print_parse_errors("<stdin>", &errors, use_color);
                    return Ok(ExitCode::FAILURE);
                }
                Err(e) => return Err(e),
                Ok(None) => {}
            }
            return Ok(ExitCode::SUCCESS);
        }

        match format_input(&formatter, &input, cli.force) {
            Ok(Some(restored)) => {
                io::stdout()
                    .write_all(restored.as_bytes())
                    .map_err(ZshFormatterError::from)?;
            }
            Err(ZshFormatterError::ParseError(errors)) => {
                print_parse_errors("<stdin>", &errors, use_color);
                return Ok(ExitCode::FAILURE);
            }
            Err(e) => return Err(e),
            Ok(None) => {}
        }

        return Ok(ExitCode::SUCCESS);
    }

    let mut all_files = Vec::new();
    for path in &cli.paths {
        all_files.extend(collect_zsh_files(path)?);
    }

    process_files(&formatter, &all_files, &cli, use_color)
}

fn process_files(
    formatter: &ZshFormatter,
    all_files: &[PathBuf],
    cli: &Cli,
    use_color: bool,
) -> Result<ExitCode, ZshFormatterError> {
    let mut any_diff = false;
    let mut any_errors = false;

    for path in all_files {
        let content = std::fs::read_to_string(path)?;
        let display = path.display().to_string();

        if cli.dump_ast {
            let ast = formatter.dump_ast(&content)?;
            println!("=== {display} ===");
            io::stdout()
                .write_all(ast.as_bytes())
                .map_err(ZshFormatterError::from)?;
            continue;
        }

        let result = format_input(formatter, &content, cli.force);

        match result {
            Err(ZshFormatterError::ParseError(errors)) => {
                print_parse_errors(&display, &errors, use_color);
                any_errors = true;
                continue;
            }
            Err(e) => return Err(e),
            Ok(None) => continue,
            Ok(Some(restored)) => {
                let changed = content != restored;

                if cli.check {
                    print_path(&display, changed, use_color);
                    if changed {
                        any_diff = true;
                    }
                    continue;
                }

                if cli.write {
                    if changed {
                        std::fs::write(path, &restored)?;
                    }
                    print_path(&display, changed, use_color);
                } else {
                    io::stdout()
                        .write_all(restored.as_bytes())
                        .map_err(ZshFormatterError::from)?;
                }
            }
        }
    }

    if any_errors || (cli.check && any_diff) {
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn print_path(path: &str, changed: bool, use_color: bool) {
    if use_color {
        if changed {
            eprintln!("{WHITE}{path}{RESET}");
        } else {
            eprintln!("{GREY}{path}{RESET}");
        }
    } else {
        eprintln!("{path}");
    }
}

fn print_parse_errors(path: &str, errors: &[zsheesh::ParseErrorInfo], use_color: bool) {
    if use_color {
        eprintln!("{YELLOW}{path}{RESET}: skipped (parse errors)");
        for e in errors {
            eprintln!("  {YELLOW}line {}:{}{RESET}: {}", e.line, e.column, e.text);
        }
    } else {
        eprintln!("{path}: skipped (parse errors)");
        for e in errors {
            eprintln!("  line {}:{}: {}", e.line, e.column, e.text);
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("zsheesh: {e}");
            ExitCode::FAILURE
        }
    }
}
