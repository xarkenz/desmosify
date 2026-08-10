use super::*;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;

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

#[derive(Debug)]
pub enum ErrorKind {
    FileOpen {
        path: Option<Box<Path>>,
        cause: std::io::Error,
    },
    FileRead {
        path: Option<Box<Path>>,
        cause: std::io::Error,
    },
    FileCreate {
        path: Option<Box<Path>>,
        cause: std::io::Error,
    },
    FileWrite {
        path: Option<Box<Path>>,
        cause: std::io::Error,
    },
    UnsupportedTarget {
        name: String,
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
        identifier: Rc<str>,
    },
    ConflictingGlobalIdentifiers {
        identifier: Rc<str>,
    },
    ConflictingActionIdentifiers {
        identifier: Rc<str>,
    },
    UnrecognizedType {
        identifier: Rc<str>,
    },
    InvalidListItemType {
        item_type: Rc<str>,
    },
    BroadcastableTypeNotAllowed,
    InvalidBroadcastableItemType {
        item_type: Rc<str>,
    },
    InvalidPointComponentType {
        component_type: Rc<str>,
    },
    InvalidArity {
        expected: usize,
        got: usize,
    },
    InvalidIntrinsicArity {
        identifier: Rc<str>,
        min: usize,
        max: usize,
        got: usize,
    },
    InvalidVariadicIntrinsicArity {
        identifier: Rc<str>,
        min: usize,
        got: usize,
    },
    MismatchedTypes {
        expected_type: Rc<str>,
        got_type: Rc<str>,
    },
    ExpectedNumericType {
        got_type: Rc<str>,
    },
    ExpectedNumericOrPointType {
        got_type: Rc<str>,
    },
    ExpectedNumericPointType {
        got_type: Rc<str>,
    },
    ExpectedNumericPoint2Type {
        got_type: Rc<str>,
    },
    ExpectedNumericPoint3Type {
        got_type: Rc<str>,
    },
    ExpectedListType {
        got_type: Rc<str>,
    },
    ExpectedListOrDistributionType {
        got_type: Rc<str>,
    },
    ExpectedFunctionType {
        got_type: Rc<str>,
    },
    ExpectedActionType {
        expected_parameter_lists: Vec<Vec<Rc<str>>>,
        got_type: Rc<str>,
    },
    ExpectedTypeValue,
    ExpectedEnumTypeValue,
    CannotMergeTypes {
        lhs_type: Rc<str>,
        rhs_type: Rc<str>,
    },
    IncompatibleTickerIntervals,
    InvalidUpdateLhs,
    UnexpectedExpressionKind,
    IntegerTooLarge,
    UndefinedIntrinsic {
        identifier: Rc<str>,
    },
    UndefinedAction {
        identifier: Rc<str>,
    },
    UndefinedIdentifier {
        identifier: Rc<str>,
    },
    UndefinedEnumValue {
        enum_identifier: Rc<str>,
        variant_identifier: Rc<str>,
    },
    InvalidAccessOperation {
        lhs_type: Rc<str>,
        rhs: Rc<str>,
    },
    UnsupportedValue,
    UnsupportedDisplayAttribute {
        key: Rc<str>,
    },
    DuplicatedDisplayAttribute {
        key: Rc<str>,
    },
    InvalidDisplayAttributeArity {
        key: Rc<str>,
        min: usize,
        max: usize,
        got: usize,
    },
    ExpectedConstant {
        type_identifier: Rc<str>,
    },
    ExpectedConstantStrFromList {
        allowed: Vec<Rc<str>>,
    },
    ExpectedAction,
    ExpectedGlobalOrActionReference,
    MultipleSlidersForVariable {
        identifier: Rc<str>,
    },
    InvalidSliderReference {
        identifier: Rc<str>,
    },
    CannotNestFolders,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileOpen { path, cause } => {
                if let Some(path) = path {
                    write!(f, "unable to open file '{}': {cause}", path.display())
                }
                else {
                    write!(f, "unable to open file: {cause}")
                }
            }
            Self::FileRead { path, cause } => {
                if let Some(path) = path {
                    write!(f, "error while reading file '{}': {cause}", path.display())
                }
                else {
                    write!(f, "error while reading file: {cause}")
                }
            }
            Self::FileCreate { path, cause } => {
                if let Some(path) = path {
                    write!(f, "unable to create file '{}': {cause}", path.display())
                }
                else {
                    write!(f, "unable to create file: {cause}")
                }
            }
            Self::FileWrite { path, cause } => {
                if let Some(path) = path {
                    write!(f, "error while writing file '{}': {cause}", path.display())
                }
                else {
                    write!(f, "error while writing file: {cause}")
                }
            }
            Self::UnsupportedTarget { name } => {
                write!(f, "compilation target '{}' is not supported", name)
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
                write!(f, "expected an operation or end of expression, got '{got_token}'")
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
                if min == max {
                    write!(f, "function '@{identifier}' expects {min} arguments but received {got}")
                }
                else {
                    write!(f, "function '@{identifier}' expects between {min} and {max} arguments but received {got}")
                }
            }
            Self::InvalidVariadicIntrinsicArity { identifier, min, got } => {
                write!(f, "function '@{identifier}' expects at least {min} arguments but received {got}")
            }
            Self::MismatchedTypes { expected_type: expected, got_type: got } => {
                write!(f, "expected a value of type '{expected}', but got '{got}' instead")
            }
            Self::ExpectedNumericType { got_type } => {
                write!(f, "expected a numeric value, got '{got_type}'")
            }
            Self::ExpectedNumericOrPointType { got_type } => {
                write!(f, "expected a numeric value or numeric point, got '{got_type}'")
            }
            Self::ExpectedNumericPointType { got_type } => {
                write!(f, "expected a numeric 2D or 3D point, got '{got_type}'")
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
            Self::ExpectedListOrDistributionType { got_type } => {
                write!(f, "expected a list or distribution, got '{got_type}'")
            }
            Self::ExpectedFunctionType { got_type } => {
                write!(f, "expected a function, got '{got_type}'")
            }
            Self::ExpectedActionType { expected_parameter_lists, got_type } => {
                write!(f, "expected an action accepting parameters ")?;
                fn write_parameter_list(f: &mut std::fmt::Formatter, parameter_list: &[Rc<str>]) -> std::fmt::Result {
                    match parameter_list {
                        [] => write!(f, "()"),
                        [first, rest @ ..] => {
                            write!(f, "({first}")?;
                            for parameter in rest {
                                write!(f, ", {parameter}")?;
                            }
                            Ok(())
                        }
                    }
                }
                write_parameter_list(f, &expected_parameter_lists[0])?;
                for parameter_list in &expected_parameter_lists[1..] {
                    write!(f, ", ")?;
                    write_parameter_list(f, parameter_list)?
                }
                write!(f, "; got {got_type}")
            }
            Self::ExpectedTypeValue => {
                write!(f, "expected the name of a type")
            }
            Self::ExpectedEnumTypeValue => {
                write!(f, "expected the name of an enumeration type")
            }
            Self::CannotMergeTypes { lhs_type, rhs_type } => {
                write!(f, "types '{lhs_type}' and '{rhs_type}' are incompatible")
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
            Self::UndefinedEnumValue { enum_identifier, variant_identifier } => {
                write!(f, "enum '{enum_identifier}' has no variant '{variant_identifier}'")
            }
            Self::InvalidAccessOperation { lhs_type, rhs } => {
                write!(f, "'{lhs_type}' has no member '{rhs}'")
            }
            Self::UnsupportedValue => {
                write!(f, "this value is not supported by the target")
            }
            Self::UnsupportedDisplayAttribute { key } => {
                write!(f, "display attribute '{key}' is not supported by the target")
            }
            Self::DuplicatedDisplayAttribute { key } => {
                write!(f, "display attribute '{key}' is duplicated on this element")
            }
            Self::InvalidDisplayAttributeArity { key, min, max, got } => {
                if min == max {
                    write!(f, "display attribute '{key}' expects {min} arguments but received {got}")
                }
                else {
                    write!(f, "display attribute '{key}' expects between {min} and {max} arguments but received {got}")
                }
            }
            Self::ExpectedConstant { type_identifier } => {
                write!(f, "expected a constant value of type '{type_identifier}'")
            }
            Self::ExpectedConstantStrFromList { allowed } => {
                write!(f, "expected a constant string from {:?}", &allowed[0])?;
                for string in &allowed[1..] {
                    write!(f, ", {string:?}")?;
                }
                Ok(())
            }
            Self::ExpectedAction => {
                write!(f, "expected an action")
            }
            Self::ExpectedGlobalOrActionReference => {
                write!(f, "expected a global or action referenced by identifier")
            }
            Self::MultipleSlidersForVariable { identifier } => {
                write!(f, "variable '{identifier}' has already been assigned a public slider")
            }
            Self::InvalidSliderReference { identifier } => {
                write!(f, "global '{identifier}' cannot be used as a slider")
            }
            Self::CannotNestFolders => {
                write!(f, "folders cannot contain other folders")
            }
        }
    }
}

impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FileOpen { cause, .. } => Some(cause),
            Self::FileRead { cause, .. } => Some(cause),
            Self::FileCreate { cause, .. } => Some(cause),
            Self::FileWrite { cause, .. } => Some(cause),
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

    // TODO: convert this into a separate struct with a Display impl
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
