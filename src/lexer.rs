#[derive(Debug)]
pub struct Lexer<'a> {
    pub content: &'a [char],
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Lexer { content }
    }

    fn trim_left(&mut self) {
        while !self.content.is_empty() && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    /// Slice the content with index n
    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[..n];
        self.content = &self.content[n..];

        token
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char]
    where
        P: FnMut(&char) -> bool,
    {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }

        self.chop(n)
    }

    /// Find the next token(individual word)
    pub fn next_token(&mut self) -> Option<String> {
        self.trim_left();
        if self.content.is_empty() {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(self.chop_while(|x| x.is_numeric()).iter().collect());
        }

        if self.content[0].is_alphanumeric() {
            return Some(
                self.chop_while(|x| x.is_alphanumeric())
                    .iter()
                    .map(|x| x.to_ascii_uppercase())
                    .collect(),
            );
        }

        return Some(self.chop(1).iter().collect());
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;

    #[test]
    fn trim_left_ok() {
        // arrange
        let input = [' ', ' ', 'a', 'n', 'd', 'y'];
        let mut lexer = Lexer::new(&input);
        let expected = ['a', 'n', 'd', 'y'];

        // act
        lexer.trim_left();

        // assert
        assert_eq!(expected, lexer.content);
    }

    #[test]
    fn chop_ok() {
        // arrange
        let input = ['a', 'n', 'd', 'y'];
        let mut lexer = Lexer::new(&input);
        let expected1 = ['a', 'n'];
        let expected2 = ['d', 'y'];

        // act
        let actual = lexer.chop(2);

        // assert
        assert_eq!(expected1, actual);
        assert_eq!(expected2, lexer.content);
    }

    #[test]
    fn chop_while_ok() {
        // arrange
        let input = ['a', 'a', 'a', 'a', 'n', '1', 'd', 'y'];
        let mut lexer = Lexer::new(&input);
        let expected1 = ['a', 'a', 'a', 'a'];

        // act
        let actual = lexer.chop_while(|x| *x == 'a');

        // assert
        assert_eq!(expected1, actual);
    }

    #[test]
    fn next_token_ok() {
        // arrange
        let input = [
            'a', 'n', 'd', 'y', '0', '1', ' ', '2', '3', '4', 'a', 'm', 'y',
        ];
        let mut lexer = Lexer::new(&input);
        let expected1 = Some(String::from("andy01".to_ascii_uppercase()));
        let expected2 = Some(String::from("234".to_ascii_uppercase()));
        let expected3 = Some(String::from("amy".to_ascii_uppercase()));

        // act
        let actual1 = lexer.next_token();
        let actual2 = lexer.next_token();
        let actual3 = lexer.next_token();

        // assert
        assert_eq!(expected1, actual1);
        assert_eq!(expected2, actual2);
        assert_eq!(expected3, actual3);
    }
}
