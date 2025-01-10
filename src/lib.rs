use logos::{ Logos, Span };
use core::fmt;
use std::fmt::{ Display, Formatter };
use indexmap::IndexMap;
use std::mem;
use std::ops::Range;
use smallvec::SmallVec;

pub type Error = (String, Span);

pub type Result<T> = std::result::Result<T, Error>;

/// All meaningful CSS tokens
#[derive(Logos, Debug)]
#[logos(skip r"[\t\r\n\f]+")]
pub enum Token<'s> {
    #[regex(r"[_a-zA-Z][-_a-zA-Z0-9]*")] Ident(&'s str),

    #[regex(r"@[_a-zA-Z][-_a-zA-Z0-9]*")] AtKeyword(&'s str),

    #[regex(r"#[_a-zA-Z][-_a-zA-Z0-9]*")] Hash(&'s str),

    #[regex(r#""([^"\\]|\\.)*""#)] QuotedString(&'s str),

    #[regex(r"url\(([^)]+)\)")] UnquotedUrl(&'s str),

    #[regex(r"[!-/\-@\[\\`{-~]", priority = 3)] Delim(&'s str),

    #[regex(r"[+-]?\d+(\.\d+)?")] Number(&'s str),

    #[regex(r"[+-]?\d+(\.\d+)?%")] Percentage(&'s str),

    #[regex(r"[+-]?\d+(\.\d+)?[a-zA-Z]+")] Dimension(&'s str),

    #[regex(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")] Comment(&'s str),

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[token("~=")]
    IncludeMatch,

    #[token("|=")]
    DashMatch,

    #[token("^=")]
    PrefixMatch,

    #[token("$=")]
    SuffixMatch,

    #[token("*=")]
    SubstringMatch,

    #[token("<!--")]
    CDO,

    #[token("-->")]
    CDC,

    #[regex(r"[_a-zA-Z][-_a-zA-Z0-9]*\(")] Function(&'s str),

    #[regex(r"\([^)]*\)")]
    ParenthesisBlock(&'s str),

    #[regex(r"\[[^\]]*\]")]
    SquareBracketBlock(&'s str),

    #[regex(r"\{[^}]*\}")]
    CurlyBracketBlock(&'s str),

    #[regex(r"url\(\s*\)")]
    BadUrl(&'s str),

    #[regex(r#""([^"\\]|\\.)*"#)]
    BadString(&'s str),

    #[token(")")]
    CloseParenthesis,

    #[token("]")]
    CloseSquareBracket,

    #[token("}")]
    CloseCurlyBracket,
}

pub fn parse_css(css: &str) {
    //let start = std::time::Instant::now();

    let mut lexer = Token::lexer(css);
    while let Some(token) = lexer.next() {
        match token {
            _ => {} // Ignore unexpected tokens
        }
    }

    /*     let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
    let minified = minify(parser.lock().unwrap().clone());
    // write to file
    std::fs::write("minified.css", minified).unwrap(); */
    /* let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed); */
    //println!("{}", parser);
}
