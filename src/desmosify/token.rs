use std::rc::Rc;

pub mod scan;

#[derive(Clone, PartialEq, Debug)]
pub enum TokenKind {
    // Symbols
    Ampersand,
    Ampersand2,
    AngleLeft,
    AngleLeft2,
    AngleLeftEqual,
    AngleRight,
    AngleRight2,
    AngleRightEqual,
    AtSign,
    Backslash,
    Bang,
    BangEqual,
    Caret,
    Colon,
    Colon2,
    ColonEqual,
    Comma,
    CurlyLeft,
    CurlyRight,
    Dollar,
    Dot,
    Dot2,
    Equal,
    Equal2,
    Hash,
    Minus,
    ParenLeft,
    ParenRight,
    Percent,
    Pipe,
    Pipe2,
    Plus,
    Question,
    Semicolon,
    Slash,
    SquareLeft,
    SquareRight,
    Star,
    Star2,
    Tilde,
    ArrowRight,
    DoubleArrowRight,
    RangeInclusive,
    RangeExclusive,
    // Keywords
    Action,
    Disable,
    Display,
    Elif,
    Else,
    Enum,
    For,
    If,
    In,
    Infinity,
    Let,
    Public,
    Then,
    Ticker,
    Timer,
    Undefined,
    Var,
    Where,
    With,
    // Miscellaneous
    Integer(i128),
    Real(f64),
    Boolean(bool),
    Character(char),
    String(Rc<str>),
    Identifier(Rc<str>),
}

impl TokenKind {
    pub fn get_symbolic_literal(&self) -> Option<&'static str> {
        SYMBOLIC_TOKENS
            .iter()
            .find_map(|&(literal, ref kind)| (kind == self).then_some(literal))
    }

    pub fn get_keyword_literal(&self) -> Option<&'static str> {
        KEYWORD_TOKENS
            .iter()
            .find_map(|&(literal, ref kind)| (kind == self).then_some(literal))
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(symbolic_literal) = self.get_symbolic_literal() {
            return write!(f, "{symbolic_literal}")
        }
        else if let Some(keyword_literal) = self.get_keyword_literal() {
            return write!(f, "{keyword_literal}")
        }
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::Real(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Character(value) => write!(f, "{value:?}"),
            Self::String(value) => write!(f, "{value:?}"),
            Self::Identifier(identifier) => write!(f, "{identifier}"),
            _ => unreachable!("all tokens should be matched by this point")
        }
    }
}

pub const SYMBOLIC_TOKENS: &[(&str, TokenKind)] = &[
    ("&", TokenKind::Ampersand),
    ("&&", TokenKind::Ampersand2),
    ("<", TokenKind::AngleLeft),
    ("<<", TokenKind::AngleLeft2),
    ("<=", TokenKind::AngleLeftEqual),
    (">", TokenKind::AngleRight),
    (">>", TokenKind::AngleRight2),
    (">=", TokenKind::AngleRightEqual),
    ("@", TokenKind::AtSign),
    ("\\", TokenKind::Backslash),
    ("!", TokenKind::Bang),
    ("!=", TokenKind::BangEqual),
    ("^", TokenKind::Caret),
    (":", TokenKind::Colon),
    ("::", TokenKind::Colon2),
    (":=", TokenKind::ColonEqual),
    (",", TokenKind::Comma),
    ("{", TokenKind::CurlyLeft),
    ("}", TokenKind::CurlyRight),
    ("$", TokenKind::Dollar),
    (".", TokenKind::Dot),
    ("..", TokenKind::Dot2),
    ("=", TokenKind::Equal),
    ("==", TokenKind::Equal2),
    ("#", TokenKind::Hash),
    ("-", TokenKind::Minus),
    ("(", TokenKind::ParenLeft),
    (")", TokenKind::ParenRight),
    ("%", TokenKind::Percent),
    ("|", TokenKind::Pipe),
    ("||", TokenKind::Pipe2),
    ("+", TokenKind::Plus),
    ("?", TokenKind::Question),
    (";", TokenKind::Semicolon),
    ("/", TokenKind::Slash),
    ("[", TokenKind::SquareLeft),
    ("]", TokenKind::SquareRight),
    ("*", TokenKind::Star),
    ("**", TokenKind::Star2),
    ("~", TokenKind::Tilde),
    ("->", TokenKind::ArrowRight),
    ("=>", TokenKind::DoubleArrowRight),
    ("..=", TokenKind::RangeInclusive),
    ("..<", TokenKind::RangeExclusive),
];

pub const KEYWORD_TOKENS: &[(&str, TokenKind)] = &[
    ("action", TokenKind::Action),
    ("disable", TokenKind::Disable),
    ("display", TokenKind::Display),
    ("elif", TokenKind::Elif),
    ("else", TokenKind::Else),
    ("enum", TokenKind::Enum),
    ("false", TokenKind::Boolean(false)),
    ("for", TokenKind::For),
    ("if", TokenKind::If),
    ("in", TokenKind::In),
    ("infinity", TokenKind::Infinity),
    ("let", TokenKind::Let),
    ("public", TokenKind::Public),
    ("then", TokenKind::Then),
    ("ticker", TokenKind::Ticker),
    ("timer", TokenKind::Timer),
    ("true", TokenKind::Boolean(true)),
    ("undefined", TokenKind::Undefined),
    ("var", TokenKind::Var),
    ("where", TokenKind::Where),
    ("with", TokenKind::With),
];

pub fn get_symbolic_token_partial_matches(start_content: &str) -> Vec<&'static TokenKind> {
    SYMBOLIC_TOKENS.iter()
        .filter_map(|&(literal, ref symbolic_token)| {
            literal.starts_with(start_content).then_some(symbolic_token)
        })
        .collect()
}

pub fn get_symbolic_token_match(content: &str) -> Option<&'static TokenKind> {
    SYMBOLIC_TOKENS.iter()
        .find_map(|&(literal, ref symbolic_token)| {
            (content == literal).then_some(symbolic_token)
        })
}

pub fn get_keyword_token_match(content: &str) -> Option<&'static TokenKind> {
    KEYWORD_TOKENS.iter()
        .find_map(|&(keyword, ref keyword_token)| {
            (content == keyword).then_some(keyword_token)
        })
}

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: crate::Span,
}
