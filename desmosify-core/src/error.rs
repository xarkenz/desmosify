use std::path::Path;
use crate::SourceFiles;
use crate::token::TokenKind;

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
        name: Box<str>,
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
        got_token: TokenKind,
        allowed_tokens: Vec<TokenKind>,
    },
    ExpectedIdentifier,
    ExpectedString,
    ExpectedOperand {
        got_token: TokenKind,
    },
    ExpectedOperation {
        got_token: TokenKind,
    },
    ExpectedType {
        got_token: TokenKind,
    },
    ExpectedClosingBracket {
        bracket: TokenKind,
    },
    ConditionalMissingCondition,
    UnexpectedConditionalKeyword {
        keyword: TokenKind,
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
    ExpectedNumericPointType {
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
    ExpectedListOrDistributionType {
        got_type: String,
    },
    ExpectedFunctionType {
        got_type: String,
    },
    ExpectedActionType {
        expected_parameter_lists: Vec<Vec<String>>,
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
    UnsupportedValue,
    UnsupportedDisplayAttribute {
        key: Box<str>,
    },
    DuplicatedDisplayAttribute {
        key: Box<str>,
    },
    InvalidDisplayAttributeArity {
        key: Box<str>,
        min: usize,
        max: usize,
        got: usize,
    },
    ExpectedConstant {
        type_identifier: String,
    },
    ExpectedConstantStrFromList {
        allowed: Vec<Box<str>>,
    },
    ExpectedAction,
    ExpectedGlobalOrActionReference,
    MultipleSlidersForVariable {
        identifier: Box<str>,
    },
    InvalidSliderReference {
        identifier: Box<str>,
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
            Self::MismatchedTypes { expected, got } => {
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
                fn write_parameter_list(f: &mut std::fmt::Formatter, parameter_list: &[String]) -> std::fmt::Result {
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
    pub span: Option<crate::Span>,
}

pub type Result<T> = std::result::Result<T, Box<Error>>;

impl Error {
    pub fn with_span(mut self: Box<Self>, span: Option<crate::Span>) -> Box<Self> {
        self.span = span;
        self
    }

    pub fn display_with_context<'a>(&self, sources: &SourceFiles<'a>) -> ErrorDisplayWithContext<'_, 'a> {
        ErrorDisplayWithContext {
            error: self,
            context: self.span
                .as_ref()
                .map(|span| span.get_context(sources))
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

pub struct ErrorDisplayWithContext<'err, 'ctx> {
    error: &'err Error,
    context: Option<crate::SpanContext<'ctx>>,
}

impl<'err, 'ctx> std::fmt::Display for ErrorDisplayWithContext<'err, 'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(context) = &self.context else {
            writeln!(f, "Error:")?;
            return self.error.fmt(f)
        };

        writeln!(f, "Error in '{}':", context.path.display())?;
        writeln!(f, "line {}:{}: {}", context.start_line + 1, context.start_column + 1, self.error)?;
        writeln!(f)?;

        let mut start_line = context.start_line;
        let mut start_column = context.start_column;
        let mut end_line = context.end_line;
        let mut end_column = context.end_column;

        // Make sure the start position comes first. This case shouldn't really happen, but we
        // can handle it gracefully if it does.
        if (start_line, start_column) > (end_line, end_column) {
            std::mem::swap(&mut start_line, &mut end_line);
            std::mem::swap(&mut start_column, &mut end_column);
        }

        for (index, line) in context.content.lines().enumerate() {
            let line = line.trim_end();
            if line.is_empty() {
                continue
            }
            writeln!(f, "\t{line}")?;

            // Generate the squiggly line underneath the indicated span.
            write!(f, "\t")?;
            let indicator_length;
            if index == 0 {
                for ch in line[..start_column].chars() {
                    std::fmt::Write::write_char(f, if ch.is_whitespace() { ch } else { ' ' })?;
                }
                if start_line == end_line {
                    indicator_length = end_column - start_column;
                }
                else {
                    indicator_length = line.len() - start_column;
                }
            }
            else if start_line + index < end_line {
                indicator_length = line.len();
            }
            else if start_line + index == end_line {
                indicator_length = end_column;
            }
            else {
                break
            }

            if indicator_length == 0 {
                write!(f, "^")?;
            }
            else {
                for _ in 0..indicator_length {
                    write!(f, "~")?;
                }
            }
        }

        Ok(())
    }
}
