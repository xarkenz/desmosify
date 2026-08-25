use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub struct SourceFile<'a> {
    pub path: &'a Path,
    pub content: &'a str,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SourceHandle(usize);

impl SourceHandle {
    pub fn file<'a>(self, sources: &SourceFiles<'a>) -> SourceFile<'a> {
        sources.get(self)
    }
}

#[derive(Clone, Debug)]
pub struct SourceFiles<'a> {
    files: Vec<SourceFile<'a>>,
}

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

    pub fn add(&mut self, file: SourceFile<'a>) -> SourceHandle {
        let index = self.files.len();
        self.files.push(file);
        SourceHandle(index)
    }

    pub fn get(&self, handle: SourceHandle) -> SourceFile<'a> {
        self.files[handle.0]
    }

    pub fn handles(&self) -> SourceHandles<'a, '_> {
        SourceHandles::new(self)
    }
}

impl<'a> FromIterator<SourceFile<'a>> for SourceFiles<'a> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = SourceFile<'a>>,
    {
        Self {
            files: Vec::from_iter(iter),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SourceHandles<'a, 'b> {
    sources: &'b SourceFiles<'a>,
    next_index: usize,
}

impl<'a, 'b> SourceHandles<'a, 'b> {
    fn new(sources: &'b SourceFiles<'a>) -> Self {
        Self {
            sources,
            next_index: 0,
        }
    }
}

impl<'a, 'b> Iterator for SourceHandles<'a, 'b> {
    type Item = SourceHandle;

    fn next(&mut self) -> Option<Self::Item> {
        (self.next_index < self.sources.files.len()).then(|| {
            let index = self.next_index;
            self.next_index += 1;
            SourceHandle(index)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.sources.files.len();
        (size, Some(size))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Span {
    pub source: SourceHandle,
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
            source: self.source,
            start_index: self.start_index + self.length,
            length: 0,
        }
    }

    pub fn expand_to(&self, end_span: Self) -> Self {
        if self.source != end_span.source {
            panic!("sources do not match")
        }

        Self {
            source: self.source,
            start_index: self.start_index,
            length: end_span.start_index.checked_add(end_span.length)
                .and_then(|end_index| end_index.checked_sub(self.start_index))
                .expect("end span comes before start span"),
        }
    }

    pub fn get_context<'a>(&self, sources: &SourceFiles<'a>) -> SpanContext<'a> {
        let source = sources.get(self.source);

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
