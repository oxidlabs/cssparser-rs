use std::collections::HashMap;

use cssparser_rs::Token;
use logos::Logos;

fn main() {
    let css =
        r#"
        .my-class {
            color: red;
            background-color: rgba(255, 99, 71, 0.5);
            font-size: 16px;
        }
    "#;

    let mut lexer = Token::lexer(css);
    let mut properties: HashMap<String, HashMap<String, String>> = HashMap::new();
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
                println!("Class: {}", class);
                current_selector = class;
                properties.insert(current_selector.clone(), HashMap::new());
            }
            Token::Property(property) => {
                println!("Property: {}", property);
                current_property = property;
                properties
                    .get_mut(&current_selector)
                    .unwrap()
                    .insert(current_property.clone(), String::new());
            }
            Token::Value(value) => {
                println!("Value: {}", value);
                if !current_selector.is_empty() && !current_property.is_empty() {
                    current_value = value;
                    properties
                        .get_mut(&current_selector)
                        .unwrap()
                        .insert(current_property.clone(), current_value.clone());
                }
            }
            Token::Semicolon => {
                println!("Semicolon");
                if !current_selector.is_empty() && !current_property.is_empty() {
                    properties
                        .get_mut(&current_selector)
                        .unwrap()
                        .insert(current_property.clone(), current_value.clone());
                }
            }
            Token::CloseBrace => {
                println!("{}: {:?}", current_selector, properties);
                current_selector.clear();
                current_property.clear();
                current_value.clear();
            }
            Token::Function(function) => {
                println!("Function: {}", function);
                if !current_selector.is_empty() && !current_property.is_empty() {
                    current_value = function;
                    properties
                        .get_mut(&current_selector)
                        .unwrap()
                        .insert(current_property.clone(), current_value.clone());
                }
            }
            Token::HexColor(hex) => {
                println!("HexColor: {}", hex);
                if !current_selector.is_empty() && !current_property.is_empty() {
                    current_value = hex;
                    properties
                        .get_mut(&current_selector)
                        .unwrap()
                        .insert(current_property.clone(), current_value.clone());
                }
            }
            _ => {}
        }
    }

    println!("{:?}", properties);
}
