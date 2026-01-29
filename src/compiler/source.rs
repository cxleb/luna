use std::str::Chars;

#[derive(Clone)]
pub struct Source<'a> {
    contents: Chars<'a>,
    line_no: usize,
    col_no: usize,
}

impl<'a> Source<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self {
            contents: contents.chars(),
            line_no: 1,
            col_no: 1,
        }
    }

    pub fn peek_char(&self) -> Option<char> {
        self.contents.clone().next()
    }

    /**
    Matches the contents to a str, if the internal iterator runs out of chars
    in the case of an EOF, it returns false, not an None like in other functions
     */
    pub fn peek_str(&self, s: &str) -> bool {
        let mut iter = self.contents.clone();
        for c in s.chars() {
            if Some(c) != iter.next() {
                return false;
            }
        }
        true
    }

    pub fn next(&mut self) -> Option<char> {
        let char = self.contents.next()?;
        self.col_no += 1;
        if char == '\n' {
            self.col_no = 0;
            self.line_no += 1;
        }
        Some(char)
    }

    pub fn advance(&mut self, n: usize) -> Result<(), usize> {
        for a in 0..n {
            if let None = self.next() {
                return Err(a);
            }
        }
        Ok(())
    }

    pub fn accum<F: Fn(char, usize) -> bool>(&mut self, f: F) -> &'a str {
        let contents = self.contents.as_str();
        let mut length = 0;

        loop {
            let next = self.peek_char();
            if let Some(c) = next {
                if f(c, length) {
                    length += 1;
                    self.next();
                    continue;
                }
            }
            break;
        }

        let (str, _) = contents.split_at(length);

        return str;
    }

    // Allows for the parsing of strings and templates
    // Need to refactor to minimise the allocations
    pub fn accum_string<F: FnMut(char, &mut Chars<'a>) -> bool>(&mut self, mut f: F) -> String {
        let mut contents = String::new();

        loop {
            let mut clone = self.contents.clone();
            let next = clone.next();
            if let Some(c) = next {
                if c == '\\' {
                    self.next();
                    contents.push(self.parse_escape_sequence());
                    continue;
                }
                if !f(c, &mut clone) {
                    contents.push(c);
                    self.next();
                    continue;
                }
            }
            break;
        }

        return contents;
    }

    fn parse_escape_sequence(&mut self) -> char {
        let escape_char = self.next().unwrap();
        match escape_char {
            '\'' => '\'',
            '\"' => '\"',
            '\\' => '\\',
            'b' => 'b',
            'f' => 'f',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'v' => 'v',
            'u' => todo!("Unicode escape sequences havent been implemented yet"),
            _ => panic!("Invalid escape sequence!"),
        }
    }

    pub fn line_no(&self) -> usize {
        self.line_no
    }

    pub fn col_no(&self) -> usize {
        self.col_no
    }
}
