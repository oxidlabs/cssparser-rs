#![allow(unused)]

use core::fmt;
use std::{ collections::HashMap, fmt::{ Display, Formatter }, fs::File, io::BufWriter };
use std::sync::{ Arc, Mutex, mpsc };
use std::thread;
use std::io::Write;

use cssparser_rs::Token;
use logos::{ Lexer, Logos };

use rand::{ thread_rng, Rng };
use rand::distributions::Alphanumeric;

#[derive(Clone)]
struct Parser {
    selectors: HashMap<Vec<u8>, Properties>,
}

#[derive(Debug, Clone)]
struct Properties {
    properties: HashMap<Vec<u8>, Vec<Vec<u8>>>,
}

impl Parser {
    fn new() -> Self {
        Self {
            selectors: HashMap::new(),
        }
    }

    fn create_selector(&mut self, selector: Vec<u8>) {
        self.selectors.insert(selector, Properties::new());
    }

    fn add_property(&mut self, selector: Vec<u8>, property: Vec<u8>, value: Option<Vec<u8>>) {
        if self.selectors.contains_key(&selector) {
            self.selectors.get_mut(&selector).unwrap().insert(property, value);
        }
    }

    fn update_property(&mut self, selector: Vec<u8>, property: Vec<u8>, value: Vec<u8>) {
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

    fn insert(&mut self, property: Vec<u8>, value: Option<Vec<u8>>) {
        if self.properties.contains_key(&property) {
            if let Some(prop) = self.properties.get_mut(&property) {
                if let Some(value) = value {
                    prop.push(value);
                }
            } else {
                panic!("Failed to insert property: {:?} with value: {:?}", property, value);
            }
        } else {
            if let Some(value) = value {
                self.properties.insert(property, vec![value]);
            } else {
                self.properties.insert(property, vec![]);
            }
        }
    }

    fn get(&self, property: &[u8]) -> Option<&Vec<Vec<u8>>> {
        self.properties.get(property)
    }

    fn get_mut(&mut self, property: &[u8]) -> Option<&mut Vec<Vec<u8>>> {
        self.properties.get_mut(property)
    }

    fn remove(&mut self, property: &[u8]) -> Option<Vec<Vec<u8>>> {
        self.properties.remove(property)
    }
}

impl Display for Parser {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (selector, properties) in &self.selectors {
            writeln!(f, "{} {{", std::str::from_utf8(selector).unwrap_or_default())?;

            for (property, values) in &properties.properties {
                let prop = std::str::from_utf8(property).unwrap_or_default();
                let vals: Vec<&str> = values
                    .iter()
                    .map(|v| std::str::from_utf8(v).unwrap_or_default())
                    .collect();
                writeln!(f, "  {}: {};", prop, vals.join(" "))?;
            }

            writeln!(f, "}}")?;
        }

        Ok(())
    }
}

