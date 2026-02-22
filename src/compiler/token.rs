use crate::compiler::SourceLoc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Keywords {
    Await,
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    False,
    Finally,
    For,
    Func,
    If,
    Import,
    In,
    Instanceof,
    Let,
    New,
    Not,
    Null,
    Return,
    Struct,
    Super,
    Switch,
    _Self,
    Throw,
    True,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    Yield,
}

pub const KEYWORDS_MAP: &'static [(&'static str, Keywords)] = &[
    ("await", Keywords::Await),
    ("break", Keywords::Break),
    ("case", Keywords::Case),
    ("catch", Keywords::Catch),
    ("class", Keywords::Class),
    ("const", Keywords::Const),
    ("continue", Keywords::Continue),
    ("debugger", Keywords::Debugger),
    ("default", Keywords::Default),
    ("delete", Keywords::Delete),
    ("do", Keywords::Do),
    ("else", Keywords::Else),
    ("enum", Keywords::Enum),
    ("export", Keywords::Export),
    ("extends", Keywords::Extends),
    ("false", Keywords::False),
    ("finally", Keywords::Finally),
    ("for", Keywords::For),
    ("func", Keywords::Func),
    ("if", Keywords::If),
    ("import", Keywords::Import),
    ("in", Keywords::In),
    ("instanceof", Keywords::Instanceof),
    ("let", Keywords::Let),
    ("new", Keywords::New),
    ("not", Keywords::Not),
    ("null", Keywords::Null),
    ("return", Keywords::Return),
    ("struct", Keywords::Struct),
    ("super", Keywords::Super),
    ("switch", Keywords::Switch),
    ("self", Keywords::_Self),
    ("throw", Keywords::Throw),
    ("true", Keywords::True),
    ("try", Keywords::Try),
    ("typeof", Keywords::Typeof),
    ("var", Keywords::Var),
    ("void", Keywords::Void),
    ("while", Keywords::While),
    ("with", Keywords::With),
    ("yield", Keywords::Yield),
];

pub fn keyword(k: &str) -> Option<Keywords> {
    for p in KEYWORDS_MAP.iter() {
        if p.0 == k {
            return Some(p.1);
        }
    }
    None
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Punctuation {
    And,                                  // &
    AndAnd,                               // &&
    AndAndEquals,                         // &&=
    AndEquals,                            // &=
    Bar,                                  // |
    BarBar,                               // ||
    BarBarEquals,                         // ||=
    BarEquals,                            // |=
    Caret,                                // ^
    CaretEquals,                          // ^=
    Colon,                                // :
    Comma,                                // ,
    Dot,                                  // .
    DotDotDot,                            // ...
    Equals,                               // =
    EqualsEquals,                         // ==
    EqualsEqualsEquals,                   // ===
    EqualsRightAngle,                     // =>
    Exclamation,                          // !
    ExclamationEquals,                    // !=
    ExclamationEqualsEquals,              // !==
    ForwardSlash,                         // /
    ForwardSlashEquals,                   // /=
    LeftAngle,                            // <
    LeftAngleEquals,                      // <=
    LeftAngleLeftAngle,                   // <<
    LeftAngleLeftAngleEquals,             // <<=
    LeftBrace,                            // {
    LeftBracket,                          // [
    LeftParenthesis,                      // (
    Minus,                                // -
    MinusEquals,                          // -=
    MinusMinus,                           // --
    Multiply,                             // *
    MultiplyEquals,                       // *=
    MultiplyMultiply,                     // **
    MultiplyMultiplyEquals,               // **=
    Percentage,                           // %
    PercentageEquals,                     // %=
    Plus,                                 // +
    PlusEquals,                           // +=
    PlusPlus,                             // ++
    QuestionMark,                         // ?
    QuestionQuestion,                     // ??
    QuestionQuestionEquals,               // ??=
    RightAngle,                           // >
    RightAngleEquals,                     // >=
    RightAngleRightAngle,                 // >>
    RightAngleRightAngleEquals,           // >>=
    RightAngleRightAngleRightAngle,       // >>>
    RightAngleRightAngleRightAngleEquals, // >>>=
    RightBrace,                           // }
    RightBracket,                         // ]
    RightParenthesis,                     // )
    SemiColon,                            // ;
    Tilde,                                // ~
    Underscore,                           // _
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    Keyword(Keywords),
    Identifier,
    Punctuation(Punctuation),
    StringLiteral,
    NumberLiteral,
    IntegerLiteral,
    TemplateHead,
    TemplateMiddle,
    TemplateTail,
    Regex,
    Underscore,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenData {
    StringData(String),
    FloatData(f64),
    IntData(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub loc: SourceLoc,
    pub data: Option<TokenData>,
}

impl Token {
    pub fn new(loc: SourceLoc, kind: TokenKind) -> Self {
        Self {
            loc,
            kind,
            data: None,
        }
    }

    pub fn new_string(loc: SourceLoc, kind: TokenKind, data: String) -> Self {
        Self {
            loc,
            kind,
            data: Some(TokenData::StringData(data.to_string())),
        }
    }

    pub fn new_float(loc: SourceLoc, kind: TokenKind, data: f64) -> Self {
        Self {
            loc,
            kind,
            data: Some(TokenData::FloatData(data)),
        }
    }

    pub fn new_int(loc: SourceLoc, kind: TokenKind, data: i64) -> Self {
        Self {
            loc,
            kind,
            data: Some(TokenData::IntData(data)),
        }
    }

    pub fn get_string(&self) -> String {
        if let Some(TokenData::StringData(s)) = &self.data {
            return s.clone();
        }
        panic!("Trying to get_string from a non-string token");
    }

    pub fn get_float(&self) -> f64 {
        if let Some(TokenData::FloatData(f)) = &self.data {
            return *f;
        }
        panic!("Trying to get_float from a non-float token");
    }

    pub fn get_int(&self) -> i64 {
        if let Some(TokenData::IntData(i)) = &self.data {
            return *i;
        }
        panic!("Trying to get_int from a non-int token");
    }
}
