use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Symbol {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Exponent,
    Tilde,
    Ampersand,
    Caret,
    Pipe,
    Exclamation,
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
    EvalEqual,
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
    RangeExc,
    RangeInc,
    RightArrow,
    LambdaArrow,
}

impl Symbol {
    pub fn find_variant(content: &str) -> Option<Self> {
        match content {
            "+" => Some(Self::Plus),
            "-" => Some(Self::Minus),
            "*" => Some(Self::Star),
            "/" => Some(Self::Slash),
            "%" => Some(Self::Percent),
            "**" => Some(Self::Exponent),
            "~" => Some(Self::Tilde),
            "&" => Some(Self::Ampersand),
            "^" => Some(Self::Caret),
            "|" => Some(Self::Pipe),
            "!" => Some(Self::Exclamation),
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
            ":=" => Some(Self::EvalEqual),
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
            ".." => Some(Self::RangeExc),
            "..=" => Some(Self::RangeInc),
            "->" => Some(Self::RightArrow),
            "=>" => Some(Self::LambdaArrow),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Public,
    Ticker,
    Action,
    Let,
    Const,
    Var,
    If,
    Elif,
    Else,
    For,
    In,
    With,
}

impl Keyword {
    pub fn find_variant(content: &str) -> Option<Self> {
        match content {
            "public" => Some(Self::Public),
            "ticker" => Some(Self::Ticker),
            "action" => Some(Self::Action),
            "let" => Some(Self::Let),
            "const" => Some(Self::Const),
            "var" => Some(Self::Var),
            "if" => Some(Self::If),
            "elif" => Some(Self::Elif),
            "else" => Some(Self::Else),
            "for" => Some(Self::For),
            "in" => Some(Self::In),
            "with" => Some(Self::With),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Symbol(Symbol),
    Keyword(Keyword),
    Name(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    String(String),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SourceLocation {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub const fn start() -> Self {
        Self {
            index: 0,
            line: 1,
            column: 1,
        }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(line {}:{})", self.line, self.column)
    }
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

#[derive(Clone, Debug)]
pub struct DesmosifyError {
    message: String,
    start: Option<SourceLocation>,
    end: Option<SourceLocation>,
}

impl DesmosifyError {
    pub fn new(
        message: String,
        start: Option<SourceLocation>,
        end: Option<SourceLocation>,
    ) -> Self {
        Self {
            message,
            start,
            end,
        }
    }
}

impl std::fmt::Display for DesmosifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(location) = self.start {
            write!(f, "{} ", location)?;
        }
        write!(f, "{}", self.message)
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
            Some(_) => {
                self.location.index += 1;
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

pub fn lex_tokens_from_str(source: &str) -> Result<Vec<Token>, DesmosifyError> {
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
            if let Some(keyword) = Keyword::find_variant(&word) {
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
            // TODO: binary, octal, hexidecimal, leading decimal point?
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
                                TokenValue::Symbol(Symbol::RangeInc),
                                current_end,
                                lexer.location,
                            ));
                        } else {
                            deferred_token = Some(Token::new(
                                TokenValue::Symbol(Symbol::RangeExc),
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
                        TokenValue::Number(number),
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
                if let None = Symbol::find_variant(&symbol_peek) {
                    break;
                }
                raw_symbol.push(lexer.next_char().unwrap());
            }
            if let Some(symbol) = Symbol::find_variant(&raw_symbol) {
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

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum Precedence {
    Container = 13,
    Access = 12,
    Postfix = 11,
    Prefix = 10,
    Exponent = 9,
    Multiplicative = 8,
    Additive = 7,
    Comparison = 6,
    Equality = 5,
    Logical = 4,
    Range = 3,
    Conditional = 2,
    Assignment = 1,
}

impl Precedence {
    pub fn is_left_to_right_associative(&self) -> bool {
        match self {
            Self::Prefix | Self::Conditional | Self::Assignment => false,
            _ => true,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Operation {
    Point,
    List,
    For,
    ForIf,
    Access,
    Call,
    Index,
    Pos,
    Neg,
    Not,
    Pow,
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Xor,
    Or,
    RangeExc,
    RangeInc,
    IfElse,
    Assign,
    Update,
}

impl Operation {
    pub fn find_variant(symbol: Symbol, prefix: bool) -> Option<Self> {
        use Symbol::*;
        Some(match symbol {
            Plus => {
                if prefix {
                    Self::Pos
                } else {
                    Self::Add
                }
            }
            Minus => {
                if prefix {
                    Self::Neg
                } else {
                    Self::Sub
                }
            }
            Exclamation => {
                if prefix {
                    Self::Not
                } else {
                    return None;
                }
            }
            _ if prefix => return None,
            Star => Self::Mul,
            Slash => Self::Div,
            Percent => Self::Mod,
            Caret => Self::Pow,
            Ampersand2 => Self::And,
            Caret2 => Self::Xor,
            Pipe2 => Self::Or,
            Equal2 => Self::Eq,
            NotEqual => Self::Ne,
            LessThan => Self::Lt,
            GreaterThan => Self::Gt,
            LessEqual => Self::Le,
            GreaterEqual => Self::Ge,
            Equal => Self::Assign,
            EvalEqual => Self::Update,
            Dot => Self::Access,
            ParenLeft => Self::Call,
            SquareLeft => Self::Index,
            Question => Self::IfElse,
            RangeExc => Self::RangeExc,
            RangeInc => Self::RangeInc,
            _ => return None,
        })
    }

    pub fn precedence(&self) -> Precedence {
        match self {
            Self::Point | Self::List | Self::For | Self::ForIf => Precedence::Container,
            Self::Access => Precedence::Access,
            Self::Call | Self::Index => Precedence::Postfix,
            Self::Pos | Self::Neg | Self::Not => Precedence::Prefix,
            Self::Pow => Precedence::Exponent,
            Self::Mul | Self::Div | Self::Mod => Precedence::Multiplicative,
            Self::Add | Self::Sub => Precedence::Additive,
            Self::Lt | Self::Gt | Self::Le | Self::Ge => Precedence::Comparison,
            Self::Eq | Self::Ne => Precedence::Equality,
            Self::And | Self::Xor | Self::Or => Precedence::Logical,
            Self::RangeExc | Self::RangeInc => Precedence::Range,
            Self::IfElse => Precedence::Conditional,
            Self::Assign | Self::Update => Precedence::Assignment,
        }
    }

    pub fn precedes(&self, rhs: Self) -> bool {
        let left_precedence = self.precedence();
        let right_precedence = rhs.precedence();
        left_precedence > right_precedence
            || (left_precedence == right_precedence
                && left_precedence.is_left_to_right_associative())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DataType {
    Unknown,
    Real,
    Int,
    Bool,
    Point,
    IPoint,
    Color,
    Polygon,
    String,
    List(Box<DataType>),
    Function,
    Action,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DataValue {
    Real(f64),
    Int(i64),
    Bool(bool),
    Point(f64, f64),
    IPoint(i64, i64),
    ColorRGB(f64, f64, f64),
    ColorHSV(f64, f64, f64),
    Polygon(Vec<(f64, f64)>),
    String(String),
    List(DataType, Vec<DataValue>),
}

impl DataValue {
    pub fn from_token_value(value: &TokenValue) -> Option<Self> {
        Some(match value {
            TokenValue::Integer(value) => Self::Int(*value),
            TokenValue::Number(value) => Self::Real(*value),
            TokenValue::Boolean(value) => Self::Bool(*value),
            TokenValue::String(value) => Self::String(value.clone()),
            _ => return None,
        })
    }

    pub fn data_type(&self) -> DataType {
        match self {
            Self::Real(_) => DataType::Real,
            Self::Int(_) => DataType::Int,
            Self::Bool(_) => DataType::Bool,
            Self::Point(_, _) => DataType::Point,
            Self::IPoint(_, _) => DataType::IPoint,
            Self::ColorRGB(_, _, _) => DataType::Color,
            Self::ColorHSV(_, _, _) => DataType::Color,
            Self::Polygon(_) => DataType::Polygon,
            Self::String(_) => DataType::String,
            Self::List(inner, _) => DataType::List(Box::new(inner.clone())),
        }
    }
}

#[derive(Debug)]
pub enum ExpressionValue {
    Literal(DataValue),
    Operator(Operation, Vec<Expression>),
    Name(String),
}

impl ExpressionValue {
    pub fn from_token_value(value: &TokenValue) -> Option<Self> {
        if let TokenValue::Name(name) = value {
            Some(Self::Name(name.clone()))
        } else {
            DataValue::from_token_value(value).map(|literal| Self::Literal(literal))
        }
    }
}

#[derive(Debug)]
pub struct Expression {
    data_type: DataType,
    value: ExpressionValue,
    start_token: Option<usize>,
    op_token: Option<usize>,
    end_token: Option<usize>,
    constant: bool,
}

#[derive(Debug)]
pub struct Parameter {
    name: String,
    data_type: DataType,
}

#[derive(Debug)]
pub enum Action {
    Block(Vec<Action>),
    Update(Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Conditional(Vec<(Expression, Action)>, Option<Box<Action>>),
}

#[derive(Debug)]
pub enum Identifier {
    Const(String, Vec<Parameter>, Option<Box<Expression>>, Box<Expression>),
    Let(String, Vec<Parameter>, Option<Box<Expression>>, Box<Expression>),
    Var(String, Option<Box<Expression>>, Box<Expression>),
    Action(String, Vec<Parameter>, Box<Action>),
}

#[derive(Debug)]
pub struct Definitions {
    public: Option<Vec<Expression>>,
    ticker: Option<(Option<Box<Expression>>, Box<Action>)>,
    namespace: BTreeMap<String, Identifier>,
}

#[derive(Debug)]
struct Parser<'a> {
    tokens: &'a [Token],
    token_index: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            token_index: 0,
        }
    }

    fn token(&self) -> Result<&'a Token, DesmosifyError> {
        let location = Some(self.tokens.last().unwrap().end);
        self.tokens.get(self.token_index).map_or_else(
            || {
                Err(DesmosifyError::new(
                    String::from("unexpected end of file"),
                    location,
                    location,
                ))
            },
            |token| Ok(token),
        )
    }

    fn next(&mut self) {
        self.token_index += 1;
    }

    fn is_at_symbol(&self, value: Symbol) -> Result<bool, DesmosifyError> {
        if let TokenValue::Symbol(symbol) = self.token()?.value {
            Ok(symbol == value)
        } else {
            Ok(false)
        }
    }

    fn is_at_keyword(&self, value: Keyword) -> Result<bool, DesmosifyError> {
        if let TokenValue::Keyword(keyword) = self.token()?.value {
            Ok(keyword == value)
        } else {
            Ok(false)
        }
    }

    fn get_string_from_name(&self) -> Result<String, DesmosifyError> {
        let token = self.token()?;
        if let TokenValue::Name(name) = &token.value {
            Ok(name.clone())
        } else {
            Err(DesmosifyError::new(
                String::from("expected a name"),
                Some(token.start),
                Some(token.end),
            ))
        }
    }

    fn get_operation(&self, prefix: bool) -> Result<Operation, DesmosifyError> {
        let token = self.token()?;
        if let TokenValue::Symbol(symbol) = token.value {
            if let Some(operation) = Operation::find_variant(symbol, prefix) {
                return Ok(operation);
            }
        }
        Err(DesmosifyError::new(
            String::from(if prefix {
                "expected an operand"
            } else {
                "expected an operator"
            }),
            Some(token.start),
            Some(token.end),
        ))
    }

    fn wrap_top_operator_into_operand(
        &mut self,
        operators: &mut Vec<(Operation, usize)>,
        operands: &mut Vec<Expression>,
    ) -> Result<(), DesmosifyError> {
        let (operation, operand_count) = operators.pop().unwrap();
        if operands.len() < operand_count {
            Err(DesmosifyError::new(
                format!(
                    "too few operands for operation (expected {operand_count}, got {})",
                    operands.len()
                ),
                None,
                None,
            ))
        } else {
            let child_operands: Vec<_> = operands
                .splice((operands.len() - operand_count)..operands.len(), [])
                .collect();
            operands.push(Expression {
                data_type: DataType::Unknown, // TODO
                start_token: child_operands
                    .first()
                    .map_or(None, |first| first.start_token),
                op_token: None, // TODO
                end_token: child_operands.last().map_or(None, |last| last.end_token),
                constant: child_operands.iter().all(|item| item.constant),
                value: ExpressionValue::Operator(operation, child_operands),
            });
            Ok(())
        }
    }

    fn parse_expression(
        &mut self,
        end_symbols: &[Symbol],
        end_keywords: &[Keyword],
    ) -> Result<Expression, DesmosifyError> {
        let mut operators: Vec<(Operation, usize)> = Vec::new();
        let mut operands: Vec<Expression> = Vec::new();
        let mut expect_operand = true;
        loop {
            let token = self.token()?;
            if (!expect_operand || operands.is_empty())
                && token.is_one_of(end_symbols, end_keywords)
            {
                break;
            } else if token.is_symbol_or_keyword() {
                let mut operand_count: usize = 2;
                if expect_operand {
                    if self.is_at_symbol(Symbol::ParenLeft)? {
                        expect_operand = false;
                        self.next();
                        operands.push(
                            self.parse_expression(&[Symbol::Comma, Symbol::ParenRight], &[])?,
                        );
                        if self.is_at_symbol(Symbol::Comma)? {
                            self.next();
                            operands.push(self.parse_expression(&[Symbol::ParenRight], &[])?);
                            operators.push((Operation::Point, 2));
                            self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                        }
                        self.next();
                        continue;
                    } else if self.is_at_symbol(Symbol::SquareLeft)? {
                        expect_operand = false;
                        operand_count = 0;
                        self.next();
                        let mut list_comprehension = false;
                        while !self.is_at_symbol(Symbol::SquareRight)? {
                            operands.push(self.parse_expression(
                                &[Symbol::Comma, Symbol::SquareRight],
                                &[Keyword::For],
                            )?);
                            operand_count += 1;
                            if self.is_at_symbol(Symbol::SquareRight)? {
                                break;
                            } else if self.is_at_keyword(Keyword::For)? {
                                if operand_count != 1 {
                                    return Err(DesmosifyError::new(
                                        String::from(
                                            "list comprehension must be done on a single element",
                                        ),
                                        None,
                                        None,
                                    ));
                                }
                                list_comprehension = true;
                                while self.is_at_keyword(Keyword::For)? {
                                    self.next();
                                    operands.push(self.parse_expression(&[], &[Keyword::In])?);
                                    self.next();
                                    operands.push(self.parse_expression(
                                        &[Symbol::SquareRight],
                                        &[Keyword::For, Keyword::If],
                                    )?);
                                    operators.push((Operation::For, 3));
                                    self.wrap_top_operator_into_operand(
                                        &mut operators,
                                        &mut operands,
                                    )?;
                                }
                                if self.is_at_keyword(Keyword::If)? {
                                    self.next();
                                    operands
                                        .push(self.parse_expression(&[Symbol::SquareRight], &[])?);
                                    operators.push((Operation::ForIf, 2));
                                    self.wrap_top_operator_into_operand(
                                        &mut operators,
                                        &mut operands,
                                    )?;
                                }
                                self.next();
                                break;
                            }
                            self.next();
                        }
                        if !list_comprehension {
                            operators.push((Operation::List, operand_count));
                            self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                            self.next();
                        }
                        continue;
                    }
                }
                let operation = self.get_operation(expect_operand)?;
                while operators
                    .last()
                    .map_or(false, |(lhs, _)| lhs.precedes(operation))
                {
                    self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                }
                match operation {
                    Operation::Call => {
                        operand_count = 1;
                        self.next();
                        if !self.is_at_symbol(Symbol::ParenRight)? {
                            loop {
                                operands.push(
                                    self.parse_expression(
                                        &[Symbol::Comma, Symbol::ParenRight],
                                        &[],
                                    )?,
                                );
                                operand_count += 1;
                                if self.is_at_symbol(Symbol::ParenRight)? {
                                    break;
                                }
                                self.next();
                            }
                        }
                    }
                    Operation::Index => {
                        self.next();
                        operands.push(self.parse_expression(&[Symbol::SquareRight], &[])?);
                    }
                    Operation::IfElse => {
                        operand_count = 3;
                        self.next();
                        operands.push(self.parse_expression(&[Symbol::Colon], &[])?);
                    }
                    Operation::Pos | Operation::Neg | Operation::Not => {
                        operand_count = 1;
                    }
                    _ => {}
                }
                operators.push((operation, operand_count));
                if let Operation::Call | Operation::Index = operation {
                    self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                    expect_operand = false;
                } else {
                    expect_operand = true;
                }
            } else {
                if !expect_operand {
                    return Err(DesmosifyError::new(
                        String::from("expected an operator"),
                        Some(token.start),
                        Some(token.end),
                    ));
                }
                operands.push(Expression {
                    data_type: DataType::Unknown, // TODO
                    start_token: Some(self.token_index),
                    op_token: None,
                    end_token: Some(self.token_index),
                    constant: true,
                    value: ExpressionValue::from_token_value(&token.value).unwrap(),
                });
                expect_operand = false;
            }
            self.next();
        }
        let token = self.token()?;
        if expect_operand {
            return Err(DesmosifyError::new(
                String::from("expected an operand"),
                Some(token.start),
                Some(token.start),
            ));
        }
        while !operators.is_empty() {
            self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
        }
        if operands.len() != 1 {
            Err(DesmosifyError::new(
                format!(
                    "expression resolved to {} operands instead of 1 as expected",
                    operands.len()
                ),
                Some(token.start),
                Some(token.start),
            ))
        } else {
            Ok(operands.pop().unwrap())
        }
    }

    fn parse_signature(&mut self) -> Result<(String, Vec<Parameter>), DesmosifyError> {
        let name = self.get_string_from_name()?;
        let mut parameters = Vec::new();
        self.next();
        if self.is_at_symbol(Symbol::ParenLeft)? {
            self.next();
            while !self.is_at_symbol(Symbol::ParenRight)? {
                let parameter_name = self.get_string_from_name()?;
                let parameter_type = DataType::Unknown;
                self.next();
                let token = self.token()?;
                if self.is_at_symbol(Symbol::Colon)? {
                    self.next();
                    let _type_expression =
                        self.parse_expression(&[Symbol::Comma, Symbol::ParenRight], &[])?;
                    // TODO: actually parse type
                } else if !token.is_one_of(&[Symbol::Comma, Symbol::ParenRight], &[]) {
                    return Err(DesmosifyError::new(
                        String::from("expected ',' or ')'"),
                        Some(token.start),
                        Some(token.end),
                    ));
                }
                parameters.push(Parameter {
                    name: parameter_name,
                    data_type: parameter_type,
                });
                if self.is_at_symbol(Symbol::Comma)? {
                    self.next();
                }
            }
            self.next();
        }
        Ok((name, parameters))
    }

    fn parse_action(&mut self, allow_inline: bool) -> Result<Action, DesmosifyError> {
        if !self.is_at_symbol(Symbol::CurlyLeft)? {
            return if allow_inline {
                let action_expr =
                    self.parse_expression(&[Symbol::Comma, Symbol::CurlyRight], &[])?;
                match action_expr.value {
                    ExpressionValue::Operator(Operation::Update, mut operands) => {
                        let value = operands.pop().unwrap();
                        let name = operands.pop().unwrap();
                        Ok(Action::Update(Box::new(name), Box::new(value)))
                    }
                    ExpressionValue::Operator(Operation::Call, mut operands) => {
                        let arguments: Vec<_> = operands.drain(1..).collect();
                        let name = operands.pop().unwrap();
                        Ok(Action::Call(Box::new(name), arguments))
                    }
                    _ => Err(DesmosifyError::new(
                        String::from("expected either an action call or a variable update"),
                        None,
                        None,
                    )),
                }
            } else {
                let token = self.token()?;
                Err(DesmosifyError::new(
                    String::from("expected '{'"),
                    Some(token.start),
                    Some(token.end),
                ))
            };
        }
        self.next();
        let mut action = Vec::new();
        while !self.is_at_symbol(Symbol::CurlyRight)? {
            if self.is_at_keyword(Keyword::If)? {
                let mut branches = Vec::new();
                self.next();
                let condition = self.parse_expression(&[Symbol::Colon], &[])?;
                self.next();
                branches.push((condition, self.parse_action(true)?));
                self.next();
                while self.is_at_keyword(Keyword::Elif)? {
                    self.next();
                    let condition = self.parse_expression(&[Symbol::Colon], &[])?;
                    self.next();
                    branches.push((condition, self.parse_action(true)?));
                    self.next();
                }
                action.push(Action::Conditional(
                    branches,
                    if self.is_at_keyword(Keyword::Else)? {
                        self.next();
                        if !self.is_at_symbol(Symbol::Colon)? {
                            let token = self.token()?;
                            return Err(DesmosifyError::new(
                                String::from("expected ':'"),
                                Some(token.start),
                                Some(token.end),
                            ));
                        }
                        self.next();
                        let default_branch = self.parse_action(true)?;
                        self.next();
                        Some(Box::new(default_branch))
                    } else {
                        None
                    },
                ))
            } else {
                action.push(self.parse_action(true)?);
            }
            while self.is_at_symbol(Symbol::Comma)? {
                self.next();
            }
        }
        Ok(if action.len() == 1 {
            action.pop().unwrap()
        } else {
            Action::Block(action)
        })
    }
}

pub fn parse_definitions(tokens: &[Token]) -> Result<Definitions, DesmosifyError> {
    let mut definitions = Definitions {
        public: None,
        ticker: None,
        namespace: BTreeMap::new(),
    };
    let mut parser = Parser::new(tokens);
    while parser.token_index < parser.tokens.len() {
        let token = &parser.tokens[parser.token_index];
        match token.value {
            TokenValue::Symbol(Symbol::Semicolon) => {},
            TokenValue::Keyword(Keyword::Public) => {
                if definitions.public.is_some() {
                    return Err(DesmosifyError::new(String::from("only one 'public' block can be declared"), Some(token.start), Some(token.end)));
                }
                parser.next();
                if !parser.is_at_symbol(Symbol::CurlyLeft)? {
                    let token = parser.token()?;
                    return Err(DesmosifyError::new(String::from("expected '{'"), Some(token.start), Some(token.end)));
                }
                parser.next();
                let mut public = Vec::new();
                while !parser.is_at_symbol(Symbol::CurlyRight)? {
                    public.push(parser.parse_expression(&[Symbol::Comma, Symbol::CurlyRight], &[])?);
                    while parser.is_at_symbol(Symbol::Comma)? {
                        parser.next();
                    }
                }
                definitions.public = Some(public);
            },
            TokenValue::Keyword(Keyword::Ticker) => {
                if definitions.ticker.is_some() {
                    return Err(DesmosifyError::new(String::from("only one 'ticker' block can be declared"), Some(token.start), Some(token.end)));
                }
                parser.next();
                let mut interval = None;
                if parser.is_at_symbol(Symbol::ParenLeft)? {
                    parser.next();
                    interval = Some(Box::new(parser.parse_expression(&[Symbol::ParenRight], &[])?));
                    parser.next();
                }
                let tick_action = Box::new(parser.parse_action(false)?);
                definitions.ticker = Some((interval, tick_action));
            },
            TokenValue::Keyword(Keyword::Action) => {
                parser.next();
                let (name, parameters) = parser.parse_signature()?;
                let action = Box::new(parser.parse_action(false)?);
                let identifier = Identifier::Action(name.clone(), parameters, action);
                definitions.namespace.insert(name, identifier);
            },
            TokenValue::Keyword(Keyword::Const) => {
                parser.next();
                let (name, parameters) = parser.parse_signature()?;
                let mut data_type = None;
                if parser.is_at_symbol(Symbol::Colon)? {
                    parser.next();
                    data_type = Some(Box::new(parser.parse_expression(&[Symbol::Equal], &[])?));
                } else if !parser.is_at_symbol(Symbol::Equal)? {
                    let token = parser.token()?;
                    return Err(DesmosifyError::new(String::from("expected '='"), Some(token.start), Some(token.end)));
                }
                parser.next();
                let value = Box::new(parser.parse_expression(&[Symbol::Semicolon], &[])?);
                let identifier = Identifier::Const(name.clone(), parameters, data_type, value);
                definitions.namespace.insert(name, identifier);
            },
            TokenValue::Keyword(Keyword::Let) => {
                parser.next();
                let (name, parameters) = parser.parse_signature()?;
                let mut data_type = None;
                if parser.is_at_symbol(Symbol::Colon)? {
                    parser.next();
                    data_type = Some(Box::new(parser.parse_expression(&[Symbol::Equal], &[])?));
                } else if !parser.is_at_symbol(Symbol::Equal)? {
                    let token = parser.token()?;
                    return Err(DesmosifyError::new(String::from("expected '='"), Some(token.start), Some(token.end)));
                }
                parser.next();
                let value = Box::new(parser.parse_expression(&[Symbol::Semicolon], &[])?);
                let identifier = Identifier::Let(name.clone(), parameters, data_type, value);
                definitions.namespace.insert(name, identifier);
            },
            TokenValue::Keyword(Keyword::Var) => {
                parser.next();
                let name = parser.get_string_from_name()?;
                parser.next();
                let mut data_type = None;
                if parser.is_at_symbol(Symbol::Colon)? {
                    parser.next();
                    data_type = Some(Box::new(parser.parse_expression(&[Symbol::Equal], &[])?));
                } else if !parser.is_at_symbol(Symbol::Equal)? {
                    let token = parser.token()?;
                    return Err(DesmosifyError::new(String::from("expected '='"), Some(token.start), Some(token.end)));
                }
                parser.next();
                let value = Box::new(parser.parse_expression(&[Symbol::Semicolon], &[])?);
                let identifier = Identifier::Var(name.clone(), data_type, value);
                definitions.namespace.insert(name, identifier);
            },
            _ => return Err(DesmosifyError::new(String::from("expected 'const', 'let', 'var', 'action', 'public', or 'ticker'"), Some(token.start), Some(token.end)))
        }
        parser.next();
    }
    Ok(definitions)
}
