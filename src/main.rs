use core::fmt;
use std::{ collections::HashMap, fmt::{ Display, Formatter }, fs::File, io::BufWriter };
use std::sync::{ Arc, Mutex, mpsc };
use std::thread;
use std::io::Write;

use cssparser_rs::Token;
use logos::{ Lexer, Logos };

use rand::{ thread_rng, Rng };
use rand::distributions::Alphanumeric;

use indexmap::IndexMap;
use std::mem;
use std::ops::Range;

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
    selector: Vec<u8>,
    properties: Vec<(Vec<u8>, Vec<u8>)>,
    current_property: Vec<u8>,
    current_value: Vec<u8>,
}

impl SelectorBuffer {
    fn new() -> Self {
        Self {
            selector: Vec::with_capacity(256), // Most selectors < 256 bytes
            properties: Vec::with_capacity(32), // Most rules < 32 properties
            current_property: Vec::with_capacity(64), // Properties rarely > 64 bytes
            current_value: Vec::with_capacity(128), // Values rarely > 128 bytes
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

fn main() {
    // Read the css file
    let css = include_str!("../bootstrap-4.css");
    let mut parser = Parser::new();
    let mut buffer = SelectorBuffer::new();

    let start = std::time::Instant::now();

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
                    buffer.selector.extend_from_slice(b",");
                } else {
                    buffer.current_value.extend_from_slice(b",");
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
    let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
    //println!("{}", parser);
}

/* fn minify(parser: Parser) -> String {
    let mut property_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut minified = String::new();

    // Collect selectors by their properties
    for (selector, properties) in &parser.selectors {
        let mut property_string = String::new();
        for (property, values) in &properties.properties {
            property_string.push_str(&property);
            property_string.push(':');
            property_string.push_str(&values.join(" "));
            property_string.push(';');
        }

        property_map.entry(property_string).or_insert_with(Vec::new).push(selector.clone());
    }

    // Write the grouped selectors
    for (properties, selectors) in &property_map {
        minified.push_str(&selectors.join(","));
        minified.push('{');
        minified.push_str(properties);
        minified.push('}');
    }

    minified
} */
