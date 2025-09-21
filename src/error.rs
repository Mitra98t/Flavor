use std::cmp::max;
use std::fmt;

use crate::types::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPhase {
    #[allow(dead_code)]
    Lexing,
    Parsing,
    TypeChecking,
    Runtime,
}

#[derive(Debug, Clone)]
pub struct FlavorError {
    pub message: String,
    pub span: Option<Span>,
    pub phase: ErrorPhase,
}

impl FlavorError {
    pub fn new(message: impl Into<String>, span: Option<Span>, phase: ErrorPhase) -> Self {
        Self {
            message: message.into(),
            span,
            phase,
        }
    }

    pub fn with_span(message: impl Into<String>, span: Span, phase: ErrorPhase) -> Self {
        Self::new(message, Some(span), phase)
    }

    pub fn render(&self, source: &str) -> String {
        match self.span {
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

                format!(
                    "[{:?}] {}\n--> {}:{}\n{:>4} | {}\n     | {}",
                    self.phase,
                    self.message,
                    span.start_line,
                    span.start_column,
                    span.start_line,
                    line_text,
                    pointer_line
                )
            }
            None => format!("[{:?}] {}", self.phase, self.message),
        }
    }
}

impl fmt::Display for FlavorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FlavorError {}
