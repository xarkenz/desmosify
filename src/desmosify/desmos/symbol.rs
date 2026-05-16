use std::collections::HashMap;
use crate::desmos::{GraphBinaryKind, GraphExpression};

pub struct AsciiWords<'a>(&'a str);

impl<'a> Iterator for AsciiWords<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Find the next character to be included in a word and remove everything before it.
        let start_index = self.0.find(|ch: char| ch.is_ascii_alphanumeric())?;
        self.0 = &self.0[start_index..];

        let mut char_indices = self.0.char_indices();
        let (_, first_char) = char_indices.next().unwrap();

        // The word should be either purely alphabetic or purely numeric, not a combination of both.
        let matches_word = if first_char.is_ascii_digit() {
            |ch: char| ch.is_ascii_digit()
        } else {
            // An uppercase letter should begin a new word.
            |ch: char| ch.is_ascii_lowercase()
        };

        // Get the offset following the last character in the word.
        let end_index = char_indices
            .find_map(|(index, ch)| (!matches_word(ch)).then_some(index))
            .unwrap_or(self.0.len());

        let (word, rest) = self.0.split_at(end_index);
        self.0 = rest;
        Some(word)
    }
}

pub fn to_subscript(identifier: &str) -> String {
    let mut subscript = String::new();

    for word in AsciiWords(identifier) {
        let mut char_indices = word.char_indices();
        let (_, first_char) = char_indices.next().unwrap();

        // Capitalize the first character and use the rest as-is.
        subscript.push(first_char.to_ascii_uppercase());
        subscript.push_str(&word[char_indices.offset()..]);
    }

    // Remove trailing digits so a number can reliably be appended for versioning.
    while subscript.chars().rev().next().is_some_and(|ch| ch.is_ascii_digit()) {
        subscript.pop();
    }

    subscript
}

pub struct SymbolTable {
    symbol_prefix: GraphExpression,
    next_symbol_versions: HashMap<Box<str>, u64>,
    identifier_subscripts: HashMap<Box<str>, String>,
}

impl SymbolTable {
    pub fn new(symbol_prefix: GraphExpression) -> Self {
        Self {
            symbol_prefix,
            next_symbol_versions: HashMap::new(),
            identifier_subscripts: HashMap::new(),
        }
    }

    pub fn symbol_prefix(&self) -> &GraphExpression {
        &self.symbol_prefix
    }

    pub fn get_symbol_subscript(&mut self, identifier: &str) -> String {
        if let Some(subscript) = self.identifier_subscripts.get(identifier) {
            subscript.clone()
        }
        else {
            let mut subscript = to_subscript(identifier);

            if let Some(next_version) = self.next_symbol_versions.get_mut(subscript.as_str()) {
                let version = std::mem::replace(next_version, *next_version + 1);
                use std::fmt::Write;
                write!(subscript, "{version}").unwrap();
            }
            else {
                self.next_symbol_versions.insert(subscript.as_str().into(), 0);
            }

            self.identifier_subscripts.insert(identifier.into(), subscript.clone());
            subscript
        }
    }

    pub fn get_symbol(&mut self, identifier: &str) -> GraphExpression {
        GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(self.symbol_prefix().clone()),
            rhs: Box::new(GraphExpression::Alphanumeric(self.get_symbol_subscript(identifier))),
        }
    }
}
