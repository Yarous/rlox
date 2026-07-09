use super::*;

fn scan(source: &str) -> ScanResult {
    Scanner::new(source).scan_tokens()
}

fn kinds(source: &str) -> Vec<TokenKind> {
    scan(source)
        .tokens
        .into_iter()
        .map(|token| token.token_type)
        .collect()
}

#[test]
fn scans_single_character_tokens() {
    assert_eq!(
        kinds("(){},.-+;:*?"),
        vec![
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::Comma,
            TokenKind::Dot,
            TokenKind::Minus,
            TokenKind::Plus,
            TokenKind::Semicolon,
            TokenKind::Colon,
            TokenKind::Star,
            TokenKind::Question,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn scans_one_or_two_character_tokens() {
    assert_eq!(
        kinds("! != = == > >= < <= /"),
        vec![
            TokenKind::Bang,
            TokenKind::BangEqual,
            TokenKind::Equal,
            TokenKind::EqualEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Slash,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn scans_keywords_and_identifiers() {
    assert_eq!(
        kinds(
            "and class else false fun for if nil or print return super this true var while break alpha _x x9"
        ),
        vec![
            TokenKind::And,
            TokenKind::Class,
            TokenKind::Else,
            TokenKind::False,
            TokenKind::Fun,
            TokenKind::For,
            TokenKind::If,
            TokenKind::Nil,
            TokenKind::Or,
            TokenKind::Print,
            TokenKind::Return,
            TokenKind::Super,
            TokenKind::This,
            TokenKind::True,
            TokenKind::Var,
            TokenKind::While,
            TokenKind::Break,
            TokenKind::Identifier,
            TokenKind::Identifier,
            TokenKind::Identifier,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn scans_number_literals() {
    let result = scan("123 45.67");

    assert!(result.errors.is_empty());
    assert_eq!(result.tokens[0].literal, Some(Literal::Number(123.0)));
    assert_eq!(result.tokens[1].literal, Some(Literal::Number(45.67)));
}

#[test]
fn scans_string_literals() {
    let result = scan("\"lox\" \"line\\nnext\"");

    assert!(result.errors.is_empty());
    assert_eq!(
        result.tokens[0].literal,
        Some(Literal::String("lox".to_string()))
    );
    assert_eq!(
        result.tokens[1].literal,
        Some(Literal::String("line\\nnext".to_string()))
    );
}

#[test]
fn ignores_line_comments() {
    assert_eq!(
        kinds("print 1; // ignored\nprint 2;"),
        vec![
            TokenKind::Print,
            TokenKind::Number,
            TokenKind::Semicolon,
            TokenKind::Print,
            TokenKind::Number,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn ignores_block_comments_and_tracks_lines() {
    let result = scan("print 1; /* a\nb\nc */ print 2;");

    assert!(result.errors.is_empty());
    assert_eq!(result.tokens[3].token_type, TokenKind::Print);
    assert_eq!(result.tokens[3].line, 3);
}

#[test]
fn reports_unterminated_string() {
    let result = scan("\"unterminated");

    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].message, "Unterminated string");
    assert_eq!(result.tokens.last().unwrap().token_type, TokenKind::Eof);
}

#[test]
fn reports_unterminated_block_comment() {
    let result = scan("/* nope");

    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].message, "Unterminated block comment");
}

#[test]
fn reports_unexpected_character_and_continues() {
    let result = scan("@ print 1;");

    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].message, "Failed to parse lexeme as token");
    assert_eq!(
        result
            .tokens
            .into_iter()
            .map(|token| token.token_type)
            .collect::<Vec<_>>(),
        vec![
            TokenKind::Print,
            TokenKind::Number,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}
