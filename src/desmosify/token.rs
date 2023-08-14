use super::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Symbol {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Star2,
    Tilde,
    Ampersand,
    Caret,
    Pipe,
    Bang,
    Ampersand2,
    Caret2,
    Pipe2,
    LessThan2,
    GreaterThan2,
    Equal2,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Equal,
    ColonEqual,
    Dot,
    Comma,
    Colon,
    Semicolon,
    ParenLeft,
    ParenRight,
    SquareLeft,
    SquareRight,
    CurlyLeft,
    CurlyRight,
    Question,
    AtSign,
    Hash,
    Dollar,
    Backslash,
    ExclusiveRange,
    InclusiveRange,
    RightArrow,
    RightEqualArrow,
    Colon2,
}

impl Symbol {
    pub fn from_literal(content: &str) -> Option<Self> {
        match content {
            "+" => Some(Self::Plus),
            "-" => Some(Self::Minus),
            "*" => Some(Self::Star),
            "/" => Some(Self::Slash),
            "%" => Some(Self::Percent),
            "**" => Some(Self::Star2),
            "~" => Some(Self::Tilde),
            "&" => Some(Self::Ampersand),
            "^" => Some(Self::Caret),
            "|" => Some(Self::Pipe),
            "!" => Some(Self::Bang),
            "&&" => Some(Self::Ampersand2),
            "^^" => Some(Self::Caret2),
            "||" => Some(Self::Pipe2),
            "<<" => Some(Self::LessThan2),
            ">>" => Some(Self::GreaterThan2),
            "==" => Some(Self::Equal2),
            "!=" => Some(Self::NotEqual),
            "<" => Some(Self::LessThan),
            ">" => Some(Self::GreaterThan),
            "<=" => Some(Self::LessEqual),
            ">=" => Some(Self::GreaterEqual),
            "=" => Some(Self::Equal),
            ":=" => Some(Self::ColonEqual),
            "." => Some(Self::Dot),
            "," => Some(Self::Comma),
            ":" => Some(Self::Colon),
            ";" => Some(Self::Semicolon),
            "(" => Some(Self::ParenLeft),
            ")" => Some(Self::ParenRight),
            "[" => Some(Self::SquareLeft),
            "]" => Some(Self::SquareRight),
            "{" => Some(Self::CurlyLeft),
            "}" => Some(Self::CurlyRight),
            "?" => Some(Self::Question),
            "@" => Some(Self::AtSign),
            "#" => Some(Self::Hash),
            "$" => Some(Self::Dollar),
            "\\" => Some(Self::Backslash),
            ".." => Some(Self::ExclusiveRange),
            "..=" => Some(Self::InclusiveRange),
            "->" => Some(Self::RightArrow),
            "=>" => Some(Self::RightEqualArrow),
            "::" => Some(Self::Colon2),
            _ => None,
        }
    }

    pub fn literal(self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::Star2 => "**",
            Self::Tilde => "~",
            Self::Ampersand => "&",
            Self::Caret => "^",
            Self::Pipe => "|",
            Self::Bang => "!",
            Self::Ampersand2 => "&&",
            Self::Caret2 => "^^",
            Self::Pipe2 => "||",
            Self::LessThan2 => "<<",
            Self::GreaterThan2 => ">>",
            Self::Equal2 => "==",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::GreaterThan => ">",
            Self::LessEqual => "<=",
            Self::GreaterEqual => ">=",
            Self::Equal => "=",
            Self::ColonEqual => ":=",
            Self::Dot => ".",
            Self::Comma => ",",
            Self::Colon => ":",
            Self::Semicolon => ";",
            Self::ParenLeft => "(",
            Self::ParenRight => ")",
            Self::SquareLeft => "[",
            Self::SquareRight => "]",
            Self::CurlyLeft => "{",
            Self::CurlyRight => "}",
            Self::Question => "?",
            Self::AtSign => "@",
            Self::Hash => "#",
            Self::Dollar => "$",
            Self::Backslash => "\\",
            Self::ExclusiveRange => "..",
            Self::InclusiveRange => "..=",
            Self::RightArrow => "->",
            Self::RightEqualArrow => "=>",
            Self::Colon2 => "::",
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.literal())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Public,
    Ticker,
    Display,
    Enum,
    Action,
    Let,
    Const,
    Var,
    Timer,
    If,
    Elif,
    Else,
    For,
    In,
    Where,
    With,
}

