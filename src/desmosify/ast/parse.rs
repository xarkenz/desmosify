use super::*;

use std::io::BufRead;
use std::rc::Rc;
use crate::token::scan::Scanner;
use crate::token::Token;

#[derive(Debug)]
pub struct Parser<'a, T: BufRead> {
    scanner: &'a mut Scanner<T>,
    current_token: Option<Token>,
}

impl<'a, T: BufRead> Parser<'a, T> {
    pub fn new(scanner: &'a mut Scanner<T>) -> crate::Result<Self> {
        let mut parser = Self {
            scanner,
            current_token: None,
        };
        parser.consume_token()?;
        Ok(parser)
    }

    pub fn source_id(&self) -> usize {
        self.scanner.source_id()
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    pub fn current_token_kind(&self) -> Option<&TokenKind> {
        self.current_token.as_ref().map(|token| &token.kind)
    }

    pub fn current_span(&self) -> crate::Span {
        if let Some(token) = &self.current_token {
            token.span
        }
        else {
            self.scanner.create_span(self.scanner.next_index(), self.scanner.next_index())
        }
    }

    pub fn consume_token(&mut self) -> crate::Result<()> {
        self.current_token = self.scanner.next_token()?;
        Ok(())
    }

    pub fn get_token(&self) -> crate::Result<&Token> {
        self.current_token().ok_or_else(|| Box::new(crate::Error {
            kind: crate::ErrorKind::ExpectedToken,
            span: Some(self.current_span()),
        }))
    }

    pub fn expect_token_from(&self, allowed_kinds: &[TokenKind]) -> crate::Result<&Token> {
        let current_token = self.get_token()?;
        allowed_kinds.iter()
            .find_map(|kind| (kind == &current_token.kind).then_some(current_token))
            .ok_or_else(|| Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedTokenFromList {
                    got_token: current_token.kind.clone(),
                    allowed_tokens: allowed_kinds.to_vec(),
                },
                span: Some(current_token.span),
            }))
    }

    pub fn expect_identifier(&self) -> crate::Result<(Rc<str>, crate::Span)> {
        let token = self.get_token()?;
        match &token.kind {
            TokenKind::Identifier(identifier) => Ok((identifier.clone(), token.span)),
            _ => Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedIdentifier,
                span: Some(token.span),
            }))
        }
    }

    pub fn expect_identifier_or_keyword(&self) -> crate::Result<(Rc<str>, crate::Span)> {
        let token = self.get_token()?;
        match &token.kind {
            TokenKind::Identifier(identifier) => Ok((identifier.clone(), token.span)),
            _ => match token.kind.get_keyword_literal() {
                Some(literal) => Ok((literal.into(), token.span)),
                None => Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::ExpectedIdentifier,
                    span: Some(token.span),
                }))
            }
        }
    }

    pub fn expect_string(&self) -> crate::Result<(Rc<str>, crate::Span)> {
        let token = self.get_token()?;
        match &token.kind {
            TokenKind::String(string) => Ok((string.clone(), token.span)),
            _ => Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedString,
                span: Some(token.span),
            }))
        }
    }

    pub fn parse_type(&mut self, allowed_ends: &[TokenKind]) -> crate::Result<TypeExpression> {
        let start_token = self.get_token()?;
        let start_span = start_token.span;

        let mut expression_kind = match &start_token.kind {
            TokenKind::Identifier(identifier) => {
                TypeExpressionKind::Identifier(identifier.clone())
            }
            TokenKind::ParenLeft => {
                self.consume_token()?; // ParenLeft
                let first_item = self.parse_type(&[TokenKind::Comma, TokenKind::ParenRight])?;

                if let Some(TokenKind::ParenRight) = self.current_token_kind() {
                    // Grouping parentheses
                    TypeExpressionKind::Grouping {
                        expression: Box::new(first_item),
                    }
                }
                else {
                    // Point type
                    self.consume_token()?; // Comma
                    let second_item = self.parse_type(&[TokenKind::Comma, TokenKind::ParenRight])?;

                    // Allow trailing comma
                    if let Some(TokenKind::Comma) = self.current_token_kind() {
                        self.consume_token()?; // Comma
                    }

                    if let Some(TokenKind::ParenRight) = self.current_token_kind() {
                        // Point type with two components
                        TypeExpressionKind::Point2 {
                            x_type: Box::new(first_item),
                            y_type: Box::new(second_item),
                        }
                    }
                    else {
                        // Point type with three components
                        let third_item = self.parse_type(&[TokenKind::Comma, TokenKind::ParenRight])?;

                        // Allow trailing comma
                        if let Some(TokenKind::Comma) = self.current_token_kind() {
                            self.consume_token()?; // Comma
                            self.expect_token_from(&[TokenKind::ParenRight])?;
                        }

                        TypeExpressionKind::Point3 {
                            x_type: Box::new(first_item),
                            y_type: Box::new(second_item),
                            z_type: Box::new(third_item),
                        }
                    }
                }
            }
            TokenKind::SquareLeft => {
                self.consume_token()?; // SquareLeft
                let item_type = self.parse_type(&[TokenKind::SquareRight])?;

                TypeExpressionKind::List {
                    item_type: Box::new(item_type),
                }
            }
            got_token => {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::ExpectedType {
                        got_token: got_token.clone(),
                    },
                    span: Some(self.current_span()),
                }))
            }
        };

        let mut span = start_span.expand_to(self.current_span());
        self.consume_token()?; // Whatever the last token of the expression was

        while let Some(TokenKind::Plus) = self.current_token_kind() {
            expression_kind = TypeExpressionKind::Broadcastable {
                item_type: Box::new(TypeExpression {
                    span,
                    kind: expression_kind,
                })
            };

            span = start_span.expand_to(self.current_span());
            self.consume_token()?; // Plus
        }

        // Check that we have arrived at one of the allowed ending tokens.
        self.expect_token_from(allowed_ends)?;

        Ok(TypeExpression {
            span,
            kind: expression_kind,
        })
    }

    pub fn parse_operand(&mut self, allowed_ends: &[TokenKind]) -> crate::Result<Expression> {
        let start_token = self.get_token()?;
        let start_span = start_token.span;

        if let Some(operation) = UnaryOperation::from_prefix_token(&start_token.kind) {
            self.consume_token()?; // Whatever the prefix operator token was
            let operand = self.parse_expression(Some(Precedence::Prefix), allowed_ends)?;

            Ok(Expression {
                span: start_span.expand_to(operand.span),
                kind: ExpressionKind::Unary {
                    operation,
                    operand: Box::new(operand),
                },
            })
        }
        else {
            let operand_kind = match &start_token.kind {
                TokenKind::Undefined => {
                    ExpressionKind::Undefined
                }
                TokenKind::Infinity => {
                    ExpressionKind::Infinity
                }
                TokenKind::Integer(value) => {
                    ExpressionKind::Integer(*value)
                }
                TokenKind::Real(value) => {
                    ExpressionKind::Real(*value)
                }
                TokenKind::Boolean(value) => {
                    ExpressionKind::Boolean(*value)
                }
                TokenKind::Character(value) => {
                    ExpressionKind::Character(*value)
                }
                TokenKind::String(value) => {
                    ExpressionKind::String(value.clone())
                }
                TokenKind::Identifier(identifier) => {
                    ExpressionKind::Identifier(identifier.clone())
                }
                TokenKind::Action => {
                    self.consume_token()?; // Action
                    let (identifier, _) = self.expect_identifier_or_keyword()?;

                    ExpressionKind::ActionIdentifier(identifier)
                }
                TokenKind::AtSign => {
                    self.consume_token()?; // AtSign
                    let (identifier, _) = self.expect_identifier_or_keyword()?;

                    ExpressionKind::IntrinsicIdentifier(identifier)
                }
                TokenKind::ParenLeft => {
                    self.consume_token()?; // ParenLeft
                    let first_item = self.parse_expression(None, &[TokenKind::Comma, TokenKind::ParenRight])?;

                    if let Some(TokenKind::ParenRight) = self.current_token_kind() {
                        // Grouping parentheses
                        ExpressionKind::Grouping {
                            expression: Box::new(first_item),
                        }
                    }
                    else {
                        // Point literal
                        self.consume_token()?; // Comma
                        let second_item = self.parse_expression(None, &[TokenKind::Comma, TokenKind::ParenRight])?;

                        // Allow trailing comma
                        if let Some(TokenKind::Comma) = self.current_token_kind() {
                            self.consume_token()?; // Comma
                        }

                        if let Some(TokenKind::ParenRight) = self.current_token_kind() {
                            // Point literal with two components
                            ExpressionKind::Point2 {
                                x: Box::new(first_item),
                                y: Box::new(second_item),
                            }
                        }
                        else {
                            // Point literal with three components
                            let third_item = self.parse_expression(None, &[TokenKind::Comma, TokenKind::ParenRight])?;

                            // Allow trailing comma
                            if let Some(TokenKind::Comma) = self.current_token_kind() {
                                self.consume_token()?; // Comma
                                self.expect_token_from(&[TokenKind::ParenRight])?;
                            }

                            ExpressionKind::Point3 {
                                x: Box::new(first_item),
                                y: Box::new(second_item),
                                z: Box::new(third_item),
                            }
                        }
                    }
                }
                TokenKind::SquareLeft => {
                    // List literal
                    self.consume_token()?; // SquareLeft
                    if let Some(TokenKind::SquareRight) = self.current_token_kind() {
                        // Empty list literal
                        ExpressionKind::List {
                            items: Box::new([]),
                        }
                    }
                    else {
                        let first_item = self.parse_expression(None, &[
                            TokenKind::Comma,
                            TokenKind::SquareRight,
                            TokenKind::RangeInclusive,
                            TokenKind::RangeExclusive,
                            TokenKind::Semicolon,
                            TokenKind::For,
                            TokenKind::Where,
                        ])?;

                        match self.current_token_kind() {
                            Some(kind @ (TokenKind::Comma | TokenKind::SquareRight)) => {
                                // Plain list literal
                                if let TokenKind::Comma = kind {
                                    self.consume_token()?; // Comma
                                }

                                let mut items = vec![first_item];
                                while !matches!(self.current_token_kind(), Some(TokenKind::SquareRight)) {
                                    let item = self.parse_expression(None, &[TokenKind::Comma, TokenKind::SquareRight])?;
                                    items.push(item);

                                    if let Some(TokenKind::Comma) = self.current_token_kind() {
                                        self.consume_token()?; // Comma
                                    }
                                }

                                ExpressionKind::List {
                                    items: items.into_boxed_slice(),
                                }
                            }
                            Some(kind @ (TokenKind::RangeInclusive | TokenKind::RangeExclusive)) => {
                                // List range
                                let range_kind = RangeKind::from_token(kind).unwrap();
                                self.consume_token()?; // Whatever the range operator token was

                                let range_end = self.parse_expression(None, &[TokenKind::Colon, TokenKind::SquareRight])?;
                                let mut range_step = None;
                                if let Some(TokenKind::Colon) = self.current_token_kind() {
                                    self.consume_token()?; // Colon
                                    range_step = Some(self.parse_expression(None, &[TokenKind::SquareRight])?);
                                }

                                ExpressionKind::ListRange {
                                    kind: range_kind,
                                    start: Box::new(first_item),
                                    end: Box::new(range_end),
                                    step: range_step.map(Box::new),
                                }
                            }
                            Some(TokenKind::Semicolon) => {
                                // List fill
                                self.consume_token()?; // Semicolon
                                let fill_count = self.parse_expression(None, &[TokenKind::SquareRight])?;

                                ExpressionKind::ListFill {
                                    value: Box::new(first_item),
                                    count: Box::new(fill_count),
                                }
                            }
                            Some(TokenKind::For) => {
                                // List map
                                let mut loops = Vec::new();

                                while !matches!(self.current_token_kind(), Some(TokenKind::SquareRight)) {
                                    self.consume_token()?; // For
                                    let (identifier, identifier_span) = self.expect_identifier()?;
                                    self.consume_token()?; // Identifier
                                    self.expect_token_from(&[TokenKind::In])?;
                                    self.consume_token()?; // In
                                    let list = self.parse_expression(None, &[TokenKind::For, TokenKind::SquareRight])?;

                                    loops.push(ExpressionListMapLoop {
                                        identifier,
                                        identifier_span,
                                        list,
                                    });
                                }

                                ExpressionKind::ListMap {
                                    loops: loops.into_boxed_slice(),
                                    expression: Box::new(first_item),
                                }
                            }
                            Some(TokenKind::Where) => {
                                // List filter
                                self.consume_token()?; // Where
                                let condition = self.parse_expression(None, &[TokenKind::SquareRight])?;

                                ExpressionKind::ListFilter {
                                    list: Box::new(first_item),
                                    condition: Box::new(condition),
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                }
                TokenKind::CurlyLeft => {
                    // Conditional (piecewise)
                    self.consume_token()?; // CurlyLeft

                    let mut condition_consequents = Vec::new();
                    let mut alternative = None;

                    while !matches!(self.current_token_kind(), Some(TokenKind::CurlyRight)) {
                        let expression = self.parse_expression(None, &[TokenKind::Colon, TokenKind::CurlyRight])?;

                        if let Some(TokenKind::CurlyRight) = self.current_token_kind() {
                            alternative = Some(expression);
                            break;
                        }

                        self.consume_token()?; // Colon
                        let consequent = self.parse_expression(None, &[TokenKind::Comma, TokenKind::CurlyRight])?;
                        condition_consequents.push((expression, consequent));
                        if let Some(TokenKind::Comma) = self.current_token_kind() {
                            self.consume_token()?; // Comma
                        }
                    }

                    if condition_consequents.is_empty() {
                        return Err(Box::new(crate::Error {
                            kind: crate::ErrorKind::ConditionalMissingCondition,
                            span: Some(start_span.expand_to(self.current_span())),
                        }))
                    }

                    ExpressionKind::Conditional {
                        condition_consequents: condition_consequents.into_boxed_slice(),
                        alternative: alternative.map(Box::new),
                    }
                }
                TokenKind::Let => {
                    // Let expression
                    self.consume_token()?; // Let
                    let (identifier, identifier_span) = self.expect_identifier()?;
                    self.consume_token()?; // Identifier
                    let token = self.expect_token_from(&[TokenKind::Colon, TokenKind::Equal])?;

                    let mut value_type = None;
                    if let TokenKind::Colon = token.kind {
                        self.consume_token()?; // Colon
                        value_type = Some(self.parse_type(&[TokenKind::Equal])?);
                    }

                    self.consume_token()?; // Equal
                    let value = self.parse_expression(None, &[TokenKind::In])?;
                    self.consume_token()?; // In
                    let inner = self.parse_expression(None, allowed_ends)?;

                    // This operand took over the rest of the expression, so return here
                    return Ok(Expression {
                        span: start_span.expand_to(inner.span),
                        kind: ExpressionKind::Let {
                            identifier,
                            identifier_span,
                            value_type: value_type.map(Box::new),
                            value: Box::new(value),
                            inner: Box::new(inner),
                        },
                    })
                }
                got_token => {
                    return Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::ExpectedOperand {
                            got_token: got_token.clone(),
                        },
                        span: Some(self.current_span()),
                    }))
                }
            };

            let span = start_span.expand_to(self.current_span());
            self.consume_token()?; // Whatever the last token of the operand was

            let mut operand = Expression {
                span,
                kind: operand_kind,
            };

            // Greedily parse postfix operators, if any
            while let Some(operation) = UnaryOperation::from_postfix_token(&self.get_token()?.kind) {
                operand = Expression {
                    span: start_span.expand_to(self.current_span()),
                    kind: ExpressionKind::Unary {
                        operation,
                        operand: Box::new(operand),
                    },
                };
                self.consume_token()?; // Whatever the postfix operator token was
            }

            Ok(operand)
        }
    }

    pub fn parse_argument_list(&mut self) -> crate::Result<Box<[Expression]>> {
        let mut arguments = Vec::new();

        while !matches!(self.current_token_kind(), Some(TokenKind::ParenRight)) {
            let argument = self.parse_expression(None, &[TokenKind::Comma, TokenKind::ParenRight])?;
            arguments.push(argument);

            if let Some(TokenKind::Comma) = self.current_token_kind() {
                self.consume_token()?; // Comma
            }
        }

        Ok(arguments.into_boxed_slice())
    }

    pub fn parse_expression(&mut self, parent_precedence: Option<Precedence>, allowed_ends: &[TokenKind]) -> crate::Result<Expression> {
        let start_span = self.current_span();
        let mut lhs = self.parse_operand(allowed_ends)?;

        while let Some(operator_token) = self.current_token() {
            if allowed_ends.contains(&operator_token.kind) {
                // Allowed ends are checked before operations, even if one of the allowed ends
                // doubles as a valid operator.
                break;
            }
            else if let Some(operation) = BinaryOperation::from_token(&operator_token.kind) {
                let precedence = operation.precedence();

                if parent_precedence.is_some_and(|parent_precedence| parent_precedence > precedence || (
                    parent_precedence == precedence && precedence.associativity() == Associativity::LeftToRight
                )) {
                    // Parent operation has a higher precedence and therefore must be made into a
                    // subtree of the next operation. If the parent operation has the same
                    // precedence, only make it a subtree when there is left-to-right associativity.
                    break;
                }

                self.consume_token()?; // Whatever the operator token was
                let rhs = self.parse_expression(Some(precedence), allowed_ends)?;

                lhs = Expression {
                    span: start_span.expand_to(rhs.span),
                    kind: ExpressionKind::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                };
            }
            else if let TokenKind::SquareLeft = operator_token.kind {
                // Parse an indexing operation
                if parent_precedence.is_some_and(|parent_precedence| parent_precedence >= Precedence::Postfix) {
                    // Parent operation should be made into a subtree of the indexing operation
                    // (which has postfix precedence). Usually the parent operation is some
                    // sort of access operation, e.g. `a.b()`.
                    break;
                }

                self.consume_token()?; // SquareLeft
                let index_operation;

                if let Some(range_kind) = RangeKind::from_token(&self.get_token()?.kind) {
                    self.consume_token()?; // Whatever the range operator token was
                    let to_index = self.parse_expression(None, &[TokenKind::SquareRight])?;

                    index_operation = IndexOperation::RangeTo {
                        kind: range_kind,
                        to_index: Box::new(to_index),
                    };
                }
                else {
                    let index = self.parse_expression(None, &[
                        TokenKind::RangeInclusive,
                        TokenKind::RangeExclusive,
                        TokenKind::Dot2,
                        TokenKind::SquareRight,
                    ])?;

                    if let Some(range_kind) = RangeKind::from_token(&self.get_token()?.kind) {
                        self.consume_token()?; // Whatever the range operator token was
                        let to_index = self.parse_expression(None, &[TokenKind::Colon, TokenKind::SquareRight])?;

                        let step = match self.current_token_kind() {
                            Some(TokenKind::Colon) => {
                                self.consume_token()?; // Colon
                                let step = self.parse_expression(None, &[TokenKind::SquareRight])?;

                                Some(Box::new(step))
                            }
                            Some(TokenKind::SquareRight) => {
                                None
                            }
                            _ => unreachable!()
                        };

                        index_operation = IndexOperation::Range {
                            kind: range_kind,
                            from_index: Box::new(index),
                            to_index: Box::new(to_index),
                            step,
                        };
                    }
                    else if let Some(TokenKind::Dot2) = self.current_token_kind() {
                        self.consume_token()?; // Dot2
                        let token = self.expect_token_from(&[TokenKind::Colon, TokenKind::SquareRight])?;

                        let step = match token.kind {
                            TokenKind::Colon => {
                                self.consume_token()?; // Colon
                                let step = self.parse_expression(None, &[TokenKind::SquareRight])?;

                                Some(Box::new(step))
                            }
                            TokenKind::SquareRight => {
                                None
                            }
                            _ => unreachable!()
                        };

                        index_operation = IndexOperation::RangeFrom {
                            from_index: Box::new(index),
                            step,
                        };
                    }
                    else {
                        index_operation = IndexOperation::Single {
                            index: Box::new(index),
                        };
                    }
                }

                let span = start_span.expand_to(self.current_span());
                self.consume_token()?; // SquareRight

                lhs = Expression {
                    span,
                    kind: ExpressionKind::Index {
                        list: Box::new(lhs),
                        operation: index_operation,
                    },
                };
            }
            else if let TokenKind::ParenLeft = operator_token.kind {
                // Parse a function call
                if parent_precedence.is_some_and(|parent_precedence| parent_precedence >= Precedence::Postfix) {
                    // Parent operation should be made into a subtree of the call operation
                    // (which has postfix precedence). Usually the parent operation is some
                    // sort of access operation, e.g. `a.b()`.
                    break;
                }

                self.consume_token()?; // ParenLeft
                let arguments = self.parse_argument_list()?;
                let span = start_span.expand_to(self.current_span());
                self.consume_token()?; // ParenRight

                lhs = Expression {
                    span,
                    kind: ExpressionKind::FunctionCall {
                        function: Box::new(lhs),
                        arguments,
                    },
                };
            }
            else {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::ExpectedOperation {
                        got_token: operator_token.kind.clone(),
                    },
                    span: Some(self.current_span()),
                }))
            }
        }

        if parent_precedence.is_none() {
            // Check that we have arrived at one of the allowed ending tokens.
            self.expect_token_from(allowed_ends)?;
        }

        Ok(lhs)
    }

    pub fn parse_action(&mut self, allowed_ends: &[TokenKind]) -> crate::Result<ActionExpression> {
        let start_token = self.get_token()?;
        let start_span = start_token.span;

        let action_kind = match &start_token.kind {
            TokenKind::Disable => {
                ActionExpressionKind::Disable
            }
            TokenKind::CurlyLeft => {
                self.consume_token()?; // CurlyLeft
                let mut actions = Vec::new();

                while !matches!(self.current_token_kind(), Some(TokenKind::CurlyRight)) {
                    let action = self.parse_action(&[TokenKind::Comma, TokenKind::CurlyRight])?;
                    actions.push(action);

                    if let Some(TokenKind::Comma) = self.current_token_kind() {
                        self.consume_token()?; // Comma
                    }
                }

                ActionExpressionKind::Compound {
                    actions: actions.into_boxed_slice(),
                }
            }
            TokenKind::Action => {
                self.consume_token()?; // Action
                let (identifier, identifier_span) = self.expect_identifier()?;
                self.consume_token()?; // Identifier
                self.expect_token_from(&[TokenKind::ParenLeft])?;
                self.consume_token()?; // ParenLeft
                let arguments = self.parse_argument_list()?;

                ActionExpressionKind::ActionCall {
                    action: Box::new(Expression {
                        kind: ExpressionKind::ActionIdentifier(identifier),
                        span: start_span.expand_to(identifier_span),
                    }),
                    arguments,
                }
            }
            TokenKind::If => {
                let allowed_consequent_ends: Vec<_> = allowed_ends
                    .iter()
                    .cloned()
                    .chain([TokenKind::Elif, TokenKind::Else])
                    .collect();

                self.consume_token()?; // If
                let condition = self.parse_expression(None, &[TokenKind::Then])?;
                self.consume_token()?; // Then
                let consequent = self.parse_action(&allowed_consequent_ends)?;

                let mut end_span = consequent.span;
                let mut condition_consequents = vec![(condition, consequent)];
                let mut alternative = None;
                loop {
                    match self.current_token_kind() {
                        Some(TokenKind::Elif) => {
                            self.consume_token()?; // Elif
                            let condition = self.parse_expression(None, &[TokenKind::Then])?;
                            self.consume_token()?; // Then
                            let consequent = self.parse_action(&allowed_consequent_ends)?;

                            end_span = consequent.span;
                            condition_consequents.push((condition, consequent));
                        }
                        Some(TokenKind::Else) => {
                            self.consume_token()?; // Else
                            let action = self.parse_action(allowed_ends)?;

                            end_span = action.span;
                            alternative = Some(Box::new(action));
                            break;
                        }
                        _ => {
                            break;
                        }
                    }
                }

                // The last consequent or alternative consumed the last token, so return here
                return Ok(ActionExpression {
                    span: start_span.expand_to(end_span),
                    kind: ActionExpressionKind::Conditional {
                        condition_consequents: condition_consequents.into_boxed_slice(),
                        alternative,
                    },
                })
            }
            keyword @ (TokenKind::Elif | TokenKind::Else) => {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UnexpectedConditionalKeyword {
                        keyword: keyword.clone(),
                    },
                    span: Some(start_span),
                }))
            }
            _ => {
                let variable = self.parse_expression(None, &[TokenKind::ColonEqual])?;
                self.consume_token()?; // ColonEqual
                let value = self.parse_expression(None, allowed_ends)?;

                // The value expression took over the rest of the action, so return here
                return Ok(ActionExpression {
                    span: start_span.expand_to(value.span),
                    kind: ActionExpressionKind::Update {
                        variable: Box::new(variable),
                        value: Box::new(value),
                    },
                })
            }
        };

        let span = start_span.expand_to(self.current_span());
        self.consume_token()?; // Whatever the last token of the action was

        Ok(ActionExpression {
            kind: action_kind,
            span,
        })
    }

    pub fn parse_parameter_list(&mut self) -> crate::Result<ParameterList> {
        let mut parameters = Vec::new();

        while !matches!(self.current_token_kind(), Some(TokenKind::ParenRight)) {
            let (identifier, identifier_span) = self.expect_identifier()?;
            self.consume_token()?; // Identifier
            self.expect_token_from(&[TokenKind::Colon])?;
            self.consume_token()?; // Colon
            let parameter_type = self.parse_type(&[TokenKind::Comma, TokenKind::ParenRight])?;
            parameters.push(Parameter {
                identifier,
                identifier_span,
                parameter_type,
            });

            if let Some(TokenKind::Comma) = self.current_token_kind() {
                self.consume_token()?; // Comma
            }
        }

        Ok(ParameterList(parameters.into_boxed_slice()))
    }

    fn parse_slider_range(&mut self) -> crate::Result<VariableKind> {
        let (min, max) = match self.current_token_kind() {
            Some(TokenKind::Dot2) => {
                // No min or max specified
                self.consume_token()?; // Dot2
                self.expect_token_from(&[TokenKind::Colon, TokenKind::ParenRight])?;
                (None, None)
            }
            Some(TokenKind::RangeInclusive) => {
                // Only max specified
                self.consume_token()?; // RangeInclusive
                let max = self.parse_expression(None, &[TokenKind::Colon, TokenKind::ParenRight])?;
                (None, Some(Box::new(max)))
            }
            _ => {
                // Min and possibly max specified
                let min = self.parse_expression(None, &[TokenKind::Dot2, TokenKind::RangeInclusive])?;
                if let Some(TokenKind::Dot2) = self.current_token_kind() {
                    // No max specified
                    self.consume_token()?; // Dot2
                    self.expect_token_from(&[TokenKind::Colon, TokenKind::ParenRight])?;
                    (Some(Box::new(min)), None)
                }
                else {
                    // Both min and max specified
                    self.consume_token()?; // RangeInclusive
                    let max = self.parse_expression(None, &[TokenKind::Colon, TokenKind::ParenRight])?;
                    (Some(Box::new(min)), Some(Box::new(max)))
                }
            }
        };

        let mut step = None;
        if let Some(TokenKind::Colon) = self.current_token_kind() {
            self.consume_token()?; // Colon
            step = Some(Box::new(self.parse_expression(None, &[TokenKind::ParenRight])?));
        }

        Ok(VariableKind::Slider {
            min,
            max,
            step,
        })
    }

    pub fn parse_declaration(&mut self) -> crate::Result<Option<Declaration>> {
        if self.current_token().is_none() {
            return Ok(None)
        }

        let start_token = self.expect_token_from(&[
            TokenKind::Let,
            TokenKind::Var,
            TokenKind::Action,
            TokenKind::Enum,
            TokenKind::Ticker,
            TokenKind::Public,
            TokenKind::Display,
        ])?;
        let start_span = start_token.span;

        match &start_token.kind {
            TokenKind::Let => {
                self.consume_token()?; // Let

                let (identifier, identifier_span) = self.expect_identifier()?;
                self.consume_token()?; // Identifier

                let mut parameters = None;
                if let Some(TokenKind::ParenLeft) = self.current_token_kind() {
                    self.consume_token()?; // ParenLeft
                    parameters = Some(self.parse_parameter_list()?);
                    self.consume_token()?; // ParenRight
                }

                self.expect_token_from(&[TokenKind::Colon])?;
                self.consume_token()?; // Colon
                let value_type = self.parse_type(&[TokenKind::Equal])?;
                self.consume_token()?; // Equal
                let value = self.parse_expression(None, &[TokenKind::Semicolon])?;
                self.consume_token()?; // Semicolon

                Ok(Some(Declaration::Definition(Definition {
                    identifier,
                    kind: DefinitionKind::Value(ValueDefinition::Let {
                        parameters,
                        value_type: Box::new(value_type),
                        value: Box::new(value),
                    }),
                    span: start_span.expand_to(identifier_span),
                })))
            }
            TokenKind::Var => {
                self.consume_token()?; // Var
                let variable_kind = match self.current_token_kind() {
                    Some(TokenKind::Timer) => {
                        self.consume_token()?; // Timer
                        VariableKind::Timer
                    }
                    Some(TokenKind::Slider) => {
                        self.consume_token()?; // Slider
                        self.expect_token_from(&[TokenKind::ParenLeft])?;
                        self.consume_token()?; // ParenLeft
                        let slider = self.parse_slider_range()?;
                        self.consume_token()?; // ParenRight
                        slider
                    }
                    _ => {
                        VariableKind::Default
                    }
                };

                let (identifier, identifier_span) = self.expect_identifier()?;
                self.consume_token()?; // Identifier

                self.expect_token_from(&[TokenKind::Colon])?;
                self.consume_token()?; // Colon
                let value_type = self.parse_type(&[TokenKind::Equal])?;
                self.consume_token()?; // Equal
                let value = self.parse_expression(None, &[TokenKind::Semicolon])?;
                self.consume_token()?; // Semicolon

                Ok(Some(Declaration::Definition(Definition {
                    identifier,
                    kind: DefinitionKind::Value(ValueDefinition::Variable {
                        kind: variable_kind,
                        value_type: Box::new(value_type),
                        value: Box::new(value),
                    }),
                    span: start_span.expand_to(identifier_span),
                })))
            }
            TokenKind::Action => {
                self.consume_token()?; // Action

                let (identifier, identifier_span) = self.expect_identifier()?;
                self.consume_token()?; // Identifier

                self.expect_token_from(&[TokenKind::ParenLeft])?;
                self.consume_token()?; // ParenLeft
                let parameters = self.parse_parameter_list()?;
                self.consume_token()?; // ParenRight

                self.expect_token_from(&[TokenKind::CurlyLeft])?;
                let action = self.parse_action(&[])?;

                Ok(Some(Declaration::Definition(Definition {
                    identifier,
                    kind: DefinitionKind::Value(ValueDefinition::Action {
                        parameters,
                        action: Box::new(action),
                    }),
                    span: start_span.expand_to(identifier_span),
                })))
            }
            TokenKind::Enum => {
                self.consume_token()?; // Enum

                let (identifier, identifier_span) = self.expect_identifier()?;
                self.consume_token()?; // Identifier

                self.expect_token_from(&[TokenKind::CurlyLeft])?;
                self.consume_token()?; // CurlyLeft
                let mut variants = Vec::new();

                while !matches!(self.current_token_kind(), Some(TokenKind::CurlyRight)) {
                    let (identifier, identifier_span) = self.expect_identifier()?;
                    self.consume_token()?; // Identifier
                    let token = self.expect_token_from(&[
                        TokenKind::Equal,
                        TokenKind::Comma,
                        TokenKind::CurlyRight,
                    ])?;

                    let mut ordinal = None;
                    if let TokenKind::Equal = token.kind {
                        self.consume_token()?; // Equal
                        ordinal = Some(self.parse_expression(None, &[
                            TokenKind::Comma,
                            TokenKind::CurlyRight,
                        ])?);
                    }

                    variants.push(EnumerationVariant {
                        identifier,
                        identifier_span,
                        value: ordinal,
                    });

                    if let Some(TokenKind::Comma) = self.current_token_kind() {
                        self.consume_token()?; // Comma
                    }
                }
                self.consume_token()?; // CurlyRight

                Ok(Some(Declaration::Definition(Definition {
                    identifier,
                    kind: DefinitionKind::Type(TypeDefinition::Enumeration {
                        variants: variants.into_boxed_slice(),
                    }),
                    span: start_span.expand_to(identifier_span),
                })))
            }
            TokenKind::Ticker => {
                self.consume_token()?; // Ticker

                let mut interval_ms = None;
                if let Some(TokenKind::ParenLeft) = self.current_token_kind() {
                    self.consume_token()?; // ParenLeft
                    interval_ms = Some(self.parse_expression(None, &[TokenKind::ParenRight])?);
                    self.consume_token()?; // ParenRight
                }

                self.expect_token_from(&[TokenKind::CurlyLeft])?;
                let tick_action = self.parse_action(&[])?;

                Ok(Some(Declaration::Ticker(TickerDeclaration {
                    interval_ms: interval_ms.map(Box::new),
                    tick_action: Box::new(tick_action),
                    span: start_span,
                })))
            }
            TokenKind::Public => {
                self.consume_token()?; // Public

                self.expect_token_from(&[TokenKind::CurlyLeft])?;
                self.consume_token()?; // CurlyLeft
                let mut lines = Vec::new();

                while !matches!(self.current_token_kind(), Some(TokenKind::CurlyRight)) {
                    match &self.get_token()?.kind {
                        TokenKind::Action => {
                            let action = self.parse_action(&[])?;
                            lines.push(PublicLine {
                                span: action.span,
                                kind: PublicLineKind::Action(action),
                            });

                            self.expect_token_from(&[TokenKind::Semicolon, TokenKind::CurlyRight])?;
                        }
                        TokenKind::Slider => {
                            self.consume_token()?; // Slider
                            let (var_identifier, span) = self.expect_identifier()?;
                            self.consume_token()?; // Identifier
                            lines.push(PublicLine {
                                kind: PublicLineKind::Slider {
                                    var_identifier,
                                },
                                span,
                            });

                            self.expect_token_from(&[TokenKind::Semicolon, TokenKind::CurlyRight])?;
                        }
                        _ => {
                            let expression = self.parse_expression(None, &[TokenKind::Semicolon, TokenKind::CurlyRight])?;
                            lines.push(PublicLine {
                                span: expression.span,
                                kind: PublicLineKind::Expression(expression),
                            });
                        }
                    }

                    if let Some(TokenKind::Semicolon) = self.current_token_kind() {
                        self.consume_token()?; // Semicolon
                    }
                }
                self.consume_token()?; // CurlyRight

                Ok(Some(Declaration::Public(PublicDeclaration {
                    lines: lines.into_boxed_slice(),
                    span: start_span,
                })))
            }
            TokenKind::Display => {
                self.consume_token()?; // Display

                self.expect_token_from(&[TokenKind::CurlyLeft])?;
                self.consume_token()?; // CurlyLeft
                let mut elements = Vec::new();

                while !matches!(self.current_token_kind(), Some(TokenKind::CurlyRight)) {
                    let element_expression = self.parse_expression(None, &[TokenKind::Colon, TokenKind::Semicolon, TokenKind::CurlyRight])?;

                    let mut element_span = element_expression.span;
                    let mut attributes = Vec::new();
                    if let Some(TokenKind::Colon) = self.current_token_kind() {
                        self.consume_token()?; // Colon

                        while !matches!(self.current_token_kind(), Some(TokenKind::Semicolon | TokenKind::CurlyRight)) {
                            let (attribute_key, attribute_key_span) = self.expect_identifier()?;
                            self.consume_token()?; // Identifier

                            let token = self.expect_token_from(&[TokenKind::ParenLeft, TokenKind::CurlyLeft])?;
                            let attribute_value = match token.kind {
                                TokenKind::ParenLeft => {
                                    self.consume_token()?; // ParenLeft
                                    let arguments = self.parse_argument_list()?;
                                    element_span = element_span.expand_to(self.current_span());
                                    self.consume_token()?; // ParenRight

                                    DisplayAttributeValue::Arguments(arguments)
                                }
                                TokenKind::CurlyLeft => {
                                    let action = self.parse_action(&[])?;
                                    element_span = element_span.expand_to(action.span);

                                    DisplayAttributeValue::Action(action)
                                }
                                _ => unreachable!()
                            };

                            let token = self.expect_token_from(&[TokenKind::Comma, TokenKind::Semicolon, TokenKind::CurlyRight])?;
                            if let TokenKind::Comma = token.kind {
                                self.consume_token()?; // Comma
                            }

                            attributes.push(DisplayAttribute {
                                key: attribute_key,
                                key_span: attribute_key_span,
                                value: attribute_value,
                            })
                        }
                    }

                    elements.push(DisplayElement {
                        expression: element_expression,
                        attributes: attributes.into_boxed_slice(),
                        span: element_span,
                    });

                    if let Some(TokenKind::Semicolon) = self.current_token_kind() {
                        self.consume_token()?; // Semicolon
                    }
                }
                self.consume_token()?; // CurlyRight

                Ok(Some(Declaration::Display(DisplayDeclaration {
                    elements: elements.into_boxed_slice(),
                    span: start_span,
                })))
            }
            _ => unreachable!(),
        }
    }

    pub fn parse_all_declarations(&mut self) -> crate::Result<Vec<Declaration>> {
        let mut declarations = Vec::new();

        while let Some(declaration) = self.parse_declaration()? {
            declarations.push(declaration);
        }

        Ok(declarations)
    }
}
