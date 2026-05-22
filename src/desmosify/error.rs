use super::*;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Span {
    pub source_id: usize,
    pub start_index: usize,
    pub length: usize,
}

pub struct SpanContext {
    pub line_number: usize,
    pub column_number: usize,
    pub content: String,
}

impl Span {
    pub fn tail_point(&self) -> Self {
        Self {
            source_id: self.source_id,
            start_index: self.start_index + self.length,
            length: 0,
        }
    }

    pub fn expand_to(&self, end_span: Self) -> Self {
        if self.source_id != end_span.source_id {
            panic!("source IDs do not match");
        }

        Self {
            source_id: self.source_id,
            start_index: self.start_index,
            length: end_span.start_index.checked_add(end_span.length)
                .and_then(|end_index| end_index.checked_sub(self.start_index))
                .expect("end span comes before start span"),
        }
    }

    pub fn load_context(&self, path: impl AsRef<Path>) -> std::io::Result<SpanContext> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut line_number = 1;
        let mut column_number = 0;
        let mut line_start_index = 0;
        let mut content = String::new();
        let mut line = String::new();

        while reader.read_line(&mut line)? > 0 {
            let line_end_index = line_start_index + line.len();

            if line_end_index > self.start_index {
                if column_number == 0 {
                    column_number = self.start_index - line_start_index + 1;
                }

                if line_start_index < self.start_index + self.length {
                    let line_trim = line.trim_end();

                    // Write the line from the source file
                    content.push('\t');
                    content.push_str(line_trim);
                    content.push('\n');

                    // Write the span markers on the line below
                    content.push('\t');
                    content.extend(std::iter::repeat_n(
                        ' ',
                        self.start_index.saturating_sub(line_start_index),
                    ));
                    if self.length == 0 {
                        content.push('^');
                    }
                    else {
                        content.extend(std::iter::repeat_n(
                            '~',
                            (line_start_index + line_trim.len())
                                .min(self.start_index + self.length)
                                .saturating_sub(line_start_index.max(self.start_index))
                        ));
                    }
                    content.push('\n');
                }
                else {
                    break;
                }
            }
            else {
                line_number += 1;
            }

            line_start_index = line_end_index;
            line.clear();
        }

        Ok(SpanContext {
            line_number,
            column_number,
            content,
        })
    }
}

