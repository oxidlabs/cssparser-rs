#![allow(unused)]

use core::fmt;
use std::{ collections::HashMap, fmt::{ Display, Formatter } };

use cssparser_rs::Token;
use logos::Logos;

struct Parser {
    selectors: HashMap<String, Properties>,
}

#[derive(Debug)]
struct Properties {
    properties: HashMap<String, Values>,
}

#[derive(Debug)]
struct Values(Vec<String>);

impl Parser {
    fn new() -> Self {
        Self {
            selectors: HashMap::new(),
        }
    }

    fn create_selector(&mut self, selector: String) {
        self.selectors.insert(selector, Properties::new());
    }

    fn add_property(&mut self, selector: String, property: String, value: String) {
        if self.selectors.contains_key(&selector) {
            self.selectors.get_mut(&selector).unwrap().insert(property, value);
        }
    }

    fn update_property(&mut self, selector: String, property: String, value: String) {
        if self.selectors.contains_key(&selector) {
            self.selectors.get_mut(&selector).unwrap().insert(property, value);
        }
    }
}

impl Properties {
    fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    fn insert(&mut self, property: String, value: String) {
        if self.properties.contains_key(&property) {
            self.properties.get_mut(&property).unwrap().0.push(value);
        } else {
            self.properties.insert(property, Values(vec![value]));
        }
    }

    fn get(&self, property: &str) -> Option<&Values> {
        self.properties.get(property)
    }

    fn get_mut(&mut self, property: &str) -> Option<&mut Values> {
        self.properties.get_mut(property)
    }

    fn remove(&mut self, property: &str) -> Option<Values> {
        self.properties.remove(property)
    }
}

impl Display for Parser {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (selector, properties) in &self.selectors {
            writeln!(f, "{} {{", selector)?;

            for (property, values) in &properties.properties {
                writeln!(f, "  {}: {};", property, values.0.join(" "))?;
            }

            writeln!(f, "}}")?;
        }

        Ok(())
    }
}

fn main() {
    let css =
        r#"
        .container {
            border: 1px solid;
            position: relative;
            width: 100px;
            height: 100px;
            display: inline-block;
            margin-right: 5px;
        }
    "#;

    let start = std::time::Instant::now();
    let mut lexer = Token::lexer(css);
    let mut parser = Parser::new();
    let mut current_selector = String::new();
    let mut current_property = String::new();
    let mut current_value = String::new();

    while let Some(token) = lexer.next() {
        if token.is_err() && !token.is_ok() {
            continue;
        }

        let token = token.unwrap();

        match token {
            Token::ClassSelector(class) => {
                current_selector = class;
                parser.create_selector(current_selector.clone());
            }
            Token::Property(property) => {
                current_property = property;
                parser.add_property(
                    current_selector.clone(),
                    current_property.clone(),
                    current_value.clone()
                );
            }
            Token::Value(value) => {
                if !current_selector.is_empty() && !current_property.is_empty() {
                    current_value = value;
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                }
            }
            Token::Semicolon => {
                current_property.clear();
                current_value.clear();
            }
            Token::CloseBrace => {
                current_selector.clear();
            }
            Token::Function(function) => {
                if !current_selector.is_empty() && !current_property.is_empty() {
                    current_value = function;
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                }
            }
            Token::HexColor(hex) => {
                if !current_selector.is_empty() && !current_property.is_empty() {
                    current_value = hex;
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                }
            }
            Token::NumericValue(percentage) => {
                if !current_selector.is_empty() && !current_property.is_empty() {
                    current_value = percentage;
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                }
            }
            _ => {}
        }
    }
    let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
    println!("{}", parser);
}
