use logos::{Lexer, Logos, Span};
use core::fmt;
use std::{ collections::HashMap, fmt::{ Display, Formatter }, fs::File, io::BufWriter };
use std::sync::{ Arc, Mutex, mpsc };
use std::thread;
use std::io::Write;

use indexmap::IndexMap;
use std::mem;
use std::ops::Range;
use smallvec::SmallVec;

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


#[derive(Clone)]
struct Parser {
    data: Vec<u8>, // Shared buffer for all strings
    selectors: IndexMap<Range<usize>, Properties>, // Ranges into data buffer
}

#[derive(Debug, Clone)]
struct Properties {
    properties: IndexMap<Range<usize>, Range<usize>>, // Property -> Value ranges
}

impl Parser {
    fn new() -> Self {
        Self {
            data: Vec::with_capacity(1024 * 64), // Pre-allocate 64KB
            selectors: IndexMap::new(),
        }
    }

    fn store_bytes(&mut self, bytes: &[u8]) -> Range<usize> {
        let start = self.data.len();
        self.data.extend_from_slice(bytes);
        start..self.data.len()
    }

    fn commit_buffer(&mut self, buffer: &SelectorBuffer) {
        if !buffer.selector.is_empty() {
            let selector_range = self.store_bytes(&buffer.selector);
            let mut props = Properties::new();

            // Store all properties and values in shared buffer
            for (prop, val) in &buffer.properties {
                let prop_range = self.store_bytes(prop);
                let val_range = self.store_bytes(val);
                props.properties.insert(prop_range, val_range);
            }

            self.selectors.insert(selector_range, props);
        }
    }
}

impl Properties {
    fn new() -> Self {
        Self {
            properties: IndexMap::new(),
        }
    }
}

#[derive(Default)]
struct SelectorBuffer {
    selector: SmallVec<[u8; 256]>,
    properties: SmallVec<[(SmallVec<[u8; 64]>, SmallVec<[u8; 128]>); 32]>,
    current_property: SmallVec<[u8; 64]>,
    current_value: SmallVec<[u8; 128]>,
}

impl SelectorBuffer {
    fn new() -> Self {
        Self {
            selector: SmallVec::new(),
            properties: SmallVec::new(),
            current_property: SmallVec::new(),
            current_value: SmallVec::new(),
        }
    }

    fn commit_property(&mut self) {
        if !self.current_property.is_empty() && !self.current_value.is_empty() {
            self.properties.push((
                mem::take(&mut self.current_property),
                mem::take(&mut self.current_value),
            ));
        }
    }
}

impl Display for Parser {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (selector, properties) in &self.selectors {
            let selector = String::from_utf8_lossy(&self.data[selector.clone()]);
            writeln!(f, "{}", selector)?;

            for (property, value) in &properties.properties {
                let property = String::from_utf8_lossy(&self.data[property.clone()]);
                let value = String::from_utf8_lossy(&self.data[value.clone()]);
                writeln!(f, "  {}: {};", property, value)?;
            }
        }

        Ok(())
    }
}

pub fn parse_css(css: &str) {
    let mut parser = Parser::new();
    let mut buffer = SelectorBuffer::new();

    //let start = std::time::Instant::now();

    let mut lexer = Token::lexer(css);
    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Value(value)) => {
                if buffer.selector.is_empty() {
                    buffer.selector.extend_from_slice(value.as_bytes());
                } else if !buffer.current_property.is_empty() {
                    buffer.current_value.extend_from_slice(value.as_bytes());
                } else {
                    buffer.selector.extend_from_slice(value.as_bytes());
                }
            }
            | Ok(Token::PseudoClass(pseudo))
            | Ok(Token::PseudoElement(pseudo))
            | Ok(Token::IdSelector(pseudo))
            | Ok(Token::ClassSelector(pseudo))
            | Ok(Token::AttributeSelector(pseudo)) => {
                buffer.selector.extend_from_slice(pseudo.as_bytes());
            }
            Ok(Token::Property(property)) => {
                buffer.commit_property();
                buffer.current_property.extend_from_slice(property.as_bytes());
            }
            Ok(Token::OpenBrace) => {
                // Continue collecting properties
            }
            Ok(Token::CloseBrace) => {
                buffer.commit_property();
                parser.commit_buffer(&buffer);
                buffer = SelectorBuffer::new();
            }
            Ok(Token::Semicolon) => {
                buffer.commit_property();
            }
            | Ok(Token::NumericValue(value))
            | Ok(Token::StringValue(value))
            | Ok(Token::HexColor(value))
            | Ok(Token::Function(value)) => {
                buffer.current_value.extend_from_slice(value.as_bytes());
            }
            Ok(Token::ChildCombinator) => {
                buffer.selector.extend_from_slice(b" > ");
            }
            Ok(Token::Comment) => {
                // ignore comments
            }
            Ok(Token::Comma) => {
                if buffer.current_property.is_empty() {
                    buffer.selector.push(b',');
                } else {
                    buffer.current_value.push(b',');
                }
            }
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

