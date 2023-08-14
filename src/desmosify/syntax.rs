use super::*;

fn message_expected_one_of(symbols: &[Symbol], keywords: &[Keyword]) -> String {
    let count = symbols.len() + keywords.len();
    let mut message = String::from("expected ");
    symbols.iter().map(|symbol| symbol.literal())
        .chain(keywords.iter().map(|keyword| keyword.literal()))
        .enumerate()
        .for_each(|(index, literal)| {
            if index < count - 1 {
                message.push('\'');
                message.push_str(literal);
                message.push('\'');
                if count > 2 {
                    message.push(',');
                }
                message.push(' ');
            } else {
                if count > 1 {
                    message.push_str("or ");
                }
                message.push('\'');
                message.push_str(literal);
                message.push('\'');
            }
        });
    message
}

fn message_identifier_conflict(original: Identifier) -> String {
    format!("name conflicts with previous '{} {}'", original.variant_name(), original.name())
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum Precedence {
    // Lowest
    With,
    Assignment,
    Logical,
    Equality,
    Comparison,
    Additive,
    Multiplicative,
    Exponent,
    Prefix,
    Postfix,
    Access,
    Container,
    // Highest
}

impl Precedence {
    pub fn is_left_to_right_associative(self) -> bool {
        match self {
            Self::Prefix | Self::Assignment | Self::With => false,
            _ => true
        }
    }

    pub fn precedes(self, rhs: Self) -> bool {
        let left_precedence = self as isize;
        let right_precedence = rhs as isize;
        left_precedence > right_precedence
            || (left_precedence == right_precedence
                && self.is_left_to_right_associative())
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Operation {
    PointLiteral,
    ListLiteral,
    ListFill,
    ListMap,
    ListFilter,
    MemberAccess,
    BuiltIn,
    Call,
    ActionCall,
    Index,
    Posate,
    Negate,
    Not,
    Exponent,
    Multiply,
    Divide,
    Modulus,
    Add,
    Subtract,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
    And,
    Or,
    ExclusiveRange,
    InclusiveRange,
    Conditional,
    Assignment,
    Update,
    With,
}

impl Operation {
    pub fn from_symbol(symbol: Symbol, expect_operand: bool) -> Option<Self> {
        Some(match symbol {
            Symbol::Plus => if expect_operand {
                Self::Posate
            } else {
                Self::Add
            },
            Symbol::Minus => if expect_operand {
                Self::Negate
            } else {
                Self::Subtract
            },
            Symbol::Bang => if expect_operand {
                Self::Not
            } else {
                return None;
            },
            Symbol::AtSign => if expect_operand {
                Self::BuiltIn
            } else {
                return None;
            },
            _ if expect_operand => return None,
            Symbol::Star => Self::Multiply,
            Symbol::Slash => Self::Divide,
            Symbol::Percent => Self::Modulus,
            Symbol::Caret => Self::Exponent,
            Symbol::Ampersand2 => Self::And,
            Symbol::Pipe2 => Self::Or,
            Symbol::Equal2 => Self::Equal,
            Symbol::NotEqual => Self::NotEqual,
            Symbol::LessThan => Self::LessThan,
            Symbol::GreaterThan => Self::GreaterThan,
            Symbol::LessEqual => Self::LessEqual,
            Symbol::GreaterEqual => Self::GreaterEqual,
            Symbol::Equal => Self::Assignment,
            Symbol::ColonEqual => Self::Update,
            Symbol::Dot => Self::MemberAccess,
            Symbol::ParenLeft => Self::Call,
            Symbol::SquareLeft => Self::Index,
            _ => return None
        })
    }

    pub fn from_keyword(keyword: Keyword, expect_operand: bool) -> Option<Self> {
        Some(match keyword {
            Keyword::Action => if expect_operand {
                Self::ActionCall
            } else {
                return None;
            },
            _ if expect_operand => return None,
            Keyword::With => Self::With,
            _ => return None
        })
    }

    pub fn precedence(self) -> Precedence {
        use Operation::*;
        match self {
            PointLiteral | ListLiteral | ListFill | ListMap | ListFilter
            | Conditional | ExclusiveRange | InclusiveRange
                => Precedence::Container,
            MemberAccess | BuiltIn
                => Precedence::Access,
            Call | ActionCall | Index
                => Precedence::Postfix,
            Posate | Negate | Not
                => Precedence::Prefix,
            Exponent
                => Precedence::Exponent,
            Multiply | Divide | Modulus
                => Precedence::Multiplicative,
            Add | Subtract
                => Precedence::Additive,
            LessThan | GreaterThan | LessEqual | GreaterEqual
                => Precedence::Comparison,
            Equal | NotEqual
                => Precedence::Equality,
            And | Or
                => Precedence::Logical,
            Assignment | Update
                => Precedence::Assignment,
            With
                => Precedence::With,
        }
    }

    pub fn precedes(self, rhs: Self) -> bool {
        self.precedence().precedes(rhs.precedence())
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
    pub data_type: DataType,
    pub value: ExpressionValue,
    pub start_token: Option<usize>,
    pub op_token: Option<usize>,
    pub end_token: Option<usize>,
    pub constant: bool,
}

impl Expression {
    pub fn from_constant(value: DataValue) -> Self {
        Self {
            data_type: value.data_type(),
            value: ExpressionValue::Literal(value),
            start_token: None,
            op_token: None,
            end_token: None,
            constant: true,
        }
    }
}

#[derive(Debug)]
pub struct Parser<'a> {
    tokens: &'a [Token],
    token_index: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            token_index: 0,
        }
    }

    pub fn token(&self) -> Result<&'a Token, DesmosifyError> {
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

    pub fn next(&mut self) {
        self.token_index += 1;
    }

    pub fn is_at_symbol(&self, value: Symbol) -> Result<bool, DesmosifyError> {
        if let TokenValue::Symbol(symbol) = self.token()?.value {
            Ok(symbol == value)
        } else {
            Ok(false)
        }
    }

    pub fn is_at_keyword(&self, value: Keyword) -> Result<bool, DesmosifyError> {
        if let TokenValue::Keyword(keyword) = self.token()?.value {
            Ok(keyword == value)
        } else {
            Ok(false)
        }
    }

    pub fn is_at_one_of(&self, symbols: &[Symbol], keywords: &[Keyword]) -> Result<bool, DesmosifyError> {
        Ok(self.token()?.is_one_of(symbols, keywords))
    }

    pub fn expect_symbol(&self, value: Symbol) -> Result<(), DesmosifyError> {
        if self.is_at_symbol(value)? {
            Ok(())
        } else {
            let token = self.token()?;
            Err(DesmosifyError::new(
                format!("expected '{value}'"),
                Some(token.start),
                Some(token.end),
            ))
        }
    }

    pub fn expect_keyword(&self, value: Keyword) -> Result<(), DesmosifyError> {
        if self.is_at_keyword(value)? {
            Ok(())
        } else {
            let token = self.token()?;
            Err(DesmosifyError::new(
                format!("expected '{value}'"),
                Some(token.start),
                Some(token.end),
            ))
        }
    }

    pub fn expect_one_of(&self, symbols: &[Symbol], keywords: &[Keyword]) -> Result<(), DesmosifyError> {
        if self.is_at_one_of(symbols, keywords)? {
            Ok(())
        } else {
            let token = self.token()?;
            Err(DesmosifyError::new(
                message_expected_one_of(symbols, keywords),
                Some(token.start),
                Some(token.end),
            ))
        }
    }

    pub fn expect_name(&self) -> Result<String, DesmosifyError> {
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

    pub fn expect_string(&self) -> Result<String, DesmosifyError> {
        let token = self.token()?;
        if let TokenValue::String(name) = &token.value {
            Ok(name.clone())
        } else {
            Err(DesmosifyError::new(
                String::from("expected a string"),
                Some(token.start),
                Some(token.end),
            ))
        }
    }

    pub fn get_constant_string(&self, expression: Expression) -> Result<String, DesmosifyError> {
        if let ExpressionValue::Literal(DataValue::Str(string)) = expression.value {
            Ok(string)
        } else {
            Err(DesmosifyError::new(
                String::from("expected a string"),
                expression.start_token.map(|index| self.tokens[index].start),
                expression.end_token.map(|index| self.tokens[index].end),
            ))
        }
    }

    fn get_operation(&self, expect_operand: bool) -> Option<Operation> {
        let token = self.token().ok()?;
        if let TokenValue::Symbol(symbol) = token.value {
            Operation::from_symbol(symbol, expect_operand)
        } else if let TokenValue::Keyword(keyword) = token.value {
            Operation::from_keyword(keyword, expect_operand)
        } else {
            None
        }
    }

    fn wrap_top_operator_into_operand(&mut self, operators: &mut Vec<(Operation, usize)>, operands: &mut Vec<Expression>) -> Result<(), DesmosifyError> {
        let (operation, operand_count) = operators.pop().unwrap();
        if operands.len() < operand_count {
            Err(DesmosifyError::new(
                format!(
                    "too few operands for operation (expected {operand_count}, got {})",
                    operands.len(),
                ),
                Some(self.token()?.start),
                Some(self.token()?.start),
            ))
        } else {
            let child_operands: Vec<_> = operands
                .splice((operands.len() - operand_count)..operands.len(), [])
                .collect();
            operands.push(Expression {
                data_type: DataType::Unknown,
                start_token: child_operands
                    .first()
                    .map_or(None, |first| first.start_token),
                op_token: None, // TODO
                end_token: child_operands.last().map_or(None, |last| last.end_token),
                constant: false,
                value: ExpressionValue::Operator(operation, child_operands),
            });
            Ok(())
        }
    }

    pub fn parse_expression(&mut self, end_symbols: &[Symbol], end_keywords: &[Keyword]) -> Result<Expression, DesmosifyError> {
        let mut operators: Vec<(Operation, usize)> = Vec::new();
        let mut operands: Vec<Expression> = Vec::new();
        let mut expect_operand = true;
        'main_expression_loop: loop {
            let token = self.token()?;
            if (!expect_operand || operands.is_empty()) && token.is_one_of(end_symbols, end_keywords) {
                break 'main_expression_loop;
            }
            else if token.is_symbol_or_keyword() {
                let mut operand_count: usize = 2;

                // TODO: build this into the normal path
                if expect_operand {
                    if self.is_at_symbol(Symbol::ParenLeft)? {
                        expect_operand = false;
                        self.next();
                        operands.push(self.parse_expression(&[Symbol::Comma, Symbol::ParenRight], &[])?);
                        if self.is_at_symbol(Symbol::Comma)? {
                            self.next();
                            operands.push(self.parse_expression(&[Symbol::ParenRight], &[])?);
                            operators.push((Operation::PointLiteral, 2));
                            self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                        }
                        self.next();
                        continue 'main_expression_loop;
                    }
                    else if self.is_at_symbol(Symbol::SquareLeft)? {
                        expect_operand = false;
                        operand_count = 0;
                        self.next();
                        while !self.is_at_symbol(Symbol::SquareRight)? {
                            operands.push(self.parse_expression(&[
                                Symbol::Comma,
                                Symbol::SquareRight,
                                Symbol::Semicolon,
                                Symbol::ExclusiveRange,
                                Symbol::InclusiveRange,
                            ], &[
                                Keyword::For,
                                Keyword::Where,
                            ])?);
                            operand_count += 1;
                            if self.is_at_symbol(Symbol::Comma)? {
                                self.next();
                            }
                            else if self.is_at_symbol(Symbol::Semicolon)? {
                                if operand_count != 1 {
                                    let token = self.token()?;
                                    return Err(DesmosifyError::new(
                                        String::from("unexpected ';'"),
                                        Some(token.start),
                                        Some(token.end),
                                    ));
                                }
                                self.next();
                                operands.push(self.parse_expression(&[Symbol::SquareRight], &[])?);
                                operators.push((Operation::ListFill, 2));
                                self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                                self.next();
                                continue 'main_expression_loop;
                            }
                            else if self.is_at_keyword(Keyword::For)? || self.is_at_keyword(Keyword::Where)? {
                                if operand_count != 1 {
                                    let token = self.token()?;
                                    return Err(DesmosifyError::new(
                                        String::from("list comprehension following multiple elements"),
                                        Some(token.start),
                                        Some(token.end),
                                    ));
                                }
                                if self.is_at_keyword(Keyword::Where)? {
                                    self.next();
                                    operands.push(self.parse_expression(&[Symbol::SquareRight], &[])?);
                                    operators.push((Operation::ListFilter, 2));
                                    self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                                } else {
                                    while self.is_at_keyword(Keyword::For)? {
                                        self.next();
                                        operands.push(self.parse_expression(&[], &[Keyword::In])?);
                                        self.next();
                                        operands.push(self.parse_expression(&[Symbol::SquareRight], &[Keyword::For])?);
                                        operators.push((Operation::ListMap, 3));
                                        self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                                    }
                                }
                                self.next();
                                continue 'main_expression_loop;
                            }
                            else if self.is_at_symbol(Symbol::ExclusiveRange)? || self.is_at_symbol(Symbol::InclusiveRange)? {
                                let range_operation = if self.is_at_symbol(Symbol::ExclusiveRange)? {
                                    Operation::ExclusiveRange
                                } else {
                                    Operation::InclusiveRange
                                };
                                self.next();
                                operands.push(self.parse_expression(&[Symbol::SquareRight], &[])?);
                                operators.push((range_operation, operand_count + 1));
                                self.wrap_top_operator_into_operand(
                                    &mut operators,
                                    &mut operands,
                                )?;
                                self.next();
                                continue 'main_expression_loop;
                            }
                        }
                        operators.push((Operation::ListLiteral, operand_count));
                        self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                        self.next();
                        continue 'main_expression_loop;
                    }
                    else if self.is_at_symbol(Symbol::CurlyLeft)? {
                        expect_operand = false;
                        self.next();
                        operands.push(self.parse_expression(&[Symbol::Colon], &[])?);
                        operand_count = 1;
                        while self.is_at_symbol(Symbol::Colon)? {
                            self.next();
                            operands.push(self.parse_expression(&[Symbol::Comma, Symbol::CurlyRight], &[])?);
                            operand_count += 1;
                            if self.is_at_symbol(Symbol::Comma)? {
                                self.next();
                            }
                            if self.is_at_symbol(Symbol::CurlyRight)? {
                                break;
                            }
                            operands.push(self.parse_expression(&[Symbol::Colon, Symbol::Comma, Symbol::CurlyRight], &[])?);
                            operand_count += 1;
                        }
                        if !self.is_at_symbol(Symbol::CurlyRight)? {
                            self.next();
                            self.expect_symbol(Symbol::CurlyRight)?;
                        }
                        operators.push((Operation::Conditional, operand_count));
                        self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                        self.next();
                        continue 'main_expression_loop;
                    }
                }

                let operation = self.get_operation(expect_operand).ok_or_else(|| DesmosifyError::new(
                    if expect_operand {
                        String::from("expected an operand")
                    } else {
                        message_expected_one_of(end_symbols, end_keywords)
                    },
                    Some(token.start),
                    Some(token.end),
                ))?;
                while operators.last().map_or(false, |(lhs, _)| lhs.precedes(operation)) {
                    self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                }

                if operation == Operation::ActionCall {
                    self.next();
                    operands.push(Expression {
                        data_type: DataType::Action,
                        start_token: Some(self.token_index - 1),
                        op_token: None,
                        end_token: Some(self.token_index),
                        constant: false,
                        value: ExpressionValue::Name(self.expect_name()?),
                    });
                    self.next();
                }

                match operation {
                    Operation::Call | Operation::ActionCall => {
                        let mut arguments = self.parse_call()?;
                        operand_count = 1 + arguments.len();
                        operands.append(&mut arguments);
                    }
                    Operation::Index => {
                        // TODO: index ranges
                        self.next();
                        operands.push(self.parse_expression(&[Symbol::SquareRight], &[])?);
                    }
                    Operation::BuiltIn | Operation::Posate | Operation::Negate | Operation::Not => {
                        operand_count = 1;
                    }
                    _ => {}
                }

                operators.push((operation, operand_count));
                expect_operand = match operation {
                    Operation::Call | Operation::ActionCall | Operation::Index => {
                        self.wrap_top_operator_into_operand(&mut operators, &mut operands)?;
                        false
                    },
                    _ => true
                }
            }
            else {
                if !expect_operand {
                    return Err(DesmosifyError::new(
                        message_expected_one_of(end_symbols, end_keywords),
                        Some(token.start),
                        Some(token.end),
                    ));
                }
                operands.push(Expression {
                    data_type: DataType::Unknown,
                    start_token: Some(self.token_index),
                    op_token: None,
                    end_token: Some(self.token_index),
                    constant: false,
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

    pub fn parse_call(&mut self) -> Result<Vec<Expression>, DesmosifyError> {
        self.expect_symbol(Symbol::ParenLeft)?;
        self.next();
        let mut arguments = Vec::new();
        while !self.is_at_symbol(Symbol::ParenRight)? {
            arguments.push(self.parse_expression(&[Symbol::Comma, Symbol::ParenRight], &[])?);
            if self.is_at_symbol(Symbol::Comma)? {
                self.next();
            }
        }
        Ok(arguments)
    }

    pub fn parse_type(&mut self, end_symbols: &[Symbol], end_keywords: &[Keyword]) -> Result<DataType, DesmosifyError> {
        let is_list_type = if self.is_at_symbol(Symbol::SquareLeft)? {
            self.next();
            true
        } else {
            false
        };
        let mut data_type = if self.is_at_symbol(Symbol::Question)? {
            DataType::Unknown
        } else {
            DataType::from_name(&self.expect_name()?)
        };
        self.next();
        if is_list_type {
            self.expect_symbol(Symbol::SquareRight)?;
            self.next();
            data_type = DataType::List(Box::new(data_type));
        }
        self.expect_one_of(end_symbols, end_keywords)?;
        Ok(data_type)
    }

    pub fn parse_signature(&mut self) -> Result<Vec<Parameter>, DesmosifyError> {
        self.next();
        let mut parameters = Vec::new();
        if self.is_at_symbol(Symbol::ParenLeft)? {
            self.next();
            while !self.is_at_symbol(Symbol::ParenRight)? {
                let parameter_name = self.expect_name()?;
                let mut parameter_type = DataType::Unknown;
                self.next();
                self.expect_one_of(&[Symbol::Colon, Symbol::Comma, Symbol::ParenRight], &[])?;
                if self.is_at_symbol(Symbol::Colon)? {
                    self.next();
                    parameter_type = self.parse_type(&[Symbol::Comma, Symbol::ParenRight], &[])?;
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
        Ok(parameters)
    }

    pub fn parse_action(&mut self, allow_inline: bool) -> Result<Action, DesmosifyError> {
        if !allow_inline {
            self.expect_symbol(Symbol::CurlyLeft)?;
        } else if !self.is_at_symbol(Symbol::CurlyLeft)? {
            let action_expr =
                self.parse_expression(&[Symbol::Comma, Symbol::CurlyRight], &[])?;
            return match action_expr.value {
                ExpressionValue::Operator(Operation::Update, mut operands) => {
                    let value = operands.pop().unwrap();
                    let name = operands.pop().unwrap();
                    Ok(Action::Update(Box::new(name), Box::new(value)))
                }
                ExpressionValue::Operator(Operation::ActionCall, mut operands) => {
                    let arguments: Vec<_> = operands.drain(1..).collect();
                    let name = operands.pop().unwrap();
                    Ok(Action::Call(Box::new(name), arguments))
                }
                _ => Err(DesmosifyError::new(
                    String::from("expected action call or variable update"),
                    None,
                    None,
                )),
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
                        self.expect_symbol(Symbol::Colon)?;
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
            if self.is_at_symbol(Symbol::Comma)? {
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
    let mut definitions = Definitions::new();
    let mut parser = Parser::new(tokens);
    while parser.token_index < parser.tokens.len() {
        let token = &parser.tokens[parser.token_index];
        parser.expect_one_of(&[
            Symbol::Semicolon,
        ], &[
            Keyword::Public,
            Keyword::Ticker,
            Keyword::Display,
            Keyword::Action,
            Keyword::Const,
            Keyword::Let,
            Keyword::Var,
            Keyword::Enum,
        ])?;
        match token.value {
            TokenValue::Symbol(Symbol::Semicolon) => {},
            TokenValue::Keyword(Keyword::Public) => {
                if definitions.public.is_some() {
                    return Err(DesmosifyError::new(String::from("only one 'public' block can be declared"), Some(token.start), Some(token.end)));
                }
                parser.next();
                parser.expect_symbol(Symbol::CurlyLeft)?;
                parser.next();
                let mut public = Vec::new();
                while !parser.is_at_symbol(Symbol::CurlyRight)? {
                    public.push(parser.parse_expression(&[Symbol::Semicolon, Symbol::CurlyRight], &[])?);
                    while parser.is_at_symbol(Symbol::Semicolon)? {
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
                let mut interval_ms = None;
                if parser.is_at_symbol(Symbol::ParenLeft)? {
                    parser.next();
                    interval_ms = Some(Box::new(parser.parse_expression(&[Symbol::ParenRight], &[])?));
                    parser.next();
                }
                let tick_action = Box::new(parser.parse_action(false)?);
                definitions.ticker = Some(Ticker { interval_ms, tick_action });
            },
            TokenValue::Keyword(Keyword::Display) => {
                if definitions.display.is_some() {
                    return Err(DesmosifyError::new(String::from("only one 'display' block can be declared"), Some(token.start), Some(token.end)));
                }
                parser.next();
                parser.expect_symbol(Symbol::CurlyLeft)?;
                parser.next();
                let mut display = Vec::new();
                while !parser.is_at_symbol(Symbol::CurlyRight)? {
                    display.push(display::Element::parse(&mut parser)?);
                    while parser.is_at_symbol(Symbol::Semicolon)? {
                        parser.next();
                    }
                }
                definitions.display = Some(display);
            },
            TokenValue::Keyword(Keyword::Action) => {
                parser.next();
                let (name_start, name_end) = (parser.token()?.start, parser.token()?.end);
                let name = parser.expect_name()?;
                let parameters = parser.parse_signature()?;
                let content = Box::new(parser.parse_action(false)?);
                let identifier = Identifier::Action { name: name.clone(), parameters, content };
                if let Some(original) = definitions.identifiers.insert(name, identifier) {
                    return Err(DesmosifyError::new(
                        message_identifier_conflict(original),
                        Some(name_start),
                        Some(name_end),
                    ));
                }
            },
            TokenValue::Keyword(Keyword::Const) => {
                parser.next();
                let (name_start, name_end) = (parser.token()?.start, parser.token()?.end);
                let name = parser.expect_name()?;
                let parameters = parser.parse_signature()?;
                let mut value_type = DataType::Unknown;
                if parser.is_at_symbol(Symbol::Colon)? {
                    parser.next();
                    value_type = parser.parse_type(&[Symbol::Equal], &[])?;
                } else {
                    parser.expect_symbol(Symbol::Equal)?;
                }
                parser.next();
                let value = Box::new(parser.parse_expression(&[Symbol::Semicolon], &[])?);
                let identifier = Identifier::Const { name: name.clone(), parameters, value_type, value };
                if let Some(original) = definitions.identifiers.insert(name, identifier) {
                    return Err(DesmosifyError::new(
                        message_identifier_conflict(original),
                        Some(name_start),
                        Some(name_end),
                    ));
                }
            },
            TokenValue::Keyword(Keyword::Let) => {
                parser.next();
                let (name_start, name_end) = (parser.token()?.start, parser.token()?.end);
                let name = parser.expect_name()?;
                let parameters = parser.parse_signature()?;
                let mut value_type = DataType::Unknown;
                if parser.is_at_symbol(Symbol::Colon)? {
                    parser.next();
                    value_type = parser.parse_type(&[Symbol::Equal], &[])?;
                } else {
                    parser.expect_symbol(Symbol::Equal)?;
                }
                parser.next();
                let value = Box::new(parser.parse_expression(&[Symbol::Semicolon], &[])?);
                let identifier = Identifier::Let { name: name.clone(), parameters, value_type, value };
                if let Some(original) = definitions.identifiers.insert(name, identifier) {
                    return Err(DesmosifyError::new(
                        message_identifier_conflict(original),
                        Some(name_start),
                        Some(name_end),
                    ));
                }
            },
            TokenValue::Keyword(Keyword::Var) => {
                parser.next();
                let qualifier = if parser.is_at_keyword(Keyword::Timer)? {
                    parser.next();
                    Some(VariableQualifier::Timer)
                } else {
                    None
                };
                let (name_start, name_end) = (parser.token()?.start, parser.token()?.end);
                let name = parser.expect_name()?;
                parser.next();
                let mut value_type = DataType::Unknown;
                if parser.is_at_symbol(Symbol::Colon)? {
                    parser.next();
                    value_type = parser.parse_type(&[Symbol::Equal], &[])?;
                } else {
                    parser.expect_symbol(Symbol::Equal)?;
                }
                parser.next();
                let value = Box::new(parser.parse_expression(&[Symbol::Semicolon], &[])?);
                let identifier = Identifier::Var { name: name.clone(), qualifier, value_type, value };
                if let Some(original) = definitions.identifiers.insert(name, identifier) {
                    return Err(DesmosifyError::new(
                        message_identifier_conflict(original),
                        Some(name_start),
                        Some(name_end),
                    ));
                }
            },
            TokenValue::Keyword(Keyword::Enum) => {
                parser.next();
                let (name_start, name_end) = (parser.token()?.start, parser.token()?.end);
                let name = parser.expect_name()?;
                parser.next();
                parser.expect_symbol(Symbol::CurlyLeft)?;
                parser.next();
                let mut variants = Vec::new();
                while !parser.is_at_symbol(Symbol::CurlyRight)? {
                    variants.push(parser.expect_name()?);
                    parser.next();
                    // TODO: =
                    parser.expect_one_of(&[Symbol::Comma, Symbol::CurlyRight], &[])?;
                    if parser.is_at_symbol(Symbol::Comma)? {
                        parser.next();
                    }
                }
                let identifier = Identifier::Enum { name: name.clone(), variants };
                if let Some(original) = definitions.identifiers.insert(name, identifier) {
                    return Err(DesmosifyError::new(
                        message_identifier_conflict(original),
                        Some(name_start),
                        Some(name_end),
                    ));
                }
            }
            _ => unreachable!()
        }
        parser.next();
    }
    Ok(definitions)
}
