use std::fmt;
use std::io;
use std::path::Path;

use topiary_core::{FormatterError, Language, Operation, TopiaryQuery, Visualisation, formatter};

const ZSH_QUERY: &str = include_str!("../queries/zsh/formatting.scm");

#[derive(Debug)]
pub enum ZshFormatterError {
    Io(io::Error),
    Formatter(FormatterError),
    Query(String),
    ParseError(Vec<ParseErrorInfo>),
}

#[derive(Debug, Clone)]
pub struct ParseErrorInfo {
    pub line: usize,
    pub column: usize,
    pub text: String,
}

impl fmt::Display for ZshFormatterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Formatter(e) => write!(f, "Formatter error: {e}"),
            Self::Query(e) => write!(f, "Query error: {e}"),
            Self::ParseError(errors) => {
                write!(f, "parse errors detected ({} error(s)):", errors.len())?;
                for e in errors {
                    write!(f, "\n  line {}:{}: {}", e.line, e.column, e.text)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ZshFormatterError {}

impl From<io::Error> for ZshFormatterError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<FormatterError> for ZshFormatterError {
    fn from(e: FormatterError) -> Self {
        Self::Formatter(e)
    }
}

pub struct ZshFormatter {
    language: Language,
}

impl ZshFormatter {
    pub fn new() -> Result<Self, ZshFormatterError> {
        Self::with_indent("  ")
    }

    pub fn with_indent(indent: &str) -> Result<Self, ZshFormatterError> {
        let grammar = topiary_tree_sitter_facade::Language::from(tree_sitter_zsh::LANGUAGE);
        let query = TopiaryQuery::new(&grammar, ZSH_QUERY)
            .map_err(|e| ZshFormatterError::Query(e.to_string()))?;
        let language = Language {
            name: "zsh".to_owned(),
            query,
            grammar,
            indent: Some(indent.to_owned()),
        };
        Ok(Self { language })
    }

    pub fn check_parse_errors(&self, input: &str) -> Vec<ParseErrorInfo> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_zsh::LANGUAGE.into())
            .expect("zsh grammar is valid");

        let Some(tree) = parser.parse(input, None) else {
            return vec![ParseErrorInfo {
                line: 1,
                column: 0,
                text: "failed to parse".to_owned(),
            }];
        };

        let root = tree.root_node();
        if !root.has_error() {
            return vec![];
        }

        let lines: Vec<&str> = input.lines().collect();
        let mut errors = Vec::new();
        collect_errors(root, &lines, &mut errors);
        errors
    }

    /// Returns line ranges (0-indexed, inclusive) of top-level statements that
    /// contain parse errors. Used for auto-protecting unparseable regions.
    pub fn error_line_ranges(&self, input: &str) -> Vec<(usize, usize)> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_zsh::LANGUAGE.into())
            .expect("zsh grammar is valid");

        let Some(tree) = parser.parse(input, None) else {
            let line_count = input.lines().count();
            return vec![(0, line_count.saturating_sub(1))];
        };

        let root = tree.root_node();
        if !root.has_error() {
            return vec![];
        }

        let mut ranges = Vec::new();
        collect_error_ranges(root, &mut ranges);

        // Merge overlapping ranges
        ranges.sort();
        let mut merged: Vec<(usize, usize)> = Vec::new();
        for (start, end) in ranges {
            if let Some(last) = merged.last_mut()
                && start <= last.1 + 1
            {
                last.1 = last.1.max(end);
                continue;
            }
            merged.push((start, end));
        }
        merged
    }

    pub fn format_str(&self, input: &str) -> Result<String, ZshFormatterError> {
        let mut output = Vec::new();
        let mut reader = input.as_bytes();
        formatter(
            &mut reader,
            &mut output,
            &self.language,
            Operation::Format {
                skip_idempotence: false,
                tolerate_parsing_errors: true,
            },
        )?;
        Ok(String::from_utf8(output).expect("topiary produces valid UTF-8"))
    }

    pub fn safe_format_str(&self, input: &str) -> Result<String, ZshFormatterError> {
        let errors = self.check_parse_errors(input);
        if !errors.is_empty() {
            return Err(ZshFormatterError::ParseError(errors));
        }
        self.format_str(input)
    }

    pub fn format_file(&self, path: &Path) -> Result<String, ZshFormatterError> {
        let content = std::fs::read_to_string(path)?;
        self.format_str(&content)
    }

    pub fn check_str(&self, input: &str) -> Result<bool, ZshFormatterError> {
        let formatted = self.format_str(input)?;
        Ok(input == formatted)
    }

    pub fn check_file(&self, path: &Path) -> Result<bool, ZshFormatterError> {
        let content = std::fs::read_to_string(path)?;
        self.check_str(&content)
    }

    pub fn dump_ast(&self, input: &str) -> Result<String, ZshFormatterError> {
        let mut output = Vec::new();
        let mut reader = input.as_bytes();
        formatter(
            &mut reader,
            &mut output,
            &self.language,
            Operation::Visualise {
                output_format: Visualisation::Json,
            },
        )?;
        Ok(String::from_utf8(output).expect("topiary produces valid UTF-8"))
    }
}

impl Default for ZshFormatter {
    fn default() -> Self {
        Self::new().expect("default formatter should initialize")
    }
}

fn collect_error_ranges(node: tree_sitter::Node, ranges: &mut Vec<(usize, usize)>) {
    if node.is_error() || node.is_missing() {
        // Walk up to the nearest top-level statement (child of `program`)
        let mut ancestor = node;
        while let Some(parent) = ancestor.parent() {
            if parent.kind() == "program" {
                break;
            }
            ancestor = parent;
        }
        let start = ancestor.start_position().row;
        let end = ancestor.end_position().row;
        ranges.push((start, end));
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_error_ranges(child, ranges);
    }
}

fn collect_errors(node: tree_sitter::Node, lines: &[&str], errors: &mut Vec<ParseErrorInfo>) {
    if node.is_error() || node.is_missing() {
        let start = node.start_position();
        let line = start.row + 1;
        let col = start.column;
        let text = lines
            .get(start.row)
            .map(|l| l.trim().to_owned())
            .unwrap_or_default();
        errors.push(ParseErrorInfo {
            line,
            column: col,
            text,
        });
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_errors(child, lines, errors);
    }
}
