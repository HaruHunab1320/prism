use logos::Logos;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Single-character tokens
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    LeftBracket, RightBracket,
    Comma, Dot, Minus, Plus,
    Semicolon, Slash, Star,

    // One or two character tokens
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    Tilde, TildeGreater, TildeEqual,

    // Literals
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords
    And, Or, If, Else,
    True, False, None,
    Let, Function, Async, Await,
    Context, Verify, Against, Sources,
    Match, With, Confidence,
    Try, Below, Threshold, Uncertain,
    Medium, Low,

    // End of file
    Eof,
}

#[derive(Logos, Debug, PartialEq)]
enum LogosToken {
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Whitespace,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    String(String),

    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse())]
    Number(f64),

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token("-")]
    Minus,

    #[token("+")]
    Plus,

    #[token(";")]
    Semicolon,

    #[token("/")]
    Slash,

    #[token("*")]
    Star,

    #[token("!")]
    Bang,

    #[token("!=")]
    BangEqual,

    #[token("=")]
    Equal,

    #[token("==")]
    EqualEqual,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEqual,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEqual,

    #[token("~")]
    Tilde,

    #[token("~>")]
    TildeGreater,

    #[token("~=")]
    TildeEqual,

    #[token("and")]
    And,

    #[token("or")]
    Or,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("none")]
    None,

    #[token("let")]
    Let,

    #[token("fn")]
    Function,

    #[token("async")]
    Async,

    #[token("await")]
    Await,

    #[token("context")]
    Context,

    #[token("verify")]
    Verify,

    #[token("against")]
    Against,

    #[token("sources")]
    Sources,

    #[token("match")]
    Match,

    #[token("with")]
    With,

    #[token("confidence")]
    Confidence,

    #[token("try")]
    Try,

    #[token("below")]
    Below,

    #[token("threshold")]
    Threshold,

    #[token("uncertain")]
    Uncertain,

    #[token("medium")]
    Medium,

    #[token("low")]
    Low,

    #[error]
    Error,
}

pub struct Lexer<'a> {
    source: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn lex(&mut self) -> Result<(Vec<Token>, Vec<usize>, Vec<usize>), String> {
        let mut tokens = Vec::new();
        let mut starts = Vec::new();
        let mut ends = Vec::new();

        let mut lex = LogosToken::lexer(self.source);
        while let Some(token_result) = lex.next() {
            match token_result {
                LogosToken::Error => {
                    return Err(format!("Invalid token at position {}", lex.span().start));
                }
                LogosToken::Whitespace => continue,
                token => {
                    starts.push(lex.span().start);
                    ends.push(lex.span().end);
                    tokens.push(match token {
                        LogosToken::Identifier(s) => Token::Identifier(s),
                        LogosToken::String(s) => Token::String(s),
                        LogosToken::Number(n) => Token::Number(n),
                        LogosToken::LeftParen => Token::LeftParen,
                        LogosToken::RightParen => Token::RightParen,
                        LogosToken::LeftBrace => Token::LeftBrace,
                        LogosToken::RightBrace => Token::RightBrace,
                        LogosToken::LeftBracket => Token::LeftBracket,
                        LogosToken::RightBracket => Token::RightBracket,
                        LogosToken::Comma => Token::Comma,
                        LogosToken::Dot => Token::Dot,
                        LogosToken::Minus => Token::Minus,
                        LogosToken::Plus => Token::Plus,
                        LogosToken::Semicolon => Token::Semicolon,
                        LogosToken::Slash => Token::Slash,
                        LogosToken::Star => Token::Star,
                        LogosToken::Bang => Token::Bang,
                        LogosToken::BangEqual => Token::BangEqual,
                        LogosToken::Equal => Token::Equal,
                        LogosToken::EqualEqual => Token::EqualEqual,
                        LogosToken::Greater => Token::Greater,
                        LogosToken::GreaterEqual => Token::GreaterEqual,
                        LogosToken::Less => Token::Less,
                        LogosToken::LessEqual => Token::LessEqual,
                        LogosToken::Tilde => Token::Tilde,
                        LogosToken::TildeGreater => Token::TildeGreater,
                        LogosToken::TildeEqual => Token::TildeEqual,
                        LogosToken::And => Token::And,
                        LogosToken::Or => Token::Or,
                        LogosToken::If => Token::If,
                        LogosToken::Else => Token::Else,
                        LogosToken::True => Token::True,
                        LogosToken::False => Token::False,
                        LogosToken::None => Token::None,
                        LogosToken::Let => Token::Let,
                        LogosToken::Function => Token::Function,
                        LogosToken::Async => Token::Async,
                        LogosToken::Await => Token::Await,
                        LogosToken::Context => Token::Context,
                        LogosToken::Verify => Token::Verify,
                        LogosToken::Against => Token::Against,
                        LogosToken::Sources => Token::Sources,
                        LogosToken::Match => Token::Match,
                        LogosToken::With => Token::With,
                        LogosToken::Confidence => Token::Confidence,
                        LogosToken::Try => Token::Try,
                        LogosToken::Below => Token::Below,
                        LogosToken::Threshold => Token::Threshold,
                        LogosToken::Uncertain => Token::Uncertain,
                        LogosToken::Medium => Token::Medium,
                        LogosToken::Low => Token::Low,
                        LogosToken::Error | LogosToken::Whitespace => unreachable!(),
                    });
                }
            }
        }

        tokens.push(Token::Eof);
        starts.push(self.source.len());
        ends.push(self.source.len());

        Ok((tokens, starts, ends))
    }
} 