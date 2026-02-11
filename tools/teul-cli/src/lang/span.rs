#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    pub fn merge(self, other: Span) -> Span {
        Span::new(self.start_line, self.start_col, other.end_line, other.end_col)
    }
}
