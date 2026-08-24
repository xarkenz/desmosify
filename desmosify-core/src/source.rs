use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub struct SourceFile<'a> {
    pub path: &'a Path,
    pub content: &'a str,
}

#[derive(Clone, Debug)]
pub struct SourceFiles<'a> {
    files: Vec<SourceFile<'a>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SourceFileID(usize);

impl<'a> SourceFiles<'a> {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            files: Vec::with_capacity(capacity),
        }
    }

    pub fn add(&mut self, file: SourceFile<'a>) -> SourceFileID {
        let id = self.files.len();
        self.files.push(file);
        SourceFileID(id)
    }

    pub fn get(&self, id: SourceFileID) -> SourceFile<'a> {
        self.files[id.0]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Span {
    pub source_id: SourceFileID,
    pub start_index: usize,
    pub length: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct SpanContext<'a> {
    pub path: &'a Path,
    pub content: &'a str,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Span {
    pub fn tail_point(&self) -> Self {
        Self {
            source_id: self.source_id,
            start_index: self.start_index + self.length,
            length: 0,
        }
    }

    pub fn expand_to(&self, end_span: Self) -> Self {
        if self.source_id != end_span.source_id {
            panic!("source IDs do not match");
        }

        Self {
            source_id: self.source_id,
            start_index: self.start_index,
            length: end_span.start_index.checked_add(end_span.length)
                .and_then(|end_index| end_index.checked_sub(self.start_index))
                .expect("end span comes before start span"),
        }
    }

    pub fn get_context<'a>(&self, sources: &SourceFiles<'a>) -> SpanContext<'a> {
        let source = sources.get(self.source_id);

        fn is_newline_related(ch: char) -> bool {
            matches!(ch, '\r' | '\n')
        }
        fn count_newlines(string: &str) -> usize {
            string
                .bytes()
                .filter(|&ch| ch == b'\n')
                .count()
        }

        let context_start_index = source.content[..self.start_index]
            .rfind(is_newline_related)
            .map_or(0, |newline_index| newline_index + 1);
        let context_end_index = source.content[self.start_index + self.length..]
            .find(is_newline_related)
            .map_or(source.content.len(), |offset| self.start_index + self.length + offset);
        let last_line_start_index = &source.content[..self.start_index + self.length]
            .rfind(is_newline_related)
            .map_or(0, |newline_index| newline_index + 1);

        let content = &source.content[context_start_index..context_end_index];
        let start_line = count_newlines(&source.content[..context_start_index]);
        let end_line = start_line + count_newlines(content);

        SpanContext {
            path: source.path,
            content,
            start_line,
            start_column: self.start_index - context_start_index,
            end_line,
            end_column: self.start_index + self.length - last_line_start_index,
        }
    }
}
