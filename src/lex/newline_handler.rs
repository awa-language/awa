use itertools::{peek_nth, PeekNth};

#[derive(Debug)]
pub struct NewlineHandler<T: Iterator<Item = (u32, char)>> {
    input: PeekNth<T>,
    current_char: Option<(u32, char)>,
}

impl<T> NewlineHandler<T>
where
    T: Iterator<Item = (u32, char)>,
{
    pub fn new(source: T) -> Self {
        let mut newline_handler = NewlineHandler {
            input: peek_nth(source),
            current_char: None,
        };

        let _ = newline_handler.advance();

        newline_handler
    }

    fn advance(&mut self) -> Option<(u32, char)> {
        let result = self.current_char;
        self.current_char = self.input.next();

        result
    }

    pub fn peek_next(&mut self) -> Option<(u32, char)> {
        self.input.peek_nth(0).copied()
    }
}

impl<T> Iterator for NewlineHandler<T>
where
    T: Iterator<Item = (u32, char)>,
{
    type Item = (u32, char);

    fn next(&mut self) -> Option<Self::Item> {
        const CARRIAGE_RETURN: char = '\r';

        if let Some((carriage_return_location, CARRIAGE_RETURN)) = self.current_char {
            const LINE_FEED: char = '\n';

            if let Some((_, LINE_FEED)) = self.peek_next() {
                let _ = self.advance();
            }

            self.current_char = Some((carriage_return_location, LINE_FEED));
        }

        self.advance()
    }
}
