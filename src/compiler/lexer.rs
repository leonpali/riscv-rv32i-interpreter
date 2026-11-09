use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // keywords
    Class,
    Int,
    Void,
    If,
    Else,
    While,
    Func,
    Return,

    // names and literals
    ClassName(String), // [A-Z][a-zA-Z0-9\-_]*
    VarName(String),   // [a-z][a-zA-Z0-9\-_]*
    IntNum(i32),       // [0-9]+

    // ARITH_OPERATOR
    Plus,
    Minus,
    Mult,
    Div,

    // COND_OPERATOR
    Eq,
    Ne,
    Lt,
    Gt,
    Ge,
    Le,

    // EXTRA_COND
    And,
    Or,

    // punctuation
    Assign,
    Semicolon,
    Comma,
    LParen,
    RParen,
    LBrace,
    RBrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexError {
    UnexpectedChar(char, Position),
    IntegerTooLarge(Position),
}

const KEYWORDS: [(&str, TokenKind); 8] = [
    ("class", TokenKind::Class),
    ("int", TokenKind::Int),
    ("void", TokenKind::Void),
    ("if", TokenKind::If),
    ("else", TokenKind::Else),
    ("while", TokenKind::While),
    ("func", TokenKind::Func),
    ("return", TokenKind::Return),
];

pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer {
        chars: source.chars().peekable(),
        position: Position { line: 1, column: 1 },
    };
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token()? {
        tokens.push(token);
    }
    Ok(tokens)
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    position: Position,
}

impl Lexer<'_> {
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        if c == '\n' {
            self.position.line += 1;
            self.position.column = 1;
        } else {
            self.position.column += 1;
        }
        Some(c)
    }

    fn bump_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        self.skip_whitespace();
        let position = self.position;
        let Some(c) = self.bump() else {
            return Ok(None);
        };

        let kind = match c {
            'A'..='Z' => TokenKind::ClassName(self.name(c)),
            'a'..='z' => {
                let name = self.name(c);
                match KEYWORDS.iter().find(|(keyword, _)| *keyword == name) {
                    Some((_, keyword)) => keyword.clone(),
                    None => TokenKind::VarName(name),
                }
            }
            '0'..='9' => TokenKind::IntNum(self.int_num(c, position)?),

            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Mult,
            '/' => TokenKind::Div,

            '=' if self.bump_if('=') => TokenKind::Eq,
            '=' => TokenKind::Assign,
            '!' if self.bump_if('=') => TokenKind::Ne,
            '<' if self.bump_if('=') => TokenKind::Le,
            '<' => TokenKind::Lt,
            '>' if self.bump_if('=') => TokenKind::Ge,
            '>' => TokenKind::Gt,

            '&' if self.bump_if('&') => TokenKind::And,
            '|' if self.bump_if('|') => TokenKind::Or,

            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,

            _ => return Err(LexError::UnexpectedChar(c, position)),
        };

        Ok(Some(Token { kind, position }))
    }

    fn name(&mut self, first: char) -> String {
        let mut name = String::from(first);
        while let Some(c) = self.peek().filter(|&c| is_name_char(c)) {
            name.push(c);
            self.bump();
        }
        name
    }

    fn int_num(&mut self, first: char, position: Position) -> Result<i32, LexError> {
        let mut value = Some(first as i32 - '0' as i32);
        while let Some(digit) = self.peek().and_then(|c| c.to_digit(10)) {
            value = value
                .and_then(|v| v.checked_mul(10))
                .and_then(|v| v.checked_add(digit as i32));
            self.bump();
        }
        value.ok_or(LexError::IntegerTooLarge(position))
    }
}
