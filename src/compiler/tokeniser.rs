use crate::compiler::SourceLoc;

use super::source::Source;
use super::token::{Punctuation, Token, TokenKind, keyword};

/**
   In Section 12 of the spec it defines:
   - InputElementDiv
   - InputElementRegex
   - InputElementRegexOrTemplateTail
   - InputElementTemplateTail

   Each of those is a list of the tokens which can occur next due to the
   contextual grammer of JavaScript. So, in an effort to be as close to the
   spec as possible. We do the same thing here and the tokeniser will know
   what the mode is.
*/
#[derive(PartialEq, Copy, Clone)]
pub enum TokeniserMode {
    Div,
    Regex,
    RegexOrTemplateTail,
    TemplateTail,
}

#[derive(Clone)]
pub struct Tokeniser<'a> {
    source: Source<'a>,
}

impl<'a> Tokeniser<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self {
            source: Source::new(contents),
        }
    }

    fn eat_single_line_comment(&mut self) -> Option<()> {
        if !self.source.peek_str("//") {
            return Some(());
        }

        self.source.advance(2).unwrap();

        loop {
            let c = self.source.next()?;
            if c == '\n' {
                return Some(());
            }
        }
    }

    fn eat_multi_line_comment(&mut self) -> Option<()> {
        if !self.source.peek_str("/*") {
            return Some(());
        }

        self.source.advance(2).unwrap();

        loop {
            let c = self.source.next()?;
            if c == '*' {
                if self.source.peek_char() == Some('/') {
                    self.source.next();
                    return Some(());
                }
            }
        }
    }

    // While this function is called each whitespace
    // Comments are defined as to be treated like whitespace in the spec
    // therefore we shall do that too.
    fn eat_whitespace(&mut self) -> Option<()> {
        loop {
            let c = self.source.peek_char()?;
            if c.is_whitespace() {
                self.source.next();
            } else if self.source.peek_str("//") {
                self.eat_single_line_comment();
            } else if self.source.peek_str("/*") {
                self.eat_multi_line_comment();
            } else {
                return Some(());
            }
        }
    }

    pub fn peek(&self, mode: TokeniserMode) -> Option<Token> {
        self.clone().next(mode)
    }

    pub fn next(&mut self, mode: TokeniserMode) -> Option<Token> {
        self.eat_whitespace()?;

        let loc = SourceLoc {
            col: self.col_no(),
            line: self.line_no(),
            len: 0,
        };
        let c = self.source.peek_char()?;

        if c.is_alphabetic() || c == '_' {
            let str = self.source.accum(|c, _| c.is_alphanumeric() || c == '_');
            if str == "_" {
                Some(Token::new(loc, TokenKind::Underscore))
            } else if let Some(token) = keyword(str) {
                Some(Token::new(loc, TokenKind::Keyword(token)))
            } else {
                Some(Token::new_string(
                    loc,
                    TokenKind::Identifier,
                    String::from(str),
                ))
            }
        } else if c == '0' {
            if self.source.peek_str("0x") || self.source.peek_str("0X") {
                self.source.advance(2).unwrap();
                let str = self.source.accum(|c, _| c.is_ascii_hexdigit());
                Some(Token::new_int(
                    loc,
                    TokenKind::IntegerLiteral,
                    i64::from_str_radix(str, 16).unwrap(),
                ))
            } else if self.source.peek_str("0o") || self.source.peek_str("0O") {
                self.source.advance(2).unwrap();
                let str = self.source.accum(|c, _| matches!(c, '0'..='8'));
                Some(Token::new_int(
                    loc,
                    TokenKind::IntegerLiteral,
                    i64::from_str_radix(str, 8).unwrap(),
                ))
            } else {
                let str = self.source.accum(|c, _| c.is_numeric() || c == '.');
                if str.contains('.') {
                    Some(Token::new_float(
                        loc,
                        TokenKind::NumberLiteral,
                        str.parse().unwrap(),
                    ))
                } else {
                    Some(Token::new_int(
                        loc,
                        TokenKind::IntegerLiteral,
                        i64::from_str_radix(str, 10).unwrap(),
                    ))
                }
            }
        } else if c.is_numeric() {
            let str = self.source.accum(|c, _| c.is_numeric() || c == '.');
            if str.contains('.') {
                Some(Token::new_float(
                    loc,
                    TokenKind::NumberLiteral,
                    str.parse().unwrap(),
                ))
            } else {
                Some(Token::new_int(
                    loc,
                    TokenKind::IntegerLiteral,
                    i64::from_str_radix(str, 10).unwrap(),
                ))
            }
        }
        else if c == '\"' {
            self.source.next();
            let literal = self
                .source
                .accum_string(|c, chars| c == '"' || (c == '$' && chars.next() == Some('{')));
            let end = self.source.next().expect("Unfinished string or template sequence");
            if end == '"' {
                Some(Token::new_string(
                    loc,
                    TokenKind::StringLiteral,
                    literal,
                ))
            } else if end == '$' {
                // consume '{' which should already checked
                self.source.next();
                Some(Token::new_string(loc, TokenKind::TemplateHead, literal))
            } else {
                panic!(
                    "Something bad happened parsing a no substituion \
                    template or template head"
                )
            }
        } else if (mode == TokeniserMode::RegexOrTemplateTail
            || mode == TokeniserMode::TemplateTail)
            && c == '}'
        {
            self.source.next();
            // once we matched we either need to eat all valid template
            // characters
            let literal = self
                .source
                .accum_string(|c, chars| c == '"' || (c == '$' && chars.next() == Some('{')));
            let end = self.source.next().expect("Unfinished template sequence");
            if end == '"' {
                Some(Token::new_string(loc, TokenKind::TemplateTail, literal))
            } else if end == '$' {
                // consume '{' which should already checked
                self.source.next();
                Some(Token::new_string(loc, TokenKind::TemplateMiddle, literal))
            } else {
                panic!(
                    "Something bad happened parsing a template middle \
                    or template tail"
                )
            }
        } else if (mode == TokeniserMode::Div || mode == TokeniserMode::TemplateTail) && c == '/' {
            if self.source.peek_str("/=") {
                self.source.advance(2).unwrap();
                Some(Token::new(
                    loc,
                    TokenKind::Punctuation(Punctuation::ForwardSlashEquals),
                ))
            } else {
                self.source.next();
                Some(Token::new(
                    loc,
                    TokenKind::Punctuation(Punctuation::ForwardSlash),
                ))
            }
        } else if (mode == TokeniserMode::Regex || mode == TokeniserMode::RegexOrTemplateTail)
            && c == '/'
        {
            // eats the /
            self.source.next();
            let mut is_flags = false;
            let mut regex = self.source.accum_string(|c, _| {
                if is_flags {
                    !c.is_alphabetic()
                } else {
                    is_flags = c == '/';
                    false
                }
            });
            regex.insert(0, '/');
            Some(Token::new_string(loc, TokenKind::Regex, regex))
        } else if (mode == TokeniserMode::Div || mode == TokeniserMode::Regex) && c == '}' {
            self.source.next();
            Some(Token::new(
                loc,
                TokenKind::Punctuation(Punctuation::RightBrace),
            ))
        } else {
            let tok = match c {
                '&' => {
                    if self.source.peek_str("&&=") {
                        self.source.advance(3).unwrap();
                        Punctuation::AndAndEquals
                    } else if self.source.peek_str("&&") {
                        self.source.advance(2).unwrap();
                        Punctuation::AndAnd
                    } else if self.source.peek_str("&=") {
                        self.source.advance(2).unwrap();
                        Punctuation::AndEquals
                    } else {
                        self.source.next();
                        Punctuation::And
                    }
                }
                '|' => {
                    if self.source.peek_str("||=") {
                        self.source.advance(3).unwrap();
                        Punctuation::BarBarEquals
                    } else if self.source.peek_str("||") {
                        self.source.advance(2).unwrap();
                        Punctuation::BarBar
                    } else if self.source.peek_str("|=") {
                        self.source.advance(2).unwrap();
                        Punctuation::BarEquals
                    } else {
                        self.source.next();
                        Punctuation::Bar
                    }
                }
                '^' => {
                    if self.source.peek_str("^=") {
                        self.source.advance(2).unwrap();
                        Punctuation::CaretEquals
                    } else {
                        self.source.next();
                        Punctuation::Caret
                    }
                }
                ':' => {
                    self.source.next();
                    Punctuation::Colon
                }
                ',' => {
                    self.source.next();
                    Punctuation::Comma
                }
                '.' => {
                    if self.source.peek_str("...") {
                        self.source.advance(3).unwrap();
                        Punctuation::DotDotDot
                    } else {
                        self.source.next();
                        Punctuation::Dot
                    }
                }
                '=' => {
                    if self.source.peek_str("===") {
                        self.source.advance(3).unwrap();
                        Punctuation::EqualsEqualsEquals
                    } else if self.source.peek_str("==") {
                        self.source.advance(2).unwrap();
                        Punctuation::EqualsEquals
                    } else if self.source.peek_str("=>") {
                        self.source.advance(2).unwrap();
                        Punctuation::EqualsRightAngle
                    } else {
                        self.source.next();
                        Punctuation::Equals
                    }
                }
                '!' => {
                    if self.source.peek_str("!==") {
                        self.source.advance(3).unwrap();
                        Punctuation::ExclamationEqualsEquals
                    } else if self.source.peek_str("!=") {
                        self.source.advance(2).unwrap();
                        Punctuation::ExclamationEquals
                    } else {
                        self.source.next();
                        Punctuation::Exclamation
                    }
                }
                '<' => {
                    if self.source.peek_str("<<=") {
                        self.source.advance(3).unwrap();
                        Punctuation::LeftAngleLeftAngleEquals
                    } else if self.source.peek_str("<=") {
                        self.source.advance(2).unwrap();
                        Punctuation::LeftAngleEquals
                    } else if self.source.peek_str("<<") {
                        self.source.advance(2).unwrap();
                        Punctuation::LeftAngleLeftAngle
                    } else {
                        self.source.next();
                        Punctuation::LeftAngle
                    }
                }
                '{' => {
                    self.source.next();
                    Punctuation::LeftBrace
                }
                '[' => {
                    self.source.next();
                    Punctuation::LeftBracket
                }
                '(' => {
                    self.source.next();
                    Punctuation::LeftParenthesis
                }
                '-' => {
                    if self.source.peek_str("-=") {
                        self.source.advance(2).unwrap();
                        Punctuation::MinusEquals
                    } else if self.source.peek_str("--") {
                        self.source.advance(2).unwrap();
                        Punctuation::MinusMinus
                    } else {
                        self.source.next();
                        Punctuation::Minus
                    }
                }
                '%' => {
                    if self.source.peek_str("%=") {
                        self.source.advance(2).unwrap();
                        Punctuation::PercentageEquals
                    } else {
                        self.source.next();
                        Punctuation::Percentage
                    }
                }
                '+' => {
                    if self.source.peek_str("+=") {
                        self.source.advance(2).unwrap();
                        Punctuation::PlusEquals
                    } else if self.source.peek_str("++") {
                        self.source.advance(2).unwrap();
                        Punctuation::PlusPlus
                    } else {
                        self.source.next();
                        Punctuation::Plus
                    }
                }
                '?' => {
                    if self.source.peek_str("??=") {
                        self.source.advance(3).unwrap();
                        Punctuation::QuestionQuestionEquals
                    } else if self.source.peek_str("??") {
                        self.source.advance(2).unwrap();
                        Punctuation::QuestionQuestion
                    } else {
                        self.source.next();
                        Punctuation::QuestionMark
                    }
                }
                '>' => {
                    if self.source.peek_str(">>>=") {
                        self.source.advance(4).unwrap();
                        Punctuation::RightAngleRightAngleRightAngleEquals
                    } else if self.source.peek_str(">>>") {
                        self.source.advance(3).unwrap();
                        Punctuation::RightAngleRightAngleRightAngle
                    } else if self.source.peek_str(">>=") {
                        self.source.advance(3).unwrap();
                        Punctuation::RightAngleRightAngleEquals
                    } else if self.source.peek_str(">>") {
                        self.source.advance(2).unwrap();
                        Punctuation::RightAngleRightAngle
                    } else if self.source.peek_str(">=") {
                        self.source.advance(2).unwrap();
                        Punctuation::RightAngleEquals
                    } else {
                        self.source.next();
                        Punctuation::RightAngle
                    }
                }
                '*' => {
                    if self.source.peek_str("**=") {
                        self.source.advance(3).unwrap();
                        Punctuation::MultiplyMultiplyEquals
                    } else if self.source.peek_str("**") {
                        self.source.advance(2).unwrap();
                        Punctuation::MultiplyMultiply
                    } else if self.source.peek_str("*=") {
                        self.source.advance(2).unwrap();
                        Punctuation::MultiplyEquals
                    } else {
                        self.source.next();
                        Punctuation::Multiply
                    }
                }
                ']' => {
                    self.source.next();
                    Punctuation::RightBracket
                }
                ')' => {
                    self.source.next();
                    Punctuation::RightParenthesis
                }
                ';' => {
                    self.source.next();
                    Punctuation::SemiColon
                }
                '~' => {
                    self.source.next();
                    Punctuation::Tilde
                }
                '_' => {
                    self.source.next();
                    Punctuation::Underscore
                }
                _ => panic!(
                    "Unexpected Token \"{}\" at {}:{}",
                    c,
                    self.line_no(),
                    self.col_no()
                ),
            };
            Some(Token::new(loc, TokenKind::Punctuation(tok)))
        }
    }

    pub fn line_no(&self) -> usize {
        self.source.line_no()
    }

    pub fn col_no(&self) -> usize {
        self.source.col_no()
    }
}

