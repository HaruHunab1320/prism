use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[error]
    Error,

    #[regex(r"[0-9]+(\.[0-9]+)?")]
    Number,

    #[regex(r#""[^"]*""#)]
    String,

    #[regex(r"true|false")]
    Boolean,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("tensor")]
    Tensor,

    #[token("~>")]
    ConfidenceFlow,

    #[token("~")]
    ConfidenceAssign,

    #[token("~=")]
    SemanticMatch,

    #[token("in")]
    In,

    #[token("verify")]
    Verify,

    #[token("context")]
    Context,

    #[token("transition")]
    Transition,

    #[token("to")]
    To,

    #[token("with")]
    With,

    #[token("match")]
    Match,

    #[token("=>")]
    Arrow,

    #[token("against")]
    Against,

    #[token("sources")]
    Sources,

    #[token("uncertain")]
    Uncertain,

    #[token("medium")]
    Medium,

    #[token("low")]
    Low,

    #[token("try")]
    Try,

    #[token("confidence")]
    Confidence,

    #[token("below")]
    Below,

    #[token("threshold")]
    Threshold,

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

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[token("=")]
    Equal,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("!")]
    Bang,

    #[token("!=")]
    BangEqual,

    #[token("==")]
    EqualEqual,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEqual,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEqual,

    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("fn")]
    Function,

    #[token("return")]
    Return,

    #[token("let")]
    Let,

    #[token("const")]
    Const,

    #[token("async")]
    Async,

    #[token("await")]
    Await,
}

pub struct Lexer<'a> {
    lexer: logos::Lexer<'a, Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Token::lexer(input),
        }
    }

    pub fn lex(&mut self) -> Result<(Vec<Token>, Vec<usize>, Vec<usize>), Box<dyn std::error::Error>> {
        let mut tokens = Vec::new();
        let mut starts = Vec::new();
        let mut ends = Vec::new();

        while let Some(token_result) = self.lexer.next() {
            match token_result {
                Token::Error => return Err("Invalid token".into()),
                token => {
                    starts.push(self.lexer.span().start);
                    ends.push(self.lexer.span().end);
                    tokens.push(token);
                }
            }
        }

        Ok((tokens, starts, ends))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next()
    }
} 