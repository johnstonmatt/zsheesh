use std::io::{self, Read, Write};
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
    /// Files to format. Reads from stdin if none provided.
    #[arg()]
    files: Vec<PathBuf>,

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

fn run() -> Result<ExitCode, ZshFormatterError> {
    let cli = Cli::parse();

    let formatter = ZshFormatter::with_indent(&cli.indent)?;

    if cli.files.is_empty() {
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

    let mut any_diff = false;

    for path in &cli.files {
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

        if cli.check {
            if content != restored {
                eprintln!("{}: would reformat", path.display());
                any_diff = true;
            }
            continue;
        }

        if cli.write {
            if content != restored {
                std::fs::write(path, &restored)?;
                eprintln!("{}: formatted", path.display());
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

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("zsheesh: {e}");
            ExitCode::FAILURE
        }
    }
}