fn parse_chunk(chunk: &str, parser: &mut Parser) {
    let mut lexer = Token::lexer(chunk);
    let mut current_selector = Vec::with_capacity(1024);
    let mut current_property: Vec<u8> = Vec::with_capacity(1024);
    let mut current_value: Vec<u8> = Vec::with_capacity(1024);
    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::Value(value)) => {
                if current_selector.is_empty() {
                    current_selector.write(value.as_bytes());
                } else if !current_property.is_empty() {
                    // replace current_value with value
                    current_value.clear();
                    current_value.write(value.as_bytes());
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                } else {
                    // extend current_selector with value
                    current_selector.write(value.as_bytes());
                }
            }
            Ok(Token::PseudoClass(pseudo)) | Ok(Token::PseudoElement(pseudo)) => {
                current_selector.write(pseudo.as_bytes());
            }
            Ok(Token::IdSelector(value)) | Ok(Token::ClassSelector(value)) => {
                current_selector.write(value.as_bytes());
            }
            Ok(Token::Property(property)) => {
                current_property.clear();
                current_property.write(property.as_bytes());
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
                    current_value.clear();
                    current_value.write(value.as_bytes());
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                }
            }
            Ok(Token::StringValue(value)) => {
                if !current_property.is_empty() {
                    current_value.clear();
                    current_value.write(value.as_bytes());
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                }
            }
            Ok(Token::ChildCombinator) => {
                current_selector.write(b" > ");
            }
            Ok(Token::Function(value)) => {
                if !current_property.is_empty() {
                    // check if the value is rgb or rgba if so, convert it to hex
                    if value.starts_with("rgba") {
                        let mut values = value
                            .trim_start_matches("rgba(")
                            .trim_end_matches(')')
                            .split(',')
                            .map(|v| v.trim())
                            .collect::<Vec<&str>>();

                        let hex = format!(
                            "#{:02x}{:02x}{:02x}{:02x}",
                            values[0].parse::<u8>().expect("Expected to parse red value"),
                            values[1].parse::<u8>().expect("Expected to parse green value"),
                            values[2].parse::<u8>().expect("Expected to parse blue value"),
                            (
                                values[3]
                                    .parse::<f32>()
                                    .expect("Expected to parse alpha value")
                                    .clamp(0.0, 1.0) * 255.0
                            ).round() as u8
                        );
                        current_value.clear();
                        current_value.write(hex.as_bytes());
                        parser.update_property(
                            current_selector.clone(),
                            current_property.clone(),
                            current_value.clone()
                        );
                    } else if value.starts_with("rgb") {
                        let mut values = value
                            .trim_start_matches("rgb(")
                            .trim_end_matches(')')
                            .split(',')
                            .map(|v| v.trim().parse::<u8>().unwrap())
                            .collect::<Vec<u8>>();

                        let hex = format!("#{:02x}{:02x}{:02x}", values[0], values[1], values[2]);
                        current_value.clear();
                        current_value.write(hex.as_bytes());
                        parser.update_property(
                            current_selector.clone(),
                            current_property.clone(),
                            current_value.clone()
                        );
                    } else if value.starts_with("hsla") {
                        let mut values = value
                            .trim_start_matches("hsla(")
                            .trim_end_matches(')')
                            .split(',')
                            .map(|v| {
                                let v = v.trim();
                                if v.ends_with('%') {
                                    v.trim_end_matches('%').parse::<f32>().unwrap()
                                } else {
                                    v.parse::<f32>().unwrap()
                                }
                            })
                            .collect::<Vec<f32>>();

                        let h = (values[0] % 360.0) / 360.0; // normalize h to 0-1 range
                        let s = values[1] / 100.0;
                        let l = values[2] / 100.0;
                        let a = values[3].clamp(0.0, 1.0); // ensure alpha is between 0 and 1

                        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
                        let h_prime = h * 6.0;
                        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
                        let m = l - c / 2.0;

                        let (r, g, b) = if h_prime <= 1.0 {
                            (c, x, 0.0)
                        } else if h_prime <= 2.0 {
                            (x, c, 0.0)
                        } else if h_prime <= 3.0 {
                            (0.0, c, x)
                        } else if h_prime <= 4.0 {
                            (0.0, x, c)
                        } else if h_prime <= 5.0 {
                            (x, 0.0, c)
                        } else {
                            (c, 0.0, x)
                        };

                        let r = ((r + m) * 255.0).round() as u8;
                        let g = ((g + m) * 255.0).round() as u8;
                        let b = ((b + m) * 255.0).round() as u8;

                        let hex = format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, (a * 255.0) as u8);
                        current_value.clear();
                        current_value.write(hex.as_bytes());
                        parser.update_property(
                            current_selector.clone(),
                            current_property.clone(),
                            current_value.clone()
                        );
                    } else if value.starts_with("hsl") {
                        let mut values = value
                            .trim_start_matches("hsl(")
                            .trim_end_matches(')')
                            .split(',')
                            .map(|v| {
                                let v = v.trim();
                                if v.ends_with('%') {
                                    v.trim_end_matches('%').parse::<f32>().unwrap()
                                } else {
                                    v.parse::<f32>().unwrap()
                                }
                            })
                            .collect::<Vec<f32>>();

                        let h = (values[0] % 360.0) / 360.0; // normalize h to 0-1 range
                        let s = values[1] / 100.0;
                        let l = values[2] / 100.0;

                        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
                        let x = c * (1.0 - (((h * 6.0) % 2.0) - 1.0).abs());
                        let m = l - c / 2.0;

                        let (r, g, b) = if h < 1.0 / 6.0 {
                            (c, x, 0.0)
                        } else if h < 2.0 / 6.0 {
                            (x, c, 0.0)
                        } else if h < 3.0 / 6.0 {
                            (0.0, c, x)
                        } else if h < 4.0 / 6.0 {
                            (0.0, x, c)
                        } else if h < 5.0 / 6.0 {
                            (x, 0.0, c)
                        } else {
                            (c, 0.0, x)
                        };

                        let r = ((r + m) * 255.0).round() as u8;
                        let g = ((g + m) * 255.0).round() as u8;
                        let b = ((b + m) * 255.0).round() as u8;

                        let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
                        current_value.clear();
                        current_value.write(hex.as_bytes());
                        parser.update_property(
                            current_selector.clone(),
                            current_property.clone(),
                            current_value.clone()
                        );
                    } else {
                        current_value.clear();
                        current_value.write(value.as_bytes());
                        parser.update_property(
                            current_selector.clone(),
                            current_property.clone(),
                            current_value.clone()
                        );
                    }
                }
            }
            Ok(Token::Comment) => {
                // ignore comments
            }
            Ok(Token::HexColor(value)) => {
                if !current_property.is_empty() {
                    current_value.clear();
                    current_value.write(value.as_bytes());
                    parser.update_property(
                        current_selector.clone(),
                        current_property.clone(),
                        current_value.clone()
                    );
                }
            }
            Ok(Token::AttributeSelector(value)) => {
                current_selector.write(value.as_bytes());
            }
            Ok(Token::Comma) => {
                if current_property.is_empty() {
                    current_selector.write(b",");
                } else {
                    current_value.write(b",");
                }
            }
            _ => {
                /* let err: cssparser_rs::Result<String> = Err((
                    format!("Unexpected token: {:?}", lexer.slice()),
                    lexer.span(),
                ));

                println!("{:?}", err.err()); */
            }
        }
    }
}

fn main() {
    // Read the css file
    let css = std::fs::read_to_string("bootstrap-4.css").expect("Failed to read CSS file");
    let mut parser = Parser::new();

    let start = std::time::Instant::now();
    parse_chunk(css.as_str(), &mut parser);

    /*     let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
    let minified = minify(parser.lock().unwrap().clone());
    // write to file
    std::fs::write("minified.css", minified).unwrap(); */
    let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
    //println!("{}", parser.lock().unwrap());
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
