use logos::{ Logos, Span };
use core::fmt;

use std::ops::Range;

pub type Error = (String, Span);

/// All meaningful CSS tokens
#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token<'s> {
    #[regex(r"[_a-zA-Z][-_a-zA-Z0-9]*")] Ident(&'s str),

    #[regex(r"@[_a-zA-Z][-_a-zA-Z0-9]*", |lex| lex.slice().strip_prefix('@').unwrap())] AtKeyword(
        &'s str,
    ),

    #[regex(r"#[_a-zA-Z][-_a-zA-Z0-9]*", |lex| lex.slice().strip_prefix('#').unwrap())] Hash(
        &'s str,
    ),

    #[regex(r#"["']([^"'\\]|\\.)*["']"#, |lex| {
        let slice = lex.slice();
        if slice.starts_with('"') {
            slice.strip_prefix('"').unwrap().strip_suffix('"').unwrap()
        } else {
            slice.strip_prefix('\'').unwrap().strip_suffix('\'').unwrap()
        }
    })] QuotedString(&'s str),

    #[regex(r"url\(([^)]+)\)", |lex|
        lex.slice().strip_prefix("url(").unwrap().strip_suffix(")").unwrap()
    )] UnquotedUrl(&'s str),

    #[regex(r"[!-/:-@\[-\^_`{-~]", priority = 3)] Delim(&'s str),

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

    #[regex(r"[_a-zA-Z][-_a-zA-Z0-9]*\(", |lex| lex.slice().strip_suffix('(').unwrap())] Function(
        &'s str,
    ),

    #[regex(r"\([^)]*\)")] ParenthesisBlock(&'s str),

    #[regex(r"\[[^\]]*\]")] SquareBracketBlock(&'s str),

    #[regex(r"\{[^}]*\}")] CurlyBracketBlock(&'s str),

    #[regex(r"url\(\s*\)")] BadUrl(&'s str),

    #[regex(r#""([^"\\]|\\.)*"#)] BadString(&'s str),

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
    while let Some(token) = lexer.next() {}

    /*     let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
    let minified = minify(parser.lock().unwrap().clone());
    // write to file
    std::fs::write("minified.css", minified).unwrap(); */
    /* let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed); */
    //println!("{}", parser);
}

use logos::source::Source;

#[allow(clippy::type_complexity)]
pub fn assert_lex<'a, Token>(
    source: &'a Token::Source,
    tokens: &[(Result<Token, Token::Error>, <Token::Source as Source>::Slice<'a>, Range<usize>)]
)
    where Token: Logos<'a> + fmt::Debug + PartialEq, Token::Extras: Default
{
    let mut lex = Token::lexer(source);

    for tuple in tokens {
        assert_eq!(&(lex.next().expect("Unexpected end"), lex.slice(), lex.span()), tuple);
    }

    assert_eq!(lex.next(), None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ident() {
        let css = "container123 _main-container";
        let expected = &[
            (Ok(Token::Ident("container123")), "container123", 0..12),
            (Ok(Token::Ident("_main-container")), "_main-container", 13..28),
        ];
        assert_lex(css, expected);
    }

    #[test]
    fn test_at_keyword() {
        let css = "@media @import";
        let expected = &[
            (Ok(Token::AtKeyword("media")), "@media", 0..6),
            (Ok(Token::AtKeyword("import")), "@import", 7..14),
        ];
        assert_lex(css, expected);
    }

    #[test]
    fn test_hash() {
        let css = "#header #footer123";
        let expected = &[
            (Ok(Token::Hash("header")), "#header", 0..7),
            (Ok(Token::Hash("footer123")), "#footer123", 8..18),
        ];
        assert_lex(css, expected);
    }

    #[test]
    fn test_quoted_string() {
        let css = r#""Helvetica Neue" 'Arial'"#;
        let expected = &[
            (Ok(Token::QuotedString("Helvetica Neue")), r#""Helvetica Neue""#, 0..16),
            (Ok(Token::QuotedString("Arial")), r#"'Arial'"#, 17..24),
        ];
        assert_lex(css, expected);
    }

    #[test]
    fn test_unquoted_url() {
        let css = r#"url(image.png) url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA)"#;
        let expected = &[
            (Ok(Token::UnquotedUrl("image.png")), r#"url(image.png)"#, 0..14),
            (
                Ok(Token::UnquotedUrl("\"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA")),
                r#"url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA)"#,
                15..67,
            ),
        ];
        assert_lex(css, expected);
    }

    #[test]
    fn test_delim() {
        let css = "{ } ( ) [ ] : ; , > + ~ . ! @ # $ % ^ & * | \\ / _ ` {";
        let expected = &[
            (Ok(Token::CurlyBracketBlock("{ }")), "{ }", 0..3),
            (Ok(Token::ParenthesisBlock("( )")), "( )", 4..7),
            (Ok(Token::SquareBracketBlock("[ ]")), "[ ]", 8..11),
            (Ok(Token::Delim(":")), ":", 12..13),
            (Ok(Token::Delim(";")), ";", 14..15),
            (Ok(Token::Delim(",")), ",", 16..17),
            (Ok(Token::Delim(">")), ">", 18..19),
            (Ok(Token::Delim("+")), "+", 20..21),
            (Ok(Token::Delim("~")), "~", 22..23),
            (Ok(Token::Delim(".")), ".", 24..25),
            (Ok(Token::Delim("!")), "!", 26..27),
            (Ok(Token::Delim("@")), "@", 28..29),
            (Ok(Token::Delim("#")), "#", 30..31),
            (Ok(Token::Delim("$")), "$", 32..33),
            (Ok(Token::Delim("%")), "%", 34..35),
            (Ok(Token::Delim("^")), "^", 36..37),
            (Ok(Token::Delim("&")), "&", 38..39),
            (Ok(Token::Delim("*")), "*", 40..41),
            (Ok(Token::Delim("|")), "|", 42..43),
            (Ok(Token::Delim("\\")), "\\", 44..45),
            (Ok(Token::Delim("/")), "/", 46..47),
            (Ok(Token::Delim("_")), "_", 48..49),
            (Ok(Token::Delim("`")), "`", 50..51),
            (Ok(Token::Delim("{")), "{", 52..53),
        ];
        assert_lex(css, expected);
    }

    #[test]
    fn test_number_percentage_dimension() {
        let css = "10 -5 +3.14 50% 1.5em";
        let expected = &[
            (Ok(Token::Number("10")), "10", 0..2),
            (Ok(Token::Number("-5")), "-5", 3..5),
            (Ok(Token::Number("+3.14")), "+3.14", 6..11),
            (Ok(Token::Percentage("50%")), "50%", 12..15),
            (Ok(Token::Dimension("1.5em")), "1.5em", 16..21),
        ];
        assert_lex(css, expected);
    }

    /* #[test]
    fn test_comment() {
        let css = "/* This is a comment */";
        let expected = &[
            (Ok(Token::Comment("/* This is a comment */")), "/* This is a comment */", 0..23),
        ];
        assert_lex(css, expected);
    } */
}
