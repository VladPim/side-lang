use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Ключевые слова
    #[token("fn")]     Fn,
    #[token("let")]    Let,
    #[token("if")]     If,
    #[token("else")]   Else,

    // Символы
    #[token(".")]      Dot,
    #[token("(")]      LParen,
    #[token(")")]      RParen,
    #[token("{")]      LBrace,
    #[token("}")]      RBrace,
    #[token("=")]      Equals,         // присваивание
    #[token("==")]     EqualEqual,     // сравнения
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

    // Идентификатор io.print (io – это отдельный токен? Сделаем io)
    #[token("io")]     Io,
    #[token("print")]  Print,

    // Литералы
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLiteral(String),

    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Number(i32),

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Комментарии ## до конца строки
    #[regex(r"##[^\n]*", logos::skip)]
    Comment,

    // Пропуск пробелов
    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    #[error]
    Error,
}