pub trait TargetError : std::error::Error {
    fn span(&self) -> Option<Span> {
        None
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Target(Box<dyn TargetError>),
    SourceFileOpen {
        cause: std::io::Error,
    },
    SourceFileRead {
        cause: std::io::Error,
    },
    OutputFileOpen {
        path: Box<Path>,
        cause: std::io::Error,
    },
    OutputFileWrite {
        path: Box<Path>,
        cause: std::io::Error,
    },
    InvalidToken,
    InvalidLiteralSuffix,
    InvalidCharacterEscape {
        what: char,
    },
    InvalidHexEscapeDigit {
        what: char,
    },
    InvalidUnicode16Escape {
        value: u16,
    },
    InvalidUnicode32Escape {
        value: u32,
    },
    UnclosedString,
    UnclosedCharacter,
    UnclosedComment,
    ExpectedToken,
    ExpectedTokenFromList {
        got_token: token::TokenKind,
        allowed_tokens: Vec<token::TokenKind>,
    },
    ExpectedIdentifier,
    ExpectedString,
    ExpectedOperand {
        got_token: token::TokenKind,
    },
    ExpectedOperation {
        got_token: token::TokenKind,
    },
    ExpectedType {
        got_token: token::TokenKind,
    },
    ExpectedClosingBracket {
        bracket: token::TokenKind,
    },
    ConditionalMissingCondition,
    UnexpectedConditionalKeyword {
        keyword: token::TokenKind,
    },
    ReservedIdentifier {
        identifier: Box<str>,
    },
    ConflictingGlobalIdentifiers {
        identifier: Box<str>,
    },
    ConflictingActionIdentifiers {
        identifier: Box<str>,
    },
    UnrecognizedType {
        identifier: Box<str>,
    },
    InvalidListItemType {
        item_type: String,
    },
    BroadcastableTypeNotAllowed,
    InvalidBroadcastableItemType {
        item_type: String,
    },
    InvalidPointComponentType {
        component_type: String,
    },
    InvalidArity {
        expected: usize,
        got: usize,
    },
    InvalidIntrinsicArity {
        identifier: Box<str>,
        min: usize,
        max: usize,
        got: usize,
    },
    InvalidVariadicIntrinsicArity {
        identifier: Box<str>,
        min: usize,
        got: usize,
    },
    MismatchedTypes {
        expected: String,
        got: String,
    },
    ExpectedNumericType {
        got_type: String,
    },
    ExpectedNumericOrPointType {
        got_type: String,
    },
    ExpectedNumericPoint2Type {
        got_type: String,
    },
    ExpectedNumericPoint3Type {
        got_type: String,
    },
    ExpectedListType {
        got_type: String,
    },
    ExpectedFunctionType {
        got_type: String,
    },
    ExpectedTypeValue,
    ExpectedEnumTypeValue,
    CannotMergeTypes {
        type_1: String,
        type_2: String,
    },
    IncompatibleTickerIntervals,
    InvalidUpdateLhs,
    UnexpectedExpressionKind,
    IntegerTooLarge,
    UndefinedIntrinsic {
        identifier: Box<str>,
    },
    UndefinedAction {
        identifier: Box<str>,
    },
    UndefinedIdentifier {
        identifier: Box<str>,
    },
    UndefinedEnumVariant {
        enum_identifier: Box<str>,
        variant_identifier: Box<str>,
    },
    InvalidAccessOperation {
        lhs_type: String,
        rhs: Box<str>,
    },
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Target(error) => {
                error.fmt(f)
            }
            Self::SourceFileOpen { cause } => {
                write!(f, "unable to open file: {cause}")
            }
            Self::SourceFileRead { cause } => {
                write!(f, "error while reading file: {cause}")
            }
            Self::OutputFileOpen { path, cause } => {
                write!(f, "unable to create file '{}': {cause}", path.display())
            }
            Self::OutputFileWrite { path, cause } => {
                write!(f, "error while writing file '{}': {cause}", path.display())
            }
            Self::InvalidToken => {
                write!(f, "unrecognized token")
            }
            Self::InvalidLiteralSuffix => {
                write!(f, "unsupported literal suffix")
            }
            Self::InvalidCharacterEscape { what } => {
                write!(f, "unrecognized character escape '\\{what}'")
            }
            Self::InvalidHexEscapeDigit { what } => {
                write!(f, "expected hexadecimal digit, got '{what}'")
            }
            Self::InvalidUnicode16Escape { value } => {
                write!(f, "invalid 16-bit Unicode character '\\u{value:04X}'")
            }
            Self::InvalidUnicode32Escape { value } => {
                write!(f, "invalid 32-bit Unicode character '\\U{value:08X}'")
            }
            Self::UnclosedString => {
                write!(f, "unclosed string literal")
            }
            Self::UnclosedCharacter => {
                write!(f, "expected single quote to close character literal")
            }
            Self::UnclosedComment => {
                write!(f, "unclosed block comment")
            }
            Self::ExpectedToken => {
                write!(f, "unexpected end of file")
            }
            Self::ExpectedTokenFromList { got_token, allowed_tokens } => {
                write!(f, "expected '{}'", &allowed_tokens[0])?;
                for token in &allowed_tokens[1..] {
                    write!(f, ", '{token}'")?;
                }
                write!(f, "; got '{got_token}'")
            }
            Self::ExpectedIdentifier => {
                write!(f, "expected an identifier")
            }
            Self::ExpectedString => {
                write!(f, "expected a quoted string")
            }
            Self::ExpectedOperand { got_token } => {
                write!(f, "expected an operand, got '{got_token}'")
            }
            Self::ExpectedOperation { got_token } => {
                write!(f, "expected an operation, got '{got_token}'")
            }
            Self::ExpectedType { got_token } => {
                write!(f, "expected a type, got '{got_token}'")
            }
            Self::ExpectedClosingBracket { bracket } => {
                write!(f, "expected closing '{bracket}'")
            }
            Self::ConditionalMissingCondition => {
                write!(f, "conditional expression requires at least one condition")
            }
            Self::UnexpectedConditionalKeyword { keyword } => {
                write!(f, "unexpected '{keyword}' without supporting 'if' (did you add an extra comma?)")
            }
            Self::ConflictingGlobalIdentifiers { identifier } => {
                write!(f, "conflicting global definitions for identifier '{identifier}'")
            }
            Self::ConflictingActionIdentifiers { identifier } => {
                write!(f, "multiple actions defined with identifier '{identifier}'")
            }
            Self::ReservedIdentifier { identifier } => {
                write!(f, "'{identifier}' is a reserved identifier")
            }
            Self::UnrecognizedType { identifier } => {
                write!(f, "unrecognized type '{identifier}'")
            }
            Self::InvalidListItemType { item_type } => {
                write!(f, "type '{item_type}' cannot be used inside a list")
            }
            Self::BroadcastableTypeNotAllowed => {
                write!(f, "types can only be broadcastable within a function")
            }
            Self::InvalidBroadcastableItemType { item_type } => {
                write!(f, "type '{item_type}' cannot be marked as broadcastable")
            }
            Self::InvalidPointComponentType { component_type } => {
                write!(f, "type '{component_type}' cannot be the component of a point")
            }
            Self::InvalidArity { expected, got } => {
                write!(f, "expected {expected} arguments, got {got}")
            }
            Self::InvalidIntrinsicArity { identifier, min, max, got } => {
                write!(f, "function '@{identifier}' expects between {min} and {max} arguments but received {got}")
            }
            Self::InvalidVariadicIntrinsicArity { identifier, min, got } => {
                write!(f, "function '@{identifier}' expects at least {min} arguments but received {got}")
            }
            Self::MismatchedTypes { expected, got } => {
                write!(f, "expected a value of type '{expected}', but got '{got}' instead")
            }
            Self::ExpectedNumericType { got_type } => {
                write!(f, "expected a numeric value, got '{got_type}'")
            }
            Self::ExpectedNumericOrPointType { got_type } => {
                write!(f, "expected a numeric value or numeric point, got '{got_type}'")
            }
            Self::ExpectedNumericPoint2Type { got_type } => {
                write!(f, "expected a numeric 2D point, got '{got_type}'")
            }
            Self::ExpectedNumericPoint3Type { got_type } => {
                write!(f, "expected a numeric 3D point, got '{got_type}'")
            }
            Self::ExpectedListType { got_type } => {
                write!(f, "expected a list, got '{got_type}'")
            }
            Self::ExpectedFunctionType { got_type } => {
                write!(f, "expected a function, got '{got_type}'")
            }
            Self::ExpectedTypeValue => {
                write!(f, "expected the name of a type")
            }
            Self::ExpectedEnumTypeValue => {
                write!(f, "expected the name of an enumeration type")
            }
            Self::CannotMergeTypes { type_1, type_2 } => {
                write!(f, "types '{type_1}' and '{type_2}' are incompatible")
            }
            Self::IncompatibleTickerIntervals => {
                write!(f, "having multiple tickers with different intervals is not yet allowed")
            }
            Self::InvalidUpdateLhs => {
                write!(f, "only variables defined using 'var' can be updated")
            }
            Self::UnexpectedExpressionKind => {
                write!(f, "unexpected expression kind")
            }
            Self::IntegerTooLarge => {
                write!(f, "integer too large")
            }
            Self::UndefinedIntrinsic { identifier } => {
                write!(f, "intrinsic '@{}' is not defined", identifier)
            }
            Self::UndefinedAction { identifier } => {
                write!(f, "action '{identifier}' is not defined")
            }
            Self::UndefinedIdentifier { identifier } => {
                write!(f, "identifier '{identifier}' is not defined")
            }
            Self::UndefinedEnumVariant { enum_identifier, variant_identifier } => {
                write!(f, "enum '{enum_identifier}' has no variant '{variant_identifier}'")
            }
            Self::InvalidAccessOperation { lhs_type, rhs } => {
                write!(f, "'{lhs_type}' has no member '{rhs}'")
            }
        }
    }
}

impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Target(error) => Some(error.as_ref()),
            Self::SourceFileOpen { cause, .. } => Some(cause),
            Self::SourceFileRead { cause, .. } => Some(cause),
            Self::OutputFileOpen { cause, .. } => Some(cause),
            Self::OutputFileWrite { cause, .. } => Some(cause),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Option<Span>,
}

pub type Result<T> = std::result::Result<T, Box<Error>>;

impl Error {
    pub fn with_span(mut self: Box<Self>, span: Option<Span>) -> Box<Self> {
        self.span = span;
        self
    }

    pub fn to_string_with_context(&self, paths: &[PathBuf]) -> String {
        if let Some(span) = self.span {
            let path = &paths[span.source_id];
            let path_display = path.display();
            if let Ok(context) = span.load_context(path) {
                format!(
                    "Error in '{path_display}':\nline {}:{}: {self}\n\n{}",
                    context.line_number,
                    context.column_number,
                    context.content,
                )
            }
            else {
                format!("Error in '{path_display}':\n{self}")
            }
        }
        else {
            format!("Error:\n{self}")
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.kind.source()
    }
}
