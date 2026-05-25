use super::*;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::Path;
use utf8_chars::BufReadCharsExt;

// TODO: string interning

#[derive(Debug)]
pub struct Scanner<T: BufRead> {
    source_id: usize,
    next_index: usize,
    line: usize,
    source: T,
    put_backs: Vec<char>,
}

impl Scanner<BufReader<File>> {
    pub fn from_path(source_id: usize, path: impl AsRef<Path>) -> crate::Result<Self> {
        File::open(path)
            .map(|file| Self::new(source_id, BufReader::new(file)))
            .map_err(|cause| Box::new(crate::Error {
                kind: crate::ErrorKind::FileOpen {
                    path: None,
                    cause,
                },
                span: Some(crate::Span {
                    source_id,
                    start_index: 0,
                    length: 0,
                }),
            }))
    }
}

impl<T: BufRead> Scanner<T> {
    pub fn new(source_id: usize, source: T) -> Self {
        Self {
            source_id,
            next_index: 0,
            line: 1,
            source,
            put_backs: Vec::new(),
        }
    }

    pub fn source_id(&self) -> usize {
        self.source_id
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn next_index(&self) -> usize {
        self.next_index
    }

    pub fn create_span(&self, start_index: usize, end_index: usize) -> crate::Span {
        crate::Span {
            source_id: self.source_id,
            start_index,
            length: end_index.checked_sub(start_index).expect("end span comes before start span"),
        }
    }

    pub fn next_token(&mut self) -> crate::Result<Option<Token>> {
        if let Some(ch) = self.next_non_space_char()? {
            if ch.is_ascii_digit() {
                self.put_back(ch);
                self.scan_numeric_literal().map(Some)
            }
            else if ch == '_' || ch.is_ascii_alphanumeric() {
                self.put_back(ch);
                self.scan_word_literal().map(Some)
            }
            else {
                if ch == '/' {
                    match self.next_char()? {
                        Some('/') => {
                            self.skip_line_comment()?;
                            return self.next_token();
                        }
                        Some('*') => {
                            self.skip_block_comment()?;
                            return self.next_token();
                        }
                        Some(next_ch) => {
                            self.put_back(next_ch);
                        }
                        None => {}
                    }
                }
                else if ch == '"' {
                    return self.scan_string_literal().map(Some);
                }
                else if ch == '\'' {
                    return self.scan_character_literal().map(Some);
                }
                self.put_back(ch);
                self.scan_symbolic_literal().map(Some)
            }
        }
        else {
            Ok(None)
        }
    }

    fn next_char(&mut self) -> crate::Result<Option<char>> {
        if let Some(ch) = self.put_backs.pop() {
            self.next_index += 1;
            Ok(Some(ch))
        }
        else {
            let read = self.source.read_char()
                .map_err(|cause| Box::new(crate::Error {
                    kind: crate::ErrorKind::FileRead {
                        path: None,
                        cause,
                    },
                    span: Some(crate::Span {
                        source_id: self.source_id,
                        start_index: self.next_index,
                        length: 0,
                    }),
                }))?;

            if let Some(ch) = read {
                if ch == '\n' {
                    self.line += 1;
                }
                self.next_index += 1;
                Ok(Some(ch))
            }
            else {
                Ok(None)
            }
        }
    }

    fn put_back(&mut self, ch: char) {
        self.put_backs.push(ch);
        self.next_index -= 1;
    }

    fn next_non_space_char(&mut self) -> crate::Result<Option<char>> {
        while let Some(ch) = self.next_char()? {
            if !ch.is_whitespace() {
                return Ok(Some(ch));
            }
        }

        Ok(None)
    }

    fn scan_alphanumeric_word(&mut self) -> crate::Result<(crate::Span, String)> {
        let start_index = self.next_index;
        let mut content = String::new();

        while let Some(ch) = self.next_char()? {
            match ch {
                '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' => {
                    content.push(ch);
                }
                _ => {
                    self.put_back(ch);
                    break;
                }
            }
        }

        Ok((
            self.create_span(start_index, self.next_index),
            content,
        ))
    }

    fn scan_numeric_literal(&mut self) -> crate::Result<Token> {
        let start_index = self.next_index;
        let mut content = String::new();
        let mut suffix = None;

        while let Some(ch) = self.next_char()? {
            match ch {
                '_' => {}
                '0'..='9' => {
                    content.push(ch);
                }
                '.' => {
                    // A dot could either be a decimal point or an access operation. We'll
                    // only consider it to be a decimal point if the following character is a digit.
                    match self.next_char()? {
                        Some(digit @ '0'..='9') => {
                            content.push('.');
                            content.push(digit);
                            return self.scan_real_literal_end(start_index, content);
                        }
                        Some(non_digit) => {
                            self.put_back(non_digit);
                        }
                        None => {}
                    }
                    self.put_back(ch);
                    break;
                }
                'E' | 'e' => {
                    content.push(ch);
                    return self.scan_real_literal_end(start_index, content);
                }
                'A'..='Z' | 'a'..='z' => {
                    // The only valid integer suffixes start with 'i' and 'u', but detect any
                    // letter here to provide more graceful error handling.
                    self.put_back(ch);
                    suffix = Some(self.scan_integer_suffix()?);
                    break;
                }
                _ => {
                    self.put_back(ch);
                    break;
                }
            }
        }

        let value = content.chars().fold(0, |value, digit| {
            10 * value + (digit as i128 - '0' as i128)
        });

        let _ = suffix;
        Ok(Token {
            kind: TokenKind::Literal(Literal::Integer(value)),
            span: self.create_span(start_index, self.next_index),
        })
    }

    fn scan_real_literal_end(&mut self, start_index: usize, mut content: String) -> crate::Result<Token> {
        let mut suffix = None;

        while let Some(ch) = self.next_char()? {
            match ch {
                '_' => {}
                '0'..='9' => {
                    content.push(ch);
                }
                '+' | '-' if matches!(content.chars().last(), Some('E' | 'e')) => {
                    content.push(ch);
                }
                'A'..='Z' | 'a'..='z' => {
                    self.put_back(ch);
                    suffix = Some(self.scan_real_suffix()?);
                    break;
                }
                _ => {
                    self.put_back(ch);
                    break;
                }
            }
        }

        let value = content.parse::<f64>()
            .map_err(|_| Box::new(crate::Error {
                kind: crate::ErrorKind::InvalidToken,
                span: Some(self.create_span(start_index, self.next_index)),
            }))?;

        let _ = suffix;
        Ok(Token {
            kind: TokenKind::Literal(Literal::Real(value)),
            span: self.create_span(start_index, self.next_index),
        })
    }

    /// Currently not supported; always returns `Err`.
    fn scan_integer_suffix(&mut self) -> crate::Result<()> {
        let (span, _content) = self.scan_alphanumeric_word()?;

        Err(Box::new(crate::Error {
            kind: crate::ErrorKind::InvalidLiteralSuffix,
            span: Some(span),
        }))
    }

    /// Currently not supported; always returns `Err`.
    fn scan_real_suffix(&mut self) -> crate::Result<()> {
        let (span, _content) = self.scan_alphanumeric_word()?;

        Err(Box::new(crate::Error {
            kind: crate::ErrorKind::InvalidLiteralSuffix,
            span: Some(span),
        }))
    }

    fn scan_word_literal(&mut self) -> crate::Result<Token> {
        let (span, content) = self.scan_alphanumeric_word()?;

        if let Some(keyword_token) = get_keyword_token_match(&content) {
            Ok(Token {
                kind: keyword_token.clone(),
                span,
            })
        }
        else {
            Ok(Token {
                kind: TokenKind::Literal(Literal::from_word(&content)),
                span,
            })
        }
    }

    fn scan_symbolic_literal(&mut self) -> crate::Result<Token> {
        let start_index = self.next_index;
        let mut content = String::new();

        // Consume characters as long as the current sequence is a valid token prefix
        while let Some(ch) = self.next_char()? {
            content.push(ch);
            let matches = get_symbolic_token_partial_matches(content.as_str());
            if matches.is_empty() {
                let ch = content.pop().unwrap();
                self.put_back(ch);
                break;
            }
        }

        // Backtrack to find the longest exact token match
        while !content.is_empty() {
            if let Some(symbolic_token) = get_symbolic_token_match(content.as_str()) {
                return Ok(Token {
                    kind: symbolic_token.clone(),
                    span: self.create_span(start_index, self.next_index),
                });
            }

            // Since no match was found, take away a character and try again
            self.put_back(content.pop().unwrap());
        }

        Err(Box::new(crate::Error {
            kind: crate::ErrorKind::InvalidToken,
            span: Some(self.create_span(start_index, start_index)),
        }))
    }

    fn scan_escaped_char(&mut self) -> crate::Result<Option<char>> {
        fn hex_digit_value(ch: char) -> Option<u8> {
            // why did i do this manually, you ask? idk man no real reason
            match ch {
                '0'..='9' => Some(ch as u8 - b'0'),
                'A'..='F' => Some(ch as u8 - b'A' + 10),
                'a'..='f' => Some(ch as u8 - b'a' + 10),
                _ => None
            }
        }

        let start_index = self.next_index;

        if let Some(ch) = self.next_char()? {
            if ch == '\\' {
                match self.next_char()? {
                    Some('\\') => {
                        Ok(Some('\\'))
                    }
                    Some('\"') => {
                        Ok(Some('\"'))
                    }
                    Some('\'') => {
                        Ok(Some('\''))
                    }
                    Some('n') => {
                        Ok(Some('\n'))
                    }
                    Some('t') => {
                        Ok(Some('\t'))
                    }
                    Some('x') => {
                        let mut byte = 0;
                        for _ in 0..2 {
                            if let Some(ch) = self.next_char()? {
                                byte *= 16;
                                byte += hex_digit_value(ch)
                                    .ok_or_else(|| Box::new(crate::Error {
                                        kind: crate::ErrorKind::InvalidHexEscapeDigit {
                                            what: ch,
                                        },
                                        span: Some(self.create_span(start_index, start_index + 4)),
                                    }))?;
                            }
                            else {
                                return Ok(None);
                            }
                        }

                        Ok(Some(byte as char))
                    }
                    Some('u') => {
                        let mut unicode = 0;
                        for _ in 0..4 {
                            if let Some(ch) = self.next_char()? {
                                unicode *= 16;
                                unicode += hex_digit_value(ch)
                                    .ok_or_else(|| Box::new(crate::Error {
                                        kind: crate::ErrorKind::InvalidHexEscapeDigit {
                                            what: ch,
                                        },
                                        span: Some(self.create_span(start_index, start_index + 4)),
                                    }))? as u16;
                            }
                            else {
                                return Ok(None);
                            }
                        }

                        if let Some(char_value) = char::from_u32(unicode as u32) {
                            Ok(Some(char_value))
                        }
                        else {
                            Err(Box::new(crate::Error {
                                kind: crate::ErrorKind::InvalidUnicode16Escape {
                                    value: unicode,
                                },
                                span: Some(self.create_span(start_index, start_index + 6)),
                            }))
                        }
                    }
                    Some('U') => {
                        let mut unicode = 0;
                        for _ in 0..8 {
                            if let Some(ch) = self.next_char()? {
                                unicode *= 16;
                                unicode += hex_digit_value(ch)
                                    .ok_or_else(|| Box::new(crate::Error {
                                        kind: crate::ErrorKind::InvalidHexEscapeDigit {
                                            what: ch,
                                        },
                                        span: Some(self.create_span(start_index, start_index + 10)),
                                    }))? as u32;
                            }
                            else {
                                return Ok(None);
                            }
                        }

                        if let Some(char_value) = char::from_u32(unicode) {
                            Ok(Some(char_value))
                        }
                        else {
                            Err(Box::new(crate::Error {
                                kind: crate::ErrorKind::InvalidUnicode32Escape {
                                    value: unicode,
                                },
                                span: Some(self.create_span(start_index, start_index + 10)),
                            }))
                        }
                    }
                    Some(ch) => {
                        Err(Box::new(crate::Error {
                            kind: crate::ErrorKind::InvalidCharacterEscape {
                                what: ch,
                            },
                            span: Some(self.create_span(start_index, self.next_index)),
                        }))
                    }
                    None => {
                        Ok(None)
                    }
                }
            }
            else {
                Ok(Some(ch))
            }
        }
        else {
            Ok(None)
        }
    }

    fn scan_string_literal(&mut self) -> crate::Result<Token> {
        let start_index = self.next_index - 1;
        let mut content = String::new();

        while let Some(ch) = self.next_char()? {
            if ch == '"' {
                return Ok(Token {
                    kind: TokenKind::Literal(Literal::String(content.into())),
                    span: self.create_span(start_index, self.next_index),
                });
            }
            else {
                self.put_back(ch);
                let ch = self.scan_escaped_char()?
                    .ok_or_else(|| Box::new(crate::Error {
                        kind: crate::ErrorKind::UnclosedString,
                        span: Some(self.create_span(start_index, start_index)),
                    }))?;
                content.push(ch);
            }
        }

        Err(Box::new(crate::Error {
            kind: crate::ErrorKind::UnclosedString,
            span: Some(self.create_span(start_index, start_index)),
        }))
    }

    fn scan_character_literal(&mut self) -> crate::Result<Token> {
        let start_index = self.next_index - 1;
        let char_value = self.scan_escaped_char()?
            .ok_or_else(|| Box::new(crate::Error {
                kind: crate::ErrorKind::UnclosedCharacter,
                span: Some(self.create_span(self.next_index, self.next_index)),
            }))?;

        if let Some('\'') = self.next_char()? {
            Ok(Token {
                kind: TokenKind::Literal(Literal::Character(char_value)),
                span: self.create_span(start_index, self.next_index),
            })
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::UnclosedCharacter,
                span: Some(self.create_span(self.next_index, self.next_index)),
            }))
        }
    }

    fn skip_line_comment(&mut self) -> crate::Result<()> {
        let mut escape_next_newline = false;

        while let Some(ch) = self.next_char()? {
            if ch == '\n' {
                if escape_next_newline {
                    escape_next_newline = false;
                }
                else {
                    break;
                }
            }
            else if ch == '\\' {
                escape_next_newline = !escape_next_newline;
            }
            else if !ch.is_whitespace() {
                escape_next_newline = false;
            }
        }

        Ok(())
    }

    fn skip_block_comment(&mut self) -> crate::Result<()> {
        // TODO: recursive block comments
        let start_index = self.next_index - 2;
        let mut escape_next_char = false;

        while let Some(ch) = self.next_char()? {
            if escape_next_char {
                escape_next_char = false;
            }
            else if ch == '*' {
                match self.next_char()? {
                    Some('/') => return Ok(()),
                    Some(next_ch) => self.put_back(next_ch),
                    None => break,
                }
            }
            else if ch == '\\' {
                escape_next_char = true;
            }
        }

        Err(Box::new(crate::Error {
            kind: crate::ErrorKind::UnclosedComment,
            span: Some(self.create_span(start_index, start_index + 2)),
        }))
    }
}
