use crate::token::Token;

pub struct Lexer<I>
where
    I: Iterator<Item = char>,
{
    input: I,
    current_char: Option<char>,
    position: TokenPosition,
    tokens: Vec<Token>,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(mut input: I) -> Self {
        let current_char = input.next();

        Lexer {
            input,
            current_char,
            position: TokenPosition::new(1, 0),
            tokens: vec![],
        }
    }

    pub fn scan(&mut self) -> &Vec<Token> {
        while let Ok(token) = self.scan_token() {
            if token == Token::EOF {
                self.tokens.push(token);
                break;
            }
            self.tokens.push(token);
            self.advance();
        }

        &self.tokens
    }

    fn advance(&mut self) {
        if let Some(c) = self.current_char {
            if c == '\n' {
                self.position.advance_line();
            } else {
                self.position.advance_column();
            }
        }
        self.current_char = self.input.next();
    }

    fn scan_token(&mut self) -> Result<Token, LexerError> {
        let curr = self.current_char;

        match curr {
            Some(ch) => match ch {
                '(' => Ok(Token::LeftParen),
                ')' => Ok(Token::RightParen),
                '{' => Ok(Token::LeftBrace),
                '}' => Ok(Token::RightBrace),
                '[' => Ok(Token::LeftBracket),
                ']' => Ok(Token::RightBracket),
                ':' => Ok(Token::Colon),
                ';' => Ok(Token::Semicolon),
                ',' => Ok(Token::Comma),
                '.' => Ok(Token::Dot),

                '-' => Ok(Token::Minus),
                '+' => Ok(Token::Plus),
                '*' => Ok(Token::Asterisk),
                '/' => {
                    if self.match_next('/') {
                        self.handle_single_line_comment()
                    } else if self.match_next('*') {
                        self.handle_multi_line_comment()
                    } else {
                        Ok(Token::Slash)
                    }
                }
                '%' => Ok(Token::Percent),

                '!' => self.handle_exclamation(),
                '=' => self.handle_equal(),
                '<' => self.handle_less(),
                '>' => self.handle_greater(),

                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                    Ok(Token::Whitespace)
                }

                'a'..='z' | 'A'..='Z' => self.scan_identifier_or_keyword(),

                '0'..='9' => self.scan_number(),

                '"' => self.scan_string(),

                _ => Err(LexerError::new(
                    "Unexpected character",
                    self.position.line,
                    self.position.column,
                )),
            },
            None => Ok(Token::EOF),
        }
    }

    fn scan_identifier_or_keyword(&mut self) -> Result<Token, LexerError> {
        let mut identifier = String::new();


        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }


        match identifier.as_str() {
            "rule" => Ok(Token::Rule),
            "trigger" => Ok(Token::Trigger),
            "transform" => Ok(Token::Transform),
            "match" => Ok(Token::Match),
            "action" => Ok(Token::Action),
            "let" => Ok(Token::Let),
            "if" => Ok(Token::If),
            "else" => Ok(Token::Else),
            "fn" => Ok(Token::Fn),
            "enum" => Ok(Token::Enum),
            "for" => Ok(Token::For),
            "in" => Ok(Token::In),
            "import" => Ok(Token::Import),
            "return" => Ok(Token::Return),
            _ => Ok(Token::Identifier(identifier)),
        }
    }

    fn scan_number(&mut self) -> Result<Token, LexerError> {
        let mut number = String::new();
        let mut has_dot = false;

        while let Some(ch) = self.current_char {
            if ch.is_digit(10) {
                number.push(ch);
                self.advance();
            } else if ch == '.' {
                if has_dot {
                    return Err(LexerError::new(
                        "Unexpected character in number",
                        self.position.line,
                        self.position.column,
                    ));
                }
                has_dot = true;
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        match number.parse::<f64>() {
            Ok(num) => Ok(Token::NumberLiteral(num)),
            Err(_) => Err(LexerError::new(
                "Invalid number format",
                self.position.line,
                self.position.column,
            )),
        }
    }

    fn scan_string(&mut self) -> Result<Token, LexerError> {
        let mut string = String::new();

        self.advance();

        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance();
                return Ok(Token::StringLiteral(string));
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char {
                    match escaped {
                        'n' => string.push('\n'),
                        'r' => string.push('\r'),
                        't' => string.push('\t'),
                        '\\' => string.push('\\'),
                        '"' => string.push('"'),
                        _ => {
                            return Err(LexerError::new(
                                "Invalid escape sequence",
                                self.position.line,
                                self.position.column,
                            ))
                        }
                    }
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Err(LexerError::new(
            "Unterminated string literal",
            self.position.line,
            self.position.column,
        ))
    }

    fn handle_exclamation(&mut self) -> Result<Token, LexerError> {
        if self.match_next('=') {
            Ok(Token::NotEqual)
        } else {
            Ok(Token::Not)
        }
    }

    fn handle_equal(&mut self) -> Result<Token, LexerError> {
        if self.match_next('=') {
            Ok(Token::Equal)
        } else {
            Ok(Token::Assign)
        }
    }

    fn handle_less(&mut self) -> Result<Token, LexerError> {
        if self.match_next('=') {
            Ok(Token::LessEqual)
        } else {
            Ok(Token::Less)
        }
    }

    fn handle_greater(&mut self) -> Result<Token, LexerError> {
        if self.match_next('=') {
            Ok(Token::GreaterEqual)
        } else {
            Ok(Token::Greater)
        }
    }

    fn handle_single_line_comment(&mut self) -> Result<Token, LexerError> {
        let mut comment = String::new();

        self.advance();
        self.advance();

        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            comment.push(ch);
            self.advance();
        }

        Ok(Token::Comment(comment))
    }

    fn handle_multi_line_comment(&mut self) -> Result<Token, LexerError> {
        let mut comment = String::new();

        self.advance();
        self.advance();

        while let Some(ch) = self.current_char {
            if ch == '*' {
                if self.match_next('/') {
                    self.advance();
                    return Ok(Token::Comment(comment));
                }
            }

            if ch == '\n' {
                self.position.advance_line();
            } else {
                self.position.advance_column();
            }

            comment.push(ch);
            self.advance();
        }

        Err(LexerError::new(
            "Unterminated multi-line comment",
            self.position.line,
            self.position.column,
        ))
    }

    fn peek(&mut self) -> Option<char> {
        self.input.next()
    }

    fn match_next(&mut self, expected: char) -> bool {
        let next = self.peek();
        match next {
            Some(ch) => {
                if ch == expected {
                    self.advance();
                    return true;
                }
                false
            }
            None => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LexerError {
    message: String,
    line: usize,
    column: usize,
}

impl LexerError {
    fn new(message: &str, line: usize, column: usize) -> Self {
        LexerError {
            message: message.to_string(),
            line,
            column,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "line: {}, column: {}\n{}",
            self.line, self.column, self.message
        )
    }
}

struct TokenPosition {
    line: usize,
    column: usize,
}

impl TokenPosition {
    fn new(line: usize, column: usize) -> Self {
        TokenPosition { line, column }
    }

    fn advance_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }

    fn advance_column(&mut self) {
        self.column += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input.chars());
        lexer.scan().clone()
    }

    #[test]
    fn test_keywords() {
        let input = r#"
            rule trigger transform match action let if else fn enum for in import return
        "#;
        let tokens = lex(input);
        let expected = vec![
            Token::Rule,
            Token::Trigger,
            Token::Transform,
            Token::Match,
            Token::Action,
            Token::Let,
            Token::If,
            Token::Else,
            Token::Fn,
            Token::Enum,
            Token::For,
            Token::In,
            Token::Import,
            Token::Return,
            Token::Whitespace,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_identifiers() {
        let input = r#"
            let myVar = 42;
            let another_var = "hello";
        "#;
        let tokens = lex(input);
        let expected = vec![
            Token::Let,
            Token::Whitespace,
            Token::Identifier("myVar".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::NumberLiteral(42.0),
            Token::Semicolon,
            Token::Whitespace,
            Token::Let,
            Token::Whitespace,
            Token::Identifier("another_var".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::StringLiteral("hello".to_string()),
            Token::Semicolon,
            Token::Whitespace,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_operators_and_delimiters() {
        let input = r#"
            let result = 5 + 3 * (10 / 2) - 7;
            let isEqual = (5 + 3) == 8;
            let isNotEqual = (10 - 2) != 5;
        "#;
        let tokens = lex(input);
        let expected = vec![
            Token::Let,
            Token::Whitespace,
            Token::Identifier("result".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::NumberLiteral(5.0),
            Token::Whitespace,
            Token::Plus,
            Token::Whitespace,
            Token::NumberLiteral(3.0),
            Token::Whitespace,
            Token::Asterisk,
            Token::Whitespace,
            Token::LeftParen,
            Token::NumberLiteral(10.0),
            Token::Whitespace,
            Token::Slash,
            Token::Whitespace,
            Token::NumberLiteral(2.0),
            Token::RightParen,
            Token::Whitespace,
            Token::Minus,
            Token::Whitespace,
            Token::NumberLiteral(7.0),
            Token::Semicolon,
            Token::Whitespace,
            Token::Let,
            Token::Whitespace,
            Token::Identifier("isEqual".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::LeftParen,
            Token::NumberLiteral(5.0),
            Token::Whitespace,
            Token::Plus,
            Token::Whitespace,
            Token::NumberLiteral(3.0),
            Token::RightParen,
            Token::Whitespace,
            Token::Equal,
            Token::Whitespace,
            Token::NumberLiteral(8.0),
            Token::Semicolon,
            Token::Whitespace,
            Token::Let,
            Token::Whitespace,
            Token::Identifier("isNotEqual".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::LeftParen,
            Token::NumberLiteral(10.0),
            Token::Whitespace,
            Token::Minus,
            Token::Whitespace,
            Token::NumberLiteral(2.0),
            Token::RightParen,
            Token::Whitespace,
            Token::NotEqual,
            Token::Whitespace,
            Token::NumberLiteral(5.0),
            Token::Semicolon,
            Token::Whitespace,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_literals() {
        let input = r#"
            let name = "John Doe";
            let age = 30;
            let pi = 3.14159;
        "#;
        let tokens = lex(input);
        let expected = vec![
            Token::Let,
            Token::Whitespace,
            Token::Identifier("name".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::StringLiteral("John Doe".to_string()),
            Token::Semicolon,
            Token::Whitespace,
            Token::Let,
            Token::Whitespace,
            Token::Identifier("age".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::NumberLiteral(30.0),
            Token::Semicolon,
            Token::Whitespace,
            Token::Let,
            Token::Whitespace,
            Token::Identifier("pi".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::NumberLiteral(3.14159),
            Token::Semicolon,
            Token::Whitespace,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comments() {
        let input = r#"
            // This is a single-line comment
            let x = 10; /* This is a
            multi-line comment */
            let y = x + 5;
        "#;
        let tokens = lex(input);
        let expected = vec![
            Token::Comment(" This is a single-line comment".to_string()),
            Token::Whitespace,
            Token::Let,
            Token::Whitespace,
            Token::Identifier("x".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::NumberLiteral(10.0),
            Token::Semicolon,
            Token::Whitespace,
            Token::Comment(" This is a\n            multi-line comment ".to_string()),
            Token::Whitespace,
            Token::Let,
            Token::Whitespace,
            Token::Identifier("y".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::Identifier("x".to_string()),
            Token::Whitespace,
            Token::Plus,
            Token::Whitespace,
            Token::NumberLiteral(5.0),
            Token::Semicolon,
            Token::Whitespace,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_complex_syntax() {
        let input = r#"
        rule "Detect API Changes" {
            trigger {
                path = "backend/**/*.go"
                match = regex("func (\\w+)Handler")
            }
            transform "toApiEndpoint" {
                input = "$1"
                steps = [
                    { toLowerCase },
                    { replace: { pattern: "Handler$", with: "_endpoint" } },
                    { prepend: "/api/" }
                ]
                output = "$result"
            }
            match {
                path = "frontend/**/*.dart"
                match = regex("ApiClient.call('$transform')")
            }
            action {
                alert = Alert.Severe
            }
        }
    "#;

        let tokens = lex(input);
        let expected = vec![
            Token::Whitespace, // Initial whitespace
            Token::Rule,
            Token::Whitespace,
            Token::StringLiteral("Detect API Changes".to_string()),
            Token::Whitespace,
            Token::LeftBrace,
            Token::Whitespace,
            Token::Trigger,
            Token::Whitespace,
            Token::LeftBrace,
            Token::Whitespace,
            Token::Identifier("path".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::StringLiteral("backend/**/*.go".to_string()),
            Token::Whitespace,
            Token::Identifier("match".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::Identifier("regex".to_string()),
            Token::LeftParen,
            Token::StringLiteral(r"func (\w+)Handler".to_string()),
            Token::RightParen,
            Token::Whitespace,
            Token::RightBrace,
            Token::Whitespace,
            Token::Transform,
            Token::Whitespace,
            Token::StringLiteral("toApiEndpoint".to_string()),
            Token::Whitespace,
            Token::LeftBrace,
            Token::Whitespace,
            Token::Identifier("input".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::StringLiteral("$1".to_string()),
            Token::Whitespace,
            Token::Identifier("steps".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::LeftBracket,
            Token::Whitespace,
            Token::LeftBrace,
            Token::Identifier("toLowerCase".to_string()),
            Token::RightBrace,
            Token::Comma,
            Token::Whitespace,
            Token::LeftBrace,
            Token::Identifier("replace".to_string()),
            Token::Colon,
            Token::Whitespace,
            Token::LeftBrace,
            Token::Identifier("pattern".to_string()),
            Token::Colon,
            Token::Whitespace,
            Token::StringLiteral("Handler$".to_string()),
            Token::Comma,
            Token::Whitespace,
            Token::Identifier("with".to_string()),
            Token::Colon,
            Token::Whitespace,
            Token::StringLiteral("_endpoint".to_string()),
            Token::RightBrace,
            Token::RightBrace,
            Token::Comma,
            Token::Whitespace,
            Token::LeftBrace,
            Token::Identifier("prepend".to_string()),
            Token::Colon,
            Token::Whitespace,
            Token::StringLiteral("/api/".to_string()),
            Token::RightBrace,
            Token::Whitespace,
            Token::RightBracket,
            Token::Whitespace,
            Token::Identifier("output".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::StringLiteral("$result".to_string()),
            Token::Whitespace,
            Token::RightBrace,
            Token::Whitespace,
            Token::Match,
            Token::Whitespace,
            Token::LeftBrace,
            Token::Whitespace,
            Token::Identifier("path".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::StringLiteral("frontend/**/*.dart".to_string()),
            Token::Whitespace,
            Token::Identifier("match".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::Identifier("regex".to_string()),
            Token::LeftParen,
            Token::StringLiteral("ApiClient.call('$transform')".to_string()),
            Token::RightParen,
            Token::Whitespace,
            Token::RightBrace,
            Token::Whitespace,
            Token::Action,
            Token::Whitespace,
            Token::LeftBrace,
            Token::Whitespace,
            Token::Identifier("alert".to_string()),
            Token::Whitespace,
            Token::Assign,
            Token::Whitespace,
            Token::Identifier("Alert".to_string()),
            Token::Dot,
            Token::Identifier("Severe".to_string()),
            Token::Whitespace,
            Token::RightBrace,
            Token::Whitespace,
            Token::RightBrace,
            Token::Whitespace,
        ];

        assert_eq!(tokens, expected);
    }
}
