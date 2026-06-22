use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Ключевые слова
    #[token("fn")]       Fn,
    #[token("let")]      Let,
    #[token("if")]       If,
    #[token("else")]     Else,
    #[token("while")]    While,
    #[token("return")]   Return,
    #[token("input")]    Input,
    #[token("and")]      And,
    #[token("or")]       Or,
    #[token("not")]      Not,
    #[token("break")]    Break,
    #[token("continue")] Continue,

    // Символы
    #[token(".")]      Dot,
    #[token(",")]      Comma,
    #[token("(")]      LParen,
    #[token(r#")"#)]   RParen,
    #[token("{")]      LBrace,
    #[token("}")]      RBrace,
    #[token("=")]      Equals,
    #[token("==")]     EqualEqual,
    #[token("!=")]     NotEqual,
    #[token("<")]      Less,
    #[token(">")]      Greater,
    #[token("<=")]     LessEqual,
    #[token(">=")]     GreaterEqual,

    // Арифметика
    #[token("+")]      Plus,
    #[token("-")]      Minus,
    #[token("*")]      Star,
    #[token("/")]      Slash,

    // io
    #[token("io")]     Io,
    #[token("print")]  Print,

    // Литералы
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLiteral(String),

    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Number(i32),

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex(r"##[^\n]*", logos::skip)]
    Comment,

    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    #[error]
    Error,
}
