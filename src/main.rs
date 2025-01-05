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
    properties: HashMap<String, Vec<String>>,
}

impl Parser {
    fn new() -> Self {
        Self {
            selectors: HashMap::new(),
        }
    }

    fn create_selector(&mut self, selector: String) {
        self.selectors.insert(selector, Properties::new());
    }

    fn add_property(&mut self, selector: String, property: String, value: Option<String>) {
        if self.selectors.contains_key(&selector) {
            self.selectors.get_mut(&selector).unwrap().insert(property, value);
        }
    }

    fn update_property(&mut self, selector: String, property: String, value: String) {
        if self.selectors.contains_key(&selector) {
            self.selectors.get_mut(&selector).unwrap().insert(property, Some(value));
        }
    }
}

impl Properties {
    fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    fn insert(&mut self, property: String, value: Option<String>) {
        if self.properties.contains_key(&property) {
            self.properties.get_mut(&property).unwrap().push(value.unwrap());
        } else {
            if let Some(value) = value {
                self.properties.insert(property, vec![value]);
            } else {
                self.properties.insert(property, vec![]);
            }
        }
    }

    fn get(&self, property: &str) -> Option<&Vec<String>> {
        self.properties.get(property)
    }

    fn get_mut(&mut self, property: &str) -> Option<&mut Vec<String>> {
        self.properties.get_mut(property)
    }

    fn remove(&mut self, property: &str) -> Option<Vec<String>> {
        self.properties.remove(property)
    }
}

impl Display for Parser {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (selector, properties) in &self.selectors {
            writeln!(f, "{} {{", selector)?;

            for (property, values) in &properties.properties {
                writeln!(f, "  {}: {};", property, values.join(" "))?;
            }

            writeln!(f, "}}")?;
        }

        Ok(())
    }
}

fn main() {
    let css =
        r#"
body {
  margin: 0;
}

.container {
  writing-mode: vertical-rl;
  direction: ltr;
  display: inline-block;
  position: relative;
  margin: 20px;
  border: solid 4px;
  width: 40px;
  height: 40px;
}

.item {
  position: absolute;
  background: green;
  inset: 5px;
  margin: 10px;
  width: 30px;
  height: 30px;
}

.safe {
  align-self: safe end;
}
.unsafe {
  align-self: unsafe end;
}

.rtl {
  direction: rtl;
}
    "#;

    let start = std::time::Instant::now();
    let mut lexer = Token::lexer(css);
    let mut parser = Parser::new();
    let mut current_selector = String::new();
    let mut current_property = String::new();
    let mut current_value = String::new();

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Value(value)) => {
                if current_selector.is_empty() {
                    current_selector = value;
                } else if !current_property.is_empty() {
                    current_value = value;
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                } else {
                    current_selector.push_str(&value);
                }
            }
            Ok(Token::PseudoClass(pseudo)) | Ok(Token::PseudoElement(pseudo)) => {
                current_selector.push_str(&pseudo);
            }
            Ok(Token::IdSelector(value)) | Ok(Token::ClassSelector(value)) => {
                current_selector = value;
            }
            Ok(Token::Property(property)) => {
                current_property = property;
                parser.add_property(current_selector.clone(), current_property.clone(), None);
            }
            Ok(Token::OpenBrace) => {
                parser.create_selector(current_selector.clone());
            }
            Ok(Token::CloseBrace) => {
                current_selector.clear();
                current_property.clear();
                current_value.clear();
            }
            Ok(Token::Semicolon) => {
                current_property.clear();
                current_value.clear();
            }
            Ok(Token::NumericValue(value)) => {
                if !current_property.is_empty() {
                    current_value = value;
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                }
            }
            Ok(Token::StringValue(value)) => {
                if !current_property.is_empty() {
                    current_value = value;
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                }
            }
            Ok(Token::ChildCombinator) => {
                current_selector.push_str(" > ");
            }
            _ => {
                let err: cssparser_rs::Result<String> = Err((
                    format!("Unexpected token: {:?}", lexer.slice()),
                    lexer.span(),
                ));

                println!("{:?}", err.err());
            }
        }
    }
    let minified = minify(parser);
    // write to file
    std::fs::write("minified.css", minified).unwrap();
    let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
    //println!("{}", parser);
}

fn minify(parser: Parser) -> String {
    /*
    input would be something like:
    .container {
        border: solid 4px;
        width: 100px;
        height: 100px;
    } 
    output would be:
    .container{border:solid 4px;width:100px;height:100px;}
    just make sure there is a space when there are multiple values
    */

    let mut minified = String::new();

    for (selector, properties) in &parser.selectors {
        minified.push_str(&selector.split_whitespace().collect::<String>());
        minified.push('{');

        // check if there is more than one value

        for (property, values) in &properties.properties {
            minified.push_str(&property);
            minified.push(':');
            minified.push_str(&values.join(" "));
            minified.push(';');
        }

        minified.push('}');
    }

    minified
}