#[allow(unused_imports)]
mod test {
    use crate::compiler::token::KEYWORDS_MAP;

    use super::super::token::Keywords;
    use super::*;

    #[test]
    fn whitespace() {
        let js = " \t     \nident\t   \n ident";
        let mut file = Tokeniser::new(js);
        assert_token(
            file.next(TokeniserMode::Div),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::Identifier,
                String::from("ident"),
            )),
        );
        assert_token(
            file.next(TokeniserMode::Div),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::Identifier,
                String::from("ident"),
            )),
        );
    }

    #[test]
    fn comments() {
        let js = "//comment\nident/* comment\n* */ident";
        let mut file = Tokeniser::new(js);
        assert_token(
            file.next(TokeniserMode::Div),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::Identifier,
                String::from("ident"),
            )),
        );
        assert_token(
            file.next(TokeniserMode::Div),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::Identifier,
                String::from("ident"),
            )),
        );
    }

    #[test]
    fn individual_tokens() {
        let token_tests = [
            (
                "identifier",
                Token::new_string(
                    SourceLoc::default(),
                    TokenKind::Identifier,
                    String::from("identifier"),
                ),
            ),
            (
                "100.10",
                Token::new_float(SourceLoc::default(), TokenKind::NumberLiteral, 100.10),
            ),
            (
                "0.10",
                Token::new_float(SourceLoc::default(), TokenKind::NumberLiteral, 0.10),
            ),
            (
                "0x100",
                Token::new_int(SourceLoc::default(), TokenKind::IntegerLiteral, 256),
            ),
            (
                "0o100",
                Token::new_int(SourceLoc::default(), TokenKind::IntegerLiteral, 64),
            ),
            (
                "100",
                Token::new_int(SourceLoc::default(), TokenKind::IntegerLiteral, 100),
            ),
            (
                "\"string\"",
                Token::new_string(
                    SourceLoc::default(),
                    TokenKind::StringLiteral,
                    String::from("string"),
                ),
            ),
        ];
        for test in token_tests {
            let mut file = Tokeniser::new(test.0);
            assert_token(file.next(TokeniserMode::Div), Some(test.1));
            assert_eq!(file.next(TokeniserMode::Div), None);
        }
    }

    #[test]
    fn templates() {
        let nosubstitutiontemplate = "\"template\"";
        let template = "\"head${ident}middle${ident}tail\"";
        let mut file = Tokeniser::new(nosubstitutiontemplate);
        assert_token(
            file.next(TokeniserMode::Div),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::StringLiteral,
                String::from("template"),
            )),
        );

        let mut file = Tokeniser::new(template);
        assert_token(
            file.next(TokeniserMode::Div),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::TemplateHead,
                String::from("head"),
            )),
        );
        assert_token(
            file.next(TokeniserMode::TemplateTail),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::Identifier,
                String::from("ident"),
            )),
        );
        assert_token(
            file.next(TokeniserMode::TemplateTail),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::TemplateMiddle,
                String::from("middle"),
            )),
        );
        assert_token(
            file.next(TokeniserMode::TemplateTail),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::Identifier,
                String::from("ident"),
            )),
        );
        assert_token(
            file.next(TokeniserMode::TemplateTail),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::TemplateTail,
                String::from("tail"),
            )),
        );
    }

    #[test]
    fn keywords_tokens() {
        for test in KEYWORDS_MAP {
            let mut file = Tokeniser::new(test.0);
            assert_token(
                file.next(TokeniserMode::Div),
                Some(Token::new(SourceLoc::default(), TokenKind::Keyword(test.1))),
            );
            assert_eq!(file.next(TokeniserMode::Div), None);
        }
    }

    #[test]
    fn punctuation_tokens() {
        let js = "& && &&= &= | || ||= |= ^ ^= : , . ...  = == === => ! != !== \
        / /= < <= << <<= { [ ( - -= -- * *= ** **= % %= + += ++ ? ?? ??= > >= \
        >> >>= >>> >>>= } ] ) ; ~";
        let punctuation = &[
            Punctuation::And,
            Punctuation::AndAnd,
            Punctuation::AndAndEquals,
            Punctuation::AndEquals,
            Punctuation::Bar,
            Punctuation::BarBar,
            Punctuation::BarBarEquals,
            Punctuation::BarEquals,
            Punctuation::Caret,
            Punctuation::CaretEquals,
            Punctuation::Colon,
            Punctuation::Comma,
            Punctuation::Dot,
            Punctuation::DotDotDot,
            Punctuation::Equals,
            Punctuation::EqualsEquals,
            Punctuation::EqualsEqualsEquals,
            Punctuation::EqualsRightAngle,
            Punctuation::Exclamation,
            Punctuation::ExclamationEquals,
            Punctuation::ExclamationEqualsEquals,
            Punctuation::ForwardSlash,
            Punctuation::ForwardSlashEquals,
            Punctuation::LeftAngle,
            Punctuation::LeftAngleEquals,
            Punctuation::LeftAngleLeftAngle,
            Punctuation::LeftAngleLeftAngleEquals,
            Punctuation::LeftBrace,
            Punctuation::LeftBracket,
            Punctuation::LeftParenthesis,
            Punctuation::Minus,
            Punctuation::MinusEquals,
            Punctuation::MinusMinus,
            Punctuation::Multiply,
            Punctuation::MultiplyEquals,
            Punctuation::MultiplyMultiply,
            Punctuation::MultiplyMultiplyEquals,
            Punctuation::Percentage,
            Punctuation::PercentageEquals,
            Punctuation::Plus,
            Punctuation::PlusEquals,
            Punctuation::PlusPlus,
            Punctuation::QuestionMark,
            Punctuation::QuestionQuestion,
            Punctuation::QuestionQuestionEquals,
            Punctuation::RightAngle,
            Punctuation::RightAngleEquals,
            Punctuation::RightAngleRightAngle,
            Punctuation::RightAngleRightAngleEquals,
            Punctuation::RightAngleRightAngleRightAngle,
            Punctuation::RightAngleRightAngleRightAngleEquals,
            Punctuation::RightBrace,
            Punctuation::RightBracket,
            Punctuation::RightParenthesis,
            Punctuation::SemiColon,
            Punctuation::Tilde,
        ];
        let mut file = Tokeniser::new(js);
        for test in punctuation {
            assert_token(
                file.next(TokeniserMode::Div),
                Some(Token::new(
                    SourceLoc::default(),
                    TokenKind::Punctuation(*test),
                )),
            );
        }
    }

    #[test]
    fn regex() {
        let js = "/ab+c/g";
        let mut file = Tokeniser::new(js);
        assert_token(
            file.next(TokeniserMode::Regex),
            Some(Token::new_string(
                SourceLoc::default(),
                TokenKind::Regex,
                "/ab+c/g".into(),
            )),
        );
    }

    #[test]
    fn test_single_underscore_token() {
        let mut file = Tokeniser::new("_");
        let token = file.next(TokeniserMode::Div).unwrap();
        assert_eq!(token.kind, TokenKind::Underscore);
    }

    #[test]
    fn test_underscore_in_identifier() {
        let mut file = Tokeniser::new("test_var");
        let token = file.next(TokeniserMode::Div).unwrap();
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.get_string(), "test_var");
    }

    #[test]
    fn test_multiple_underscores_in_identifier() {
        let mut file = Tokeniser::new("test_var_name");
        let token = file.next(TokeniserMode::Div).unwrap();
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.get_string(), "test_var_name");
    }

    // Assert that two tokens are equal without caring about location
    fn assert_token(a: Option<Token>, b: Option<Token>) {
        if let Some(tok) = a {
            if let Some(tok2) = b {
                assert_eq!(tok.data, tok2.data);
                assert_eq!(tok.kind, tok2.kind);
            } else {
                panic!("Tokens not equal");
            }
        } else {
            panic!("Tokens not equal");
        }
    }
}
