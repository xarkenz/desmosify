use std::rc::Rc;
use crate::token::{Literal, TokenKind};

pub mod parse;

#[derive(Clone, Debug)]
pub enum TypeExpressionKind {
    Any,
    Identifier(Rc<str>),
    Grouping {
        expression: Box<TypeExpression>,
    },
    List {
        item_type: Box<TypeExpression>,
    },
    Broadcastable {
        item_type: Box<TypeExpression>,
    },
    Point2 {
        x_type: Box<TypeExpression>,
        y_type: Box<TypeExpression>,
    },
    Point3 {
        x_type: Box<TypeExpression>,
        y_type: Box<TypeExpression>,
        z_type: Box<TypeExpression>,
    },
}

impl std::fmt::Display for TypeExpressionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => {
                write!(f, "?")
            }
            Self::Identifier(identifier) => {
                write!(f, "{identifier}")
            }
            Self::Grouping { expression } => {
                write!(f, "({expression})")
            }
            Self::List { item_type } => {
                write!(f, "[{item_type}]")
            }
            Self::Broadcastable { item_type } => {
                write!(f, "{item_type}+")
            }
            Self::Point2 { x_type, y_type } => {
                write!(f, "({x_type}, {y_type})")
            }
            Self::Point3 { x_type, y_type, z_type } => {
                write!(f, "({x_type}, {y_type}, {z_type})")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeExpression {
    pub kind: TypeExpressionKind,
    pub span: crate::Span,
}

impl std::fmt::Display for TypeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Associativity {
    LeftToRight,
    RightToLeft,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Precedence {
    // Highest
    Postfix,
    Prefix,
    Exponential,
    Multiplicative,
    Additive,
    Comparison,
    Equality,
    LogicalAnd,
    LogicalOr,
    Assignment,
    // Lowest
}

impl Precedence {
    pub fn associativity(&self) -> Associativity {
        match self {
            Self::Prefix | Self::Assignment => Associativity::RightToLeft,
            _ => Associativity::LeftToRight
        }
    }
}

impl PartialOrd for Precedence {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Compare the internal value, then reverse the ordering because
        // the lowest internal value represents the highest precedence
        (*self as isize).partial_cmp(&(*other as isize))
            .map(std::cmp::Ordering::reverse)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UnaryOperation {
    Positive,
    Negative,
    LogicalNot,
}

impl UnaryOperation {
    pub fn precedence(&self) -> Precedence {
        match self {
            Self::Positive |
            Self::Negative |
            Self::LogicalNot => Precedence::Prefix,
        }
    }

    pub fn associativity(&self) -> Associativity {
        self.precedence().associativity()
    }

    pub fn from_prefix_token(token: &TokenKind) -> Option<Self> {
        match token {
            TokenKind::Plus => Some(Self::Positive),
            TokenKind::Minus => Some(Self::Negative),
            TokenKind::Bang => Some(Self::LogicalNot),
            _ => None
        }
    }

    pub fn from_postfix_token(token: &TokenKind) -> Option<Self> {
        // Not currently used
        let _ = token;
        None
    }

    pub fn fmt_with_operand(&self, f: &mut std::fmt::Formatter, operand: &ExpressionKind) -> std::fmt::Result {
        match self {
            Self::Positive => write!(f, "+{operand}"),
            Self::Negative => write!(f, "-{operand}"),
            Self::LogicalNot => write!(f, "!{operand}"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BinaryOperation {
    MemberAccess,
    Exponent,
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Equal,
    NotEqual,
    LogicalAnd,
    LogicalOr,
}

impl BinaryOperation {
    pub fn precedence(&self) -> Precedence {
        match self {
            Self::MemberAccess => Precedence::Postfix,
            Self::Exponent => Precedence::Exponential,
            Self::Multiply |
            Self::Divide |
            Self::Remainder => Precedence::Multiplicative,
            Self::Add |
            Self::Subtract => Precedence::Additive,
            Self::LessThan |
            Self::LessEqual |
            Self::GreaterThan |
            Self::GreaterEqual => Precedence::Comparison,
            Self::Equal |
            Self::NotEqual => Precedence::Equality,
            Self::LogicalAnd => Precedence::LogicalAnd,
            Self::LogicalOr => Precedence::LogicalOr,
        }
    }

    pub fn associativity(&self) -> Associativity {
        self.precedence().associativity()
    }

    pub fn from_token(token: &TokenKind) -> Option<Self> {
        match token {
            TokenKind::Dot => Some(Self::MemberAccess),
            TokenKind::Star2 => Some(Self::Exponent),
            TokenKind::Star => Some(Self::Multiply),
            TokenKind::Slash => Some(Self::Divide),
            TokenKind::Percent => Some(Self::Remainder),
            TokenKind::Plus => Some(Self::Add),
            TokenKind::Minus => Some(Self::Subtract),
            TokenKind::AngleLeft => Some(Self::LessThan),
            TokenKind::AngleLeftEqual => Some(Self::LessEqual),
            TokenKind::AngleRight => Some(Self::GreaterThan),
            TokenKind::AngleRightEqual => Some(Self::GreaterEqual),
            TokenKind::Equal2 => Some(Self::Equal),
            TokenKind::BangEqual => Some(Self::NotEqual),
            TokenKind::Ampersand2 => Some(Self::LogicalAnd),
            TokenKind::Pipe2 => Some(Self::LogicalOr),
            _ => None
        }
    }

    pub fn fmt_with_operands(&self, f: &mut std::fmt::Formatter, lhs: &ExpressionKind, rhs: &ExpressionKind) -> std::fmt::Result {
        match self {
            Self::MemberAccess => write!(f, "{lhs}.{rhs}"),
            Self::Exponent => write!(f, "{lhs} ** {rhs}"),
            Self::Multiply => write!(f, "{lhs} * {rhs}"),
            Self::Divide => write!(f, "{lhs} / {rhs}"),
            Self::Remainder => write!(f, "{lhs} % {rhs}"),
            Self::Add => write!(f, "{lhs} + {rhs}"),
            Self::Subtract => write!(f, "{lhs} - {rhs}"),
            Self::LessThan => write!(f, "{lhs} < {rhs}"),
            Self::LessEqual => write!(f, "{lhs} <= {rhs}"),
            Self::GreaterThan => write!(f, "{lhs} > {rhs}"),
            Self::GreaterEqual => write!(f, "{lhs} >= {rhs}"),
            Self::Equal => write!(f, "{lhs} == {rhs}"),
            Self::NotEqual => write!(f, "{lhs} != {rhs}"),
            Self::LogicalAnd => write!(f, "{lhs} && {rhs}"),
            Self::LogicalOr => write!(f, "{lhs} || {rhs}"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RangeKind {
    Inclusive,
    Exclusive,
}

impl RangeKind {
    pub fn from_token(token: &TokenKind) -> Option<Self> {
        match token {
            TokenKind::RangeInclusive => Some(Self::Inclusive),
            TokenKind::RangeExclusive => Some(Self::Exclusive),
            _ => None
        }
    }
}

impl std::fmt::Display for RangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Inclusive => write!(f, "..="),
            Self::Exclusive => write!(f, "..<"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ExpressionIndexOperation {
    Single {
        index: Box<Expression>,
    },
    Range {
        kind: RangeKind,
        from_index: Box<Expression>,
        to_index: Box<Expression>,
        step: Option<Box<Expression>>,
    },
    RangeFrom {
        from_index: Box<Expression>,
        step: Option<Box<Expression>>,
    },
    RangeTo {
        kind: RangeKind,
        to_index: Box<Expression>,
    },
}

impl ExpressionIndexOperation {
    pub fn is_range(&self) -> bool {
        matches!(self, Self::Range { .. } | Self::RangeFrom { .. } | Self::RangeTo { .. })
    }
}

impl std::fmt::Display for ExpressionIndexOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Single { index } => {
                write!(f, "{index}")
            }
            Self::Range { kind, from_index, to_index, step } => {
                write!(f, "{from_index} {kind} {to_index}")?;
                if let Some(step) = step {
                    write!(f, " : {step}")?;
                }
                Ok(())
            }
            Self::RangeFrom { from_index, step } => {
                write!(f, "{from_index} ..")?;
                if let Some(step) = step {
                    write!(f, " : {step}")?;
                }
                Ok(())
            }
            Self::RangeTo { kind, to_index } => {
                write!(f, "{kind} {to_index}")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionListMapLoop {
    pub identifier: Rc<str>,
    pub identifier_span: crate::Span,
    pub list: Expression,
}

#[derive(Clone, Debug)]
pub enum ExpressionKind {
    Literal(Literal),
    Intrinsic(Rc<str>),
    Grouping {
        expression: Box<Expression>,
    },
    Unary {
        operation: UnaryOperation,
        operand: Box<Expression>,
    },
    Binary {
        operation: BinaryOperation,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Point2 {
        x: Box<Expression>,
        y: Box<Expression>,
    },
    Point3 {
        x: Box<Expression>,
        y: Box<Expression>,
        z: Box<Expression>,
    },
    List {
        items: Box<[Expression]>,
    },
    ListRange {
        kind: RangeKind,
        start: Box<Expression>,
        end: Box<Expression>,
        step: Option<Box<Expression>>,
    },
    ListFill {
        value: Box<Expression>,
        count: Box<Expression>,
    },
    ListMap {
        loops: Box<[ExpressionListMapLoop]>,
        expression: Box<Expression>,
    },
    ListFilter {
        list: Box<Expression>,
        condition: Box<Expression>,
    },
    Index {
        list: Box<Expression>,
        operation: ExpressionIndexOperation,
    },
    FunctionCall {
        function: Box<Expression>,
        arguments: Box<[Expression]>,
    },
    Conditional {
        condition_consequents: Box<[(Expression, Expression)]>,
        alternative: Option<Box<Expression>>,
    },
    Let {
        identifier: Rc<str>,
        identifier_span: crate::Span,
        value_type: Option<Box<TypeExpression>>,
        value: Box<Expression>,
        expression: Box<Expression>,
    },
}

impl std::fmt::Display for ExpressionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Literal(literal) => {
                write!(f, "{literal}")
            }
            Self::Intrinsic(name) => {
                write!(f, "@{name}")
            }
            Self::Grouping { expression } => {
                write!(f, "({expression})")
            }
            Self::Unary { operation, operand } => {
                operation.fmt_with_operand(f, &operand.kind)
            }
            Self::Binary { operation, lhs, rhs } => {
                operation.fmt_with_operands(f, &lhs.kind, &rhs.kind)
            }
            Self::Point2 { x, y } => {
                write!(f, "({x}, {y})")
            }
            Self::Point3 { x, y, z } => {
                write!(f, "({x}, {y}, {z})")
            }
            Self::List { items } => match items.as_ref() {
                [] => write!(f, "[]"),
                [first, rest @ ..] => {
                    write!(f, "[{first}")?;
                    for item in rest {
                        write!(f, ", {}", item)?;
                    }
                    write!(f, "]")
                }
            }
            Self::ListRange { kind, start, end, step } => {
                write!(f, "[{start} {kind} {end}")?;
                if let Some(step) = step {
                    write!(f, " : {step}")?;
                }
                write!(f, "]")
            }
            Self::ListFill { value, count } => {
                write!(f, "[{value}; {count}]")
            }
            Self::ListMap { loops, expression } => {
                write!(f, "[{expression}")?;
                for ExpressionListMapLoop { identifier, list, .. } in loops {
                    write!(f, " for {identifier} in {list}")?;
                }
                write!(f, "]")
            }
            Self::ListFilter { list, condition } => {
                write!(f, "[{list} where {condition}]")
            }
            Self::Index { list, operation } => {
                write!(f, "{list}[{operation}]")
            }
            Self::FunctionCall { function, arguments } => match arguments.as_ref() {
                [] => write!(f, "{function}()"),
                [first, rest @ ..] => {
                    write!(f, "{function}({first}")?;
                    for argument in rest {
                        write!(f, ", {}", argument)?;
                    }
                    write!(f, ")")
                }
            }
            Self::Conditional { condition_consequents, alternative } => {
                let (condition, consequent) = &condition_consequents[0];
                write!(f, "{{{condition}: {consequent}")?;
                for (condition, consequent) in &condition_consequents[1..] {
                    write!(f, ", {condition}: {consequent}")?;
                }
                if let Some(alternative) = alternative {
                    write!(f, ", {alternative}")?;
                }
                write!(f, "}}")
            }
            Self::Let { identifier, value_type, value, expression, .. } => {
                write!(f, "let {identifier}")?;
                if let Some(value_type) = value_type {
                    write!(f, ": {value_type}")?;
                }
                write!(f, " = {value} in {expression}")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: crate::Span,
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

// TODO: let
#[derive(Clone, Debug)]
pub enum ActionExpressionKind {
    Disable,
    Compound {
        actions: Box<[ActionExpression]>,
    },
    Update {
        variable: Box<Expression>,
        value: Box<Expression>,
    },
    ActionCall {
        identifier: Rc<str>,
        identifier_span: crate::Span,
        arguments: Box<[Expression]>,
    },
    Conditional {
        condition_consequents: Box<[(Expression, ActionExpression)]>,
        alternative: Option<Box<ActionExpression>>,
    },
}

impl std::fmt::Display for ActionExpressionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Disable => {
                write!(f, "disable")
            }
            Self::Compound { actions } => {
                match actions.as_ref() {
                    [] => {
                        write!(f, "{{}}")
                    }
                    [first, rest @ ..] => {
                        write!(f, "{{ {first}")?;
                        for action in rest {
                            write!(f, ", {action}")?;
                        }
                        write!(f, " }}")
                    }
                }
            }
            Self::Update { variable, value } => {
                write!(f, "{variable} := {value}")
            }
            Self::ActionCall { identifier, arguments, .. } => {
                write!(f, "action {identifier}(")?;
                match arguments.as_ref() {
                    [] => {}
                    [first, rest @ ..] => {
                        write!(f, "{first}")?;
                        for argument in rest {
                            write!(f, ", {argument}")?;
                        }
                    }
                }
                write!(f, ")")
            }
            Self::Conditional { condition_consequents, alternative } => {
                let (condition, consequent) = &condition_consequents[0];
                write!(f, "if {condition} then {consequent}")?;
                for (condition, consequent) in &condition_consequents[1..] {
                    write!(f, " elif {condition} then {consequent}")?;
                }
                if let Some(alternative) = alternative {
                    write!(f, " else {alternative}")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActionExpression {
    pub kind: ActionExpressionKind,
    pub span: crate::Span,
}

impl std::fmt::Display for ActionExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub identifier: Rc<str>,
    pub identifier_span: crate::Span,
    pub parameter_type: TypeExpression,
}

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.identifier, self.parameter_type)
    }
}

#[derive(Clone, Debug)]
pub struct ParameterList(pub Box<[Parameter]>);

impl std::fmt::Display for ParameterList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0.as_ref() {
            [] => {
                write!(f, "()")
            }
            [first, rest @ ..] => {
                write!(f, "({first}")?;
                for parameter in rest {
                    write!(f, ", {parameter}")?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum VariableKind {
    Default,
    Timer,
}

impl std::fmt::Display for VariableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "var"),
            Self::Timer => write!(f, "var timer"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ValueDefinition {
    Let {
        parameters: Option<ParameterList>,
        value_type: Box<TypeExpression>,
        value: Box<Expression>,
    },
    Variable {
        kind: VariableKind,
        value_type: Box<TypeExpression>,
        value: Box<Expression>,
    },
    Action {
        parameters: ParameterList,
        action: Box<ActionExpression>,
    },
}

// TODO: allow assigning values
#[derive(Clone, Debug)]
pub struct EnumerationVariant {
    pub identifier: Rc<str>,
    pub identifier_span: crate::Span,
}

impl std::fmt::Display for EnumerationVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.identifier)
    }
}

#[derive(Clone, Debug)]
pub enum TypeDefinition {
    Enumeration {
        variants: Box<[EnumerationVariant]>,
    },
}

#[derive(Clone, Debug)]
pub enum DefinitionKind {
    Type(TypeDefinition),
    Value(ValueDefinition),
}

#[derive(Clone, Debug)]
pub struct Definition {
    pub identifier: Rc<str>,
    pub kind: DefinitionKind,
    pub span: crate::Span,
}

impl std::fmt::Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.kind {
            DefinitionKind::Type(TypeDefinition::Enumeration { variants }) => {
                write!(f, "enum {} ", self.identifier)?;
                match variants.as_ref() {
                    [] => {
                        write!(f, "{{}}")
                    }
                    [first, rest @ ..] => {
                        write!(f, "{{ {first}")?;
                        for variant in rest {
                            write!(f, ", {variant}")?;
                        }
                        write!(f, " }}")
                    }
                }
            }
            DefinitionKind::Value(ValueDefinition::Let { parameters, value_type, value }) => {
                write!(f, "let {}", self.identifier)?;
                if let Some(parameters) = parameters {
                    write!(f, "{parameters}")?;
                }
                write!(f, ": {value_type} = {value};")
            }
            DefinitionKind::Value(ValueDefinition::Variable { kind, value_type, value }) => {
                write!(f, "{kind} {}: {value_type} = {value};", self.identifier)
            }
            DefinitionKind::Value(ValueDefinition::Action { parameters, action }) => {
                write!(f, "action {}{parameters} {action}", self.identifier)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TickerDeclaration {
    pub interval_ms: Option<Box<Expression>>,
    pub tick_action: Box<ActionExpression>,
    pub span: crate::Span,
}

impl std::fmt::Display for TickerDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ticker")?;
        if let Some(interval) = &self.interval_ms {
            write!(f, " ({interval})")?;
        }
        write!(f, " {}", self.tick_action)
    }
}

#[derive(Clone, Debug)]
pub enum PublicLineKind {
    Text(Rc<str>),
    Expression(Expression),
    Action(ActionExpression),
}

impl std::fmt::Display for PublicLineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Text(text) => write!(f, "{text:?}"),
            Self::Expression(expression) => write!(f, "{expression}"),
            Self::Action(action) => write!(f, "{action}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PublicLine {
    pub kind: PublicLineKind,
    pub span: crate::Span,
}

impl std::fmt::Display for PublicLine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct PublicDeclaration {
    pub lines: Box<[PublicLine]>,
    pub span: crate::Span,
}

impl std::fmt::Display for PublicDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.lines.as_ref() {
            [] => {
                write!(f, "public {{}}")
            }
            [first, rest @ ..] => {
                write!(f, "public {{ {first}")?;
                for line in rest {
                    write!(f, "; {line}")?;
                }
                write!(f, " }}")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum DisplayAttributeValue {
    Arguments(Box<[Expression]>),
    Action(ActionExpression),
}

impl std::fmt::Display for DisplayAttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Arguments(arguments) => match arguments.as_ref() {
                [] => {
                    write!(f, "()")
                }
                [first, rest @ ..] => {
                    write!(f, "({first}")?;
                    for argument in rest {
                        write!(f, ", {argument}")?;
                    }
                    write!(f, ")")
                }
            }
            Self::Action(action) => {
                write!(f, " {action}")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DisplayAttribute {
    pub key: Rc<str>,
    pub key_span: crate::Span,
    pub value: DisplayAttributeValue,
}

impl std::fmt::Display for DisplayAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.key, self.value)
    }
}

#[derive(Clone, Debug)]
pub struct DisplayElement {
    pub expression: Expression,
    pub attributes: Box<[DisplayAttribute]>,
    pub span: crate::Span,
}

impl std::fmt::Display for DisplayElement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.expression)?;
        if let [first, rest @ ..] = self.attributes.as_ref() {
            write!(f, ": {first}")?;
            for attribute in rest {
                write!(f, ", {attribute}")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct DisplayDeclaration {
    pub elements: Box<[DisplayElement]>,
    pub span: crate::Span,
}

impl std::fmt::Display for DisplayDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.elements.as_ref() {
            [] => {
                write!(f, "display {{}}")
            }
            [first, rest @ ..] => {
                write!(f, "display {{ {first}")?;
                for element in rest {
                    write!(f, "; {element}")?;
                }
                write!(f, " }}")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Declaration {
    Definition(Definition),
    Ticker(TickerDeclaration),
    Public(PublicDeclaration),
    Display(DisplayDeclaration),
}

impl std::fmt::Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Definition(definition) => definition.fmt(f),
            Self::Ticker(ticker) => ticker.fmt(f),
            Self::Public(public) => public.fmt(f),
            Self::Display(display) => display.fmt(f),
        }
    }
}