impl Keyword {
    pub fn from_literal(content: &str) -> Option<Self> {
        match content {
            "public" => Some(Self::Public),
            "ticker" => Some(Self::Ticker),
            "display" => Some(Self::Display),
            "enum" => Some(Self::Enum),
            "action" => Some(Self::Action),
            "let" => Some(Self::Let),
            "const" => Some(Self::Const),
            "var" => Some(Self::Var),
            "timer" => Some(Self::Timer),
            "if" => Some(Self::If),
            "elif" => Some(Self::Elif),
            "else" => Some(Self::Else),
            "for" => Some(Self::For),
            "in" => Some(Self::In),
            "where" => Some(Self::Where),
            "with" => Some(Self::With),
            _ => None,
        }
    }

    pub fn literal(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Ticker => "ticker",
            Self::Display => "display",
            Self::Enum => "enum",
            Self::Action => "action",
            Self::Let => "let",
            Self::Const => "const",
            Self::Var => "var",
            Self::Timer => "timer",
            Self::If => "if",
            Self::Elif => "elif",
            Self::Else => "else",
            Self::For => "for",
            Self::In => "in",
            Self::Where => "where",
            Self::With => "with",
        }
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.literal())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Symbol(Symbol),
    Keyword(Keyword),
    Name(String),
    Integer(i64),
    Real(f64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub value: TokenValue,
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl Token {
    pub fn new(value: TokenValue, start: SourceLocation, end: SourceLocation) -> Self {
        Self { value, start, end }
    }

    pub fn is_one_of(&self, symbols: &[Symbol], keywords: &[Keyword]) -> bool {
        match self.value {
            TokenValue::Symbol(symbol) => symbols.contains(&symbol),
            TokenValue::Keyword(keyword) => keywords.contains(&keyword),
            _ => false,
        }
    }

    pub fn is_symbol_or_keyword(&self) -> bool {
        if let TokenValue::Symbol(_) | TokenValue::Keyword(_) = self.value {
            true
        } else {
            false
        }
    }
}

struct Lexer<'a> {
    stream: std::iter::Peekable<std::str::Chars<'a>>,
    location: SourceLocation,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Lexer {
            stream: source.chars().peekable(),
            location: SourceLocation::start(),
            tokens: Vec::new(),
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let next = self.stream.next();
        match next {
            Some('\n') => {
                self.location.index += 1;
                self.location.column = 1;
                self.location.line += 1;
            }
            Some(c) => {
                self.location.index += c.len_utf8();
                self.location.column += 1;
            }
            None => {}
        }
        next
    }

    fn peek_char(&mut self) -> Option<&char> {
        self.stream.peek()
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, DesmosifyError> {
    let mut lexer = Lexer::new(source);
    while let Some(&next) = lexer.peek_char() {
        let start = lexer.location;
        if next.is_whitespace() {
            lexer.next_char();
        } else if next.is_ascii_alphabetic() || next == '_' {
            let mut word = String::new();
            while let Some(&next) = lexer.peek_char() {
                if !next.is_ascii_alphanumeric() && next != '_' {
                    break;
                }
                word.push(lexer.next_char().unwrap());
            }
            if let Some(keyword) = Keyword::from_literal(&word) {
                lexer.tokens.push(Token::new(
                    TokenValue::Keyword(keyword),
                    start,
                    lexer.location,
                ));
            } else if word == "true" {
                lexer
                    .tokens
                    .push(Token::new(TokenValue::Boolean(true), start, lexer.location));
            } else if word == "false" {
                lexer.tokens.push(Token::new(
                    TokenValue::Boolean(false),
                    start,
                    lexer.location,
                ));
            } else {
                lexer
                    .tokens
                    .push(Token::new(TokenValue::Name(word), start, lexer.location));
            }
        } else if next.is_ascii_digit() {
            // TODO: binary, hexidecimal
            let mut raw_number = String::new();
            let mut is_integer = true;
            let mut deferred_token = None;
            let mut current_end = lexer.location;
            while let Some(&next) = lexer.peek_char() {
                if next == '_' || next == '\'' {
                    lexer.next_char();
                    continue;
                } else if next == '.' || next == 'E' || next == 'e' {
                    is_integer = false;
                } else if !next.is_ascii_digit() {
                    break;
                }
                current_end = lexer.location;
                raw_number.push(lexer.next_char().unwrap());
                if next == '.' {
                    if let Some(&'.') = lexer.peek_char() {
                        lexer.next_char();
                        if let Some(&'=') = lexer.peek_char() {
                            lexer.next_char();
                            deferred_token = Some(Token::new(
                                TokenValue::Symbol(Symbol::InclusiveRange),
                                current_end,
                                lexer.location,
                            ));
                        } else {
                            deferred_token = Some(Token::new(
                                TokenValue::Symbol(Symbol::ExclusiveRange),
                                current_end,
                                lexer.location,
                            ));
                        }
                        is_integer = true;
                        raw_number.pop();
                        break;
                    }
                }
                current_end = lexer.location;
            }
            if is_integer {
                match raw_number.parse() {
                    Ok(number) => lexer.tokens.push(Token::new(
                        TokenValue::Integer(number),
                        start,
                        current_end,
                    )),
                    Err(_) => {
                        return Err(DesmosifyError::new(
                            String::from("invalid integer literal"),
                            Some(start),
                            Some(current_end),
                        ))
                    }
                }
            } else {
                match raw_number.parse() {
                    Ok(number) => lexer.tokens.push(Token::new(
                        TokenValue::Real(number),
                        start,
                        current_end,
                    )),
                    Err(_) => {
                        return Err(DesmosifyError::new(
                            String::from("invalid floating-point literal"),
                            Some(start),
                            Some(current_end),
                        ))
                    }
                }
            }
            if let Some(token) = deferred_token {
                lexer.tokens.push(token);
            }
        } else if next.is_ascii_graphic() {
            let mut raw_symbol = String::new();
            raw_symbol.push(lexer.next_char().unwrap());
            if next == '/' {
                match lexer.peek_char() {
                    Some(&'/') => {
                        let mut ignore_next_newline = false;
                        while let Some(next) = lexer.next_char() {
                            if ignore_next_newline {
                                if next == '\n' || !next.is_whitespace() {
                                    ignore_next_newline = false;
                                }
                            } else if next == '\n' {
                                break;
                            } else if next == '\\' {
                                ignore_next_newline = true;
                            }
                        }
                        continue;
                    }
                    Some(&'*') => {
                        let mut can_close = false;
                        while let Some(next) = lexer.next_char() {
                            if can_close {
                                if next == '/' {
                                    break;
                                } else if next != '*' {
                                    can_close = false;
                                }
                            } else if next == '*' {
                                can_close = true;
                            }
                        }
                        continue;
                    }
                    _ => {}
                }
            } else if next == '"' {
                let mut string = String::new();
                let mut escaped = false;
                let mut terminated = false;
                while let Some(next) = lexer.next_char() {
                    if escaped {
                        string.push(match next {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            '0' => '\0',
                            _ => next,
                        });
                        escaped = false;
                    } else if next == '"' {
                        terminated = true;
                        break;
                    } else if next == '\\' {
                        escaped = true;
                    } else {
                        string.push(next);
                    }
                }
                if !terminated {
                    return Err(DesmosifyError::new(
                        String::from("string has no closing quote"),
                        Some(start),
                        None,
                    ));
                }
                lexer.tokens.push(Token::new(
                    TokenValue::String(string),
                    start,
                    lexer.location,
                ));
                continue;
            }
            let mut symbol_peek = raw_symbol.clone();
            while let Some(&next) = lexer.peek_char() {
                symbol_peek.push(next);
                if let None = Symbol::from_literal(&symbol_peek) {
                    break;
                }
                raw_symbol.push(lexer.next_char().unwrap());
            }
            if let Some(symbol) = Symbol::from_literal(&raw_symbol) {
                lexer.tokens.push(Token::new(
                    TokenValue::Symbol(symbol),
                    start,
                    lexer.location,
                ));
            } else {
                return Err(DesmosifyError::new(
                    format!("invalid symbol: {symbol_peek}"),
                    Some(start),
                    Some(lexer.location),
                ));
            }
        } else {
            lexer.next_char();
            return Err(DesmosifyError::new(
                format!("unexpected character: {next}"),
                Some(start),
                Some(lexer.location),
            ));
        }
    }
    Ok(lexer.tokens)
}
