use colored::*;
use std::fmt::Display;

use crate::types::Span;
use std::cmp::max;

#[derive(Debug, Clone)]
pub enum ErrorPhase {
    Lexing,
    Parsing,
    TypeChecking,
    Runtime,
}

impl Display for ErrorPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorPhase::Lexing => write!(f, "Lexing"),
            ErrorPhase::Parsing => write!(f, "Parsing"),
            ErrorPhase::TypeChecking => write!(f, "TypeChecking"),
            ErrorPhase::Runtime => write!(f, "Runtime"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlavorError {
    pub phase: ErrorPhase,
    pub message: String,
    pub span: Option<Span>,
}

impl FlavorError {
    pub fn new(phase: ErrorPhase, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            phase,
            message: message.into(),
            span,
        }
    }

    pub fn with_span(phase: ErrorPhase, message: impl Into<String>, span: Span) -> Self {
        Self::new(phase, message, Some(span))
    }

    pub fn render(&self, source: &str) -> String {
        let phase = format!("[{}]", self.phase);
        match &self.span {
            Some(span) => {
                let line_index = span.start_line.saturating_sub(1);
                let lines: Vec<&str> = source.lines().collect();
                let line_text = lines.get(line_index).copied().unwrap_or("");
                let line_length = line_text.chars().count();

                let mut pointer_offset = span.start_column.saturating_sub(1);
                if pointer_offset > line_length {
                    pointer_offset = line_length;
                }

                let raw_width = if span.start_line == span.end_line {
                    span.end_column.saturating_sub(span.start_column) + 1
                } else {
                    1
                };
                let available = line_length.saturating_sub(pointer_offset);
                let highlight_width = max(1, raw_width.min(available.max(1)));

                let pointer_line = format!(
                    "{space}{marker}",
                    space = " ".repeat(pointer_offset),
                    marker = "^".repeat(highlight_width)
                );

                let line_pre_error = if span.start_column >= 2 {
                    &line_text[0..(span.start_column - 1).min(line_text.len())]
                } else {
                    ""
                };
                let line_post_error = if span.end_column - 1 < line_text.len() {
                    &line_text[(span.end_column - 1).min(line_text.len())..]
                } else {
                    ""
                };

                format!(
                    "\n\n{} {}\n--> {}:{}\n{:>4} | {}{}{}\n     | {}\n\n",
                    phase.yellow().bold(),
                    self.message.yellow(),
                    span.start_line,
                    span.start_column,
                    span.start_line,
                    line_pre_error,
                    line_text
                        .chars()
                        .skip(pointer_offset)
                        .take(highlight_width)
                        .collect::<String>()
                        .red(),
                    line_post_error,
                    pointer_line.red()
                )
            }
            None => format!("\n\n{} {}\n\n", phase.yellow().bold(), self.message),
        }
    }
}

impl std::fmt::Display for FlavorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FlavorError {}
