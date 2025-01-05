use logos::{Logos, Span};

pub type Error = (String, Span);

pub type Result<T> = std::result::Result<T, Error>;

/// All meaningful CSS tokens
#[derive(Logos, Debug)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token<'s> {
    #[regex(r"\[[^\]]+\]", |lex| lex.slice())]
    AttributeSelector(&'s str),

    #[token("+")]
    AdjacentSiblingCombinator,

    #[regex(r"\.[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice())]
    ClassSelector(&'s str),

    #[token("~")]
    GeneralSiblingCombinator,

    #[regex(r"#[0-9a-fA-F]{3}([0-9a-fA-F]{3})?", |lex| lex.slice())]
    HexColor(&'s str),

    #[regex(r"#[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice())]
    IdSelector(&'s str),

    #[regex(r"!important", |lex| lex.slice())]
    Important(&'s str),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*\s*", |lex| lex.slice().trim(), priority = 2)]
    Value(&'s str),

    #[regex(r":[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice())]
    PseudoClass(&'s str),

    #[regex(r"::[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice())]
    PseudoElement(&'s str),

    #[regex(r"[a-zA-Z-]+\s*:", |lex| lex.slice().trim_end_matches(':'))]
    Property(&'s str),

    #[regex(r"[0-9]+(\.[0-9]+)?(px|em|rem|%)?", |lex| lex.slice())]
    NumericValue(&'s str),

    #[token("{")]
    OpenBrace,

    #[token(">")]
    ChildCombinator,

    #[token("}")]
    CloseBrace,

    #[regex(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/", logos::skip)]
    Comment,

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[regex(r#""[^"]*""#, |lex| lex.slice())]
    #[regex(r#"'[^']*'"#, |lex| lex.slice())]
    StringValue(&'s str),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*\([^)]*\)", |lex| lex.slice())]
    Function(&'s str),
}