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
}

impl fmt::Display for ZshFormatterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Formatter(e) => write!(f, "Formatter error: {e}"),
            Self::Query(e) => write!(f, "Query error: {e}"),
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
    tolerate_parsing_errors: bool,
}

impl ZshFormatter {
    pub fn new() -> Result<Self, ZshFormatterError> {
        Self::with_indent("  ")
    }

    pub fn with_indent(indent: &str) -> Result<Self, ZshFormatterError> {
        let grammar = topiary_tree_sitter_facade::Language::from(tree_sitter_bash::LANGUAGE);
        let query = TopiaryQuery::new(&grammar, ZSH_QUERY)
            .map_err(|e| ZshFormatterError::Query(e.to_string()))?;
        let language = Language {
            name: "zsh".to_owned(),
            query,
            grammar,
            indent: Some(indent.to_owned()),
        };
        Ok(Self {
            language,
            tolerate_parsing_errors: true,
        })
    }

    pub fn tolerate_parsing_errors(mut self, yes: bool) -> Self {
        self.tolerate_parsing_errors = yes;
        self
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
                tolerate_parsing_errors: self.tolerate_parsing_errors,
            },
        )?;
        Ok(String::from_utf8(output).expect("topiary produces valid UTF-8"))
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
