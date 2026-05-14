#[derive(Copy, Clone, Debug)]
pub enum BracketType {
    Parenthesis,
    Square,
    Curly,
    Pipe,
}

impl BracketType {
    pub fn left(&self) -> &'static str {
        match self {
            Self::Parenthesis => "(",
            Self::Square => "[",
            Self::Curly => "\\{",
            Self::Pipe => "|",
        }
    }

    pub fn right(&self) -> &'static str {
        match self {
            Self::Parenthesis => ")",
            Self::Square => "]",
            Self::Curly => "\\}",
            Self::Pipe => "|",
        }
    }
}

#[derive(Debug)]
pub enum LatexNode {
    Group {
        content: Latex,
    },
    Sqrt {
        index: Option<Latex>,
        radicand: Latex,
    },
    Frac {
        numerator: Latex,
        denominator: Latex,
    },
    Subscript {
        content: Latex,
    },
    Superscript {
        content: Latex,
    },
    Left {
        bracket_type: BracketType,
    },
    Right {
        bracket_type: BracketType,
    },
    OperatorName {
        content: String,
    },
    Escape {
        value: String,
    },
    Symbol {
        value: char,
    },
    Symbols {
        value: String,
    },
}

impl LatexNode {
    fn last_char_is_alphabetic(&self) -> bool {
        match self {
            Self::Group { .. } => false,
            Self::Sqrt { .. } => false,
            Self::Frac { .. } => false,
            Self::Subscript { .. } => false,
            Self::Superscript { .. } => false,
            Self::Left { .. } => false,
            Self::Right { .. } => false,
            Self::OperatorName { .. } => false,
            Self::Escape { value } => value.ends_with(char::is_alphabetic),
            Self::Symbol { value } => value.is_alphabetic(),
            Self::Symbols { value } => value.ends_with(char::is_alphabetic),
        }
    }
}

#[derive(Debug)]
pub struct Latex {
    nodes: Vec<LatexNode>,
}

impl Latex {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    pub fn from_nodes(nodes: Vec<LatexNode>) -> Self {
        Self { nodes }
    }

    pub fn add(mut self, mut latex: Latex) -> Self {
        self.nodes.append(&mut latex.nodes);
        self
    }

    pub fn add_node(mut self, node: LatexNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn add_group(mut self, content: Latex) -> Self {
        self.nodes.push(LatexNode::Group { content });
        self
    }

    pub fn add_sqrt(mut self, index: Option<Latex>, radicand: Latex) -> Self {
        self.nodes.push(LatexNode::Sqrt { index, radicand });
        self
    }

    pub fn add_frac(mut self, numerator: Latex, denominator: Latex) -> Self {
        self.nodes.push(LatexNode::Frac { numerator, denominator });
        self
    }

    pub fn add_superscript(mut self, content: Latex) -> Self {
        self.nodes.push(LatexNode::Superscript { content });
        self
    }

    pub fn add_subscript(mut self, content: Latex) -> Self {
        self.nodes.push(LatexNode::Subscript { content });
        self
    }

    pub fn add_left(mut self, bracket_type: BracketType) -> Self {
        self.nodes.push(LatexNode::Left { bracket_type });
        self
    }

    pub fn add_right(mut self, bracket_type: BracketType) -> Self {
        self.nodes.push(LatexNode::Right { bracket_type });
        self
    }

    pub fn add_operator_name(mut self, content: String) -> Self {
        self.nodes.push(LatexNode::OperatorName { content });
        self
    }

    pub fn add_escape(mut self, value: String) -> Self {
        self.nodes.push(LatexNode::Escape { value });
        self
    }

    pub fn add_symbol(mut self, value: char) -> Self {
        self.nodes.push(LatexNode::Symbol { value });
        self
    }

    pub fn add_symbols(mut self, value: String) -> Self {
        self.nodes.push(LatexNode::Symbols { value });
        self
    }
}

impl std::fmt::Display for Latex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut end_is_alphabetic = false;

        for node in &self.nodes {
            match node {
                LatexNode::Group { content } => {
                    write!(f, "{{{content}}}")?;
                }
                LatexNode::Sqrt { index, radicand } => {
                    write!(f, "\\sqrt")?;
                    if let Some(index) = index {
                        write!(f, "[{index}]")?;
                    }
                    write!(f, "{{{radicand}}}")?;
                }
                LatexNode::Frac { numerator, denominator } => {
                    write!(f, "\\frac{{{numerator}}}{{{denominator}}}")?;
                }
                LatexNode::Superscript { content } => {
                    write!(f, "^{{{content}}}")?;
                }
                LatexNode::Subscript { content } => {
                    write!(f, "_{{{content}}}")?;
                }
                LatexNode::Left { bracket_type } => {
                    write!(f, "\\left{}", bracket_type.left())?;
                }
                LatexNode::Right { bracket_type } => {
                    write!(f, "\\right{}", bracket_type.right())?;
                }
                LatexNode::OperatorName { content } => {
                    write!(f, "\\operatorname{{{content}}}")?;
                }
                LatexNode::Escape { value } => {
                    write!(f, "\\{value}")?;
                }
                LatexNode::Symbol { value } => match *value {
                    '&' | '%' | '$' | '#' | '{' | '}' => write!(f, "\\{value}")?,
                    '~' => write!(f, "\\sim")?,
                    c if end_is_alphabetic && c.is_alphabetic() => write!(f, " {value}")?,
                    _ => write!(f, "{value}")?,
                }
                LatexNode::Symbols { value } => {
                    if end_is_alphabetic && value.starts_with(|c: char| c.is_alphabetic()) {
                        write!(f, " {value}")?;
                    }
                    else {
                        write!(f, "{value}")?;
                    }
                }
            }

            end_is_alphabetic = node.last_char_is_alphabetic();
        }

        Ok(())
    }
}