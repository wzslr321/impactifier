#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Rule,
    Trigger,
    Transform,
    Match,
    Action,
    Let,
    If,
    Else,
    Fn,
    Enum,
    For,
    In,
    Import,
    Return,

    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f64),

    // Operators
    Assign,       // =
    Plus,         // +
    Minus,        // -
    Asterisk,     // *
    Slash,        // /
    Percent,      // %
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    And,          // &&
    Or,           // ||
    Not,          // !

    // Delimiters
    LeftBrace,    // {
    RightBrace,   // }
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    Colon,        // :
    Semicolon,    // ;
    Comma,        // ,
    Dot,          // .

    // Whitespace and Comments
    Whitespace,
    Comment(String), // //

    // End of input
    EOF,

    Illegal,
}
