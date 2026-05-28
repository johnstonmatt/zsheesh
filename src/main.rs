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
    /// Files or directories to format. Reads from stdin if none provided.
    /// Directories are searched recursively for .zsh files.
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
}

const GREY: &str = "\x1b[90m";
const WHITE: &str = "\x1b[97m";
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
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|n| n.to_str().is_some_and(|s| s.starts_with('.')))
            {
                continue;
            }
            collect_zsh_files_recursive(&path, files)?;
        } else if path.is_file() {
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

fn run() -> Result<ExitCode, ZshFormatterError> {
    let cli = Cli::parse();
    let use_color = io::stderr().is_terminal();

    let formatter = ZshFormatter::with_indent(&cli.indent)?;

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
            let (protected_input, protected_regions) = protect_regions(&input);
            let formatted = formatter.format_str(&protected_input)?;
            let restored = restore_regions(&formatted, &protected_regions);
            if input != restored {
                return Ok(ExitCode::FAILURE);
            }
            return Ok(ExitCode::SUCCESS);
        }

        let (protected_input, protected_regions) = protect_regions(&input);
        let formatted = formatter.format_str(&protected_input)?;
        let restored = restore_regions(&formatted, &protected_regions);
        io::stdout()
            .write_all(restored.as_bytes())
            .map_err(ZshFormatterError::from)?;

        return Ok(ExitCode::SUCCESS);
    }

    let mut all_files = Vec::new();
    for path in &cli.paths {
        all_files.extend(collect_zsh_files(path)?);
    }

    let mut any_diff = false;

    for path in &all_files {
        let content = std::fs::read_to_string(path)?;

        if cli.dump_ast {
            let ast = formatter.dump_ast(&content)?;
            println!("=== {} ===", path.display());
            io::stdout()
                .write_all(ast.as_bytes())
                .map_err(ZshFormatterError::from)?;
            continue;
        }

        let (protected_input, protected_regions) = protect_regions(&content);
        let formatted = formatter.format_str(&protected_input)?;
        let restored = restore_regions(&formatted, &protected_regions);
        let changed = content != restored;

        if cli.check {
            if changed {
                print_path(&path.display().to_string(), true, use_color);
                any_diff = true;
            } else {
                print_path(&path.display().to_string(), false, use_color);
            }
            continue;
        }

        if cli.write {
            if changed {
                std::fs::write(path, &restored)?;
                print_path(&path.display().to_string(), true, use_color);
            } else {
                print_path(&path.display().to_string(), false, use_color);
            }
        } else {
            io::stdout()
                .write_all(restored.as_bytes())
                .map_err(ZshFormatterError::from)?;
        }
    }

    if cli.check && any_diff {
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

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("zsheesh: {e}");
            ExitCode::FAILURE
        }
    }
}
