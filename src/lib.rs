use logos::Logos;
use std::collections::HashMap;

/// All meaningful CSS tokens
#[derive(Logos, Debug)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
    #[regex(r"\[[^\]]+\]", |lex| lex.slice().to_string())]
    AttributeSelector(String),

    #[token("+")]
    AdjacentSiblingCombinator,

    #[regex(r"\.[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    ClassSelector(String),

    #[token("~")]
    GeneralSiblingCombinator,

    #[regex(r"#[0-9a-fA-F]{3}([0-9a-fA-F]{3})?", |lex| lex.slice().to_string())]
    HexColor(String),

    #[regex(r"#[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    IdSelector(String),

    #[regex(r"!important", |lex| lex.slice().to_string())]
    Important(String),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string(), priority = 1)]
    ElementSelector(String),

    #[regex(r":[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    PseudoClass(String),

    #[regex(r"::[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    PseudoElement(String),

    #[regex(r"[a-zA-Z-]+\s*:", |lex| lex.slice().trim_end_matches(':').to_string(), priority = 2)]
    Property(String),

    #[regex(r"[0-9]+(\.[0-9]+)?(px|em|rem|%)?", |lex| lex.slice().to_string(), priority = 4)]
    NumericValue(String),

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

    #[regex(r#""[^"]*""#, |lex| lex.slice().to_string())]
    #[regex(r#"'[^']*'"#, |lex| lex.slice().to_string())]
    StringValue(String),

    #[regex(r"[a-zA-Z0-9#%.\-]+", |lex| lex.slice().to_string(), priority = 3)]
    Value(String),

    #[regex(r"[a-zA-Z-]+\([^)]*\)", |lex| lex.slice().to_string())]
    Function(String),
}