#![allow(dead_code)]
#![allow(unused_imports)]
use logos::{ Logos, Lexer };

use std::collections::HashMap;

/// All meaningful CSS tokens
#[derive(Logos, Debug)]
//#[logos(skip r"[ \t\r\n\f]+")]
enum Token {
    #[regex(r"\.[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())] 
    ClassSelector(String),

    #[token("{")]
    OpenBrace,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    ElementSelector(String),

    #[token("}")]
    CloseBrace,

    #[regex(r"#[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())] 
    IdSelector(String),

    #[regex(r":[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())] 
    PseudoClass(String),

    #[regex(r"::[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())] 
    PseudoElement(String),

    #[regex(r"\[[^\]]+\]", |lex| lex.slice().to_string())] 
    AttributeSelector(String),

    #[token(">")]
    ChildCombinator,

    #[token("+")]
    AdjacentSiblingCombinator,

    #[token("~")]
    GeneralSiblingCombinator,

    #[token(" ")]
    DescendantCombinator,

    #[regex(r"[a-zA-Z-]+", |lex| lex.slice().to_string(), priority = 3)] 
    Property(String),

    #[regex(r"[^;{}]+", |lex| lex.slice().to_string(), priority = 1)] 
    Value(String),

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[regex(r#""[^"]*""#, |lex| lex.slice().to_string())]
    #[regex(r#"'[^']*'"#, |lex| lex.slice().to_string())]
    StringValue(String),

    #[regex(r"[0-9]+(\.[0-9]+)?(px|em|rem|%)?", |lex| lex.slice().to_string())]
    NumericValue(String),

    #[regex(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/", logos::skip)]
    Comment,

    #[regex(r"[a-zA-Z-]+\([^)]*\)", |lex| lex.slice().to_string())]
    Function(String),
}
