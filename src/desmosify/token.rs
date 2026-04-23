use std::rc::Rc;

pub mod scan;

// TODO: complex
#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    Identifier(Rc<str>),
    Integer(i128),
    Real(f64),
    Boolean(bool),
    Character(char),
    String(Rc<str>),
}

impl Literal {
    pub fn from_word(content: &str) -> Self {
        match content {
            "true" => Self::Boolean(true),
            "false" => Self::Boolean(false),
            "infinity" => Self::Real(f64::INFINITY),
            "undefined" => Self::Real(f64::NAN),
            _ => Self::Identifier(content.into()),
        }
    }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(name) => write!(f, "{name}"),
            Self::Integer(value) => write!(f, "{value}"),
            Self::Real(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Character(value) => write!(f, "{value:?}"),
            Self::String(value) => write!(f, "{value:?}"),
        }
    }
}

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
    Let,
    Public,
    Then,
    Ticker,
    Timer,
    Var,
    Where,
    With,
    // Miscellaneous
    Literal(Literal),
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
        match self {
            Self::Ampersand => write!(f, "&"),
            Self::Ampersand2 => write!(f, "&&"),
            Self::AngleLeft => write!(f, "<"),
            Self::AngleLeft2 => write!(f, "<<"),
            Self::AngleLeftEqual => write!(f, "<="),
            Self::AngleRight => write!(f, ">"),
            Self::AngleRight2 => write!(f, ">>"),
            Self::AngleRightEqual => write!(f, ">="),
            Self::AtSign => write!(f, "@"),
            Self::Backslash => write!(f, "\\"),
            Self::Bang => write!(f, "!"),
            Self::BangEqual => write!(f, "!="),
            Self::Caret => write!(f, "^"),
            Self::Colon => write!(f, ":"),
            Self::Colon2 => write!(f, "::"),
            Self::ColonEqual => write!(f, ":="),
            Self::Comma => write!(f, ","),
            Self::CurlyLeft => write!(f, "{{"),
            Self::CurlyRight => write!(f, "}}"),
            Self::Dollar => write!(f, "$"),
            Self::Dot => write!(f, "."),
            Self::Dot2 => write!(f, ".."),
            Self::Equal => write!(f, "="),
            Self::Equal2 => write!(f, "=="),
            Self::Hash => write!(f, "#"),
            Self::Minus => write!(f, "-"),
            Self::ParenLeft => write!(f, "("),
            Self::ParenRight => write!(f, ")"),
            Self::Percent => write!(f, "%"),
            Self::Pipe => write!(f, "|"),
            Self::Pipe2 => write!(f, "||"),
            Self::Plus => write!(f, "+"),
            Self::Question => write!(f, "?"),
            Self::Semicolon => write!(f, ";"),
            Self::Slash => write!(f, "/"),
            Self::SquareLeft => write!(f, "["),
            Self::SquareRight => write!(f, "]"),
            Self::Star => write!(f, "*"),
            Self::Star2 => write!(f, "**"),
            Self::Tilde => write!(f, "~"),
            Self::ArrowRight => write!(f, "->"),
            Self::DoubleArrowRight => write!(f, "=>"),
            Self::RangeInclusive => write!(f, "..="),
            Self::RangeExclusive => write!(f, "..<"),
            Self::Action => write!(f, "action"),
            Self::Disable => write!(f, "disable"),
            Self::Display => write!(f, "display"),
            Self::Elif => write!(f, "elif"),
            Self::Else => write!(f, "else"),
            Self::Enum => write!(f, "enum"),
            Self::For => write!(f, "for"),
            Self::If => write!(f, "if"),
            Self::In => write!(f, "in"),
            Self::Let => write!(f, "let"),
            Self::Public => write!(f, "public"),
            Self::Then => write!(f, "then"),
            Self::Ticker => write!(f, "ticker"),
            Self::Timer => write!(f, "timer"),
            Self::Var => write!(f, "var"),
            Self::Where => write!(f, "where"),
            Self::With => write!(f, "with"),
            Self::Literal(literal) => literal.fmt(f),
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
    ("for", TokenKind::For),
    ("if", TokenKind::If),
    ("in", TokenKind::In),
    ("let", TokenKind::Let),
    ("public", TokenKind::Public),
    ("then", TokenKind::Then),
    ("ticker", TokenKind::Ticker),
    ("timer", TokenKind::Timer),
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
