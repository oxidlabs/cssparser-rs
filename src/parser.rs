use crate::ast::*;
use crate::ParseError;
use crate::Token;
use logos::Lexer;
use logos::Logos;
use std::fmt::format;
use std::str::FromStr;

pub struct Parser<'a> {
    lexer: Lexer<'a, Token<'a>>,
    current: Option<Result<Token<'a>, ()>>,
    slice: &'a str,
    span: std::ops::Range<usize>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Token::lexer(source);
        let current = lexer.next();
        let slice = lexer.slice();
        let span = lexer.span();
        Parser {
            lexer,
            current,
            slice,
            span,
        }
    }

    fn advance(&mut self) {
        self.current = self.lexer.next();
        if let Some(ref token_res) = self.current {
            self.span = self.lexer.span();
        }
    }

    fn parse_selectors(&mut self) -> Result<Vec<Selector>, ParseError> {
        let mut selectors = Vec::new();
        let mut has_seperator = true;

        while let Some(token_res) = &self.current {
            match token_res {
                Ok(Token::CurlyBracketBlock(_)) => {
                    break;
                }
                // Handle ID Selectors (e.g., #header)
                Ok(Token::Hash(id)) => {
                    selectors.push(
                        Selector::Simple(SimpleSelector {
                            tag: None,
                            id: Some(id.to_string()),
                            classes: Vec::new(),
                        })
                    );
                    has_seperator = false;
                    self.advance();
                }
                // Handle Universal Selectors (e.g., *)
                Ok(Token::Delim("*")) => {
                    selectors.push(
                        Selector::Simple(SimpleSelector {
                            tag: Some("*".to_string()),
                            id: None,
                            classes: Vec::new(),
                        })
                    );
                    has_seperator = false;
                    self.advance();
                }
                | Ok(Token::PseudoClassSelector(pseudo_class))
                | Ok(Token::SquareBracketBlock(pseudo_class))
                | Ok(Token::ParenthesisBlock(pseudo_class)) => {
                    if has_seperator {
                        selectors.push(
                            Selector::Simple(SimpleSelector {
                                tag: Some(pseudo_class.to_string()),
                                id: None,
                                classes: Vec::new(),
                            })
                        );
                    } else {
                        if let Some(Selector::Simple(selector)) = selectors.last_mut() {
                            selector.classes.push(pseudo_class.to_string());
                        }
                    }
                    self.advance();
                }
                // Handle Class Selectors (e.g., .container)
                Ok(Token::ClassSelector(class)) => {
                    selectors.push(
                        Selector::Simple(SimpleSelector {
                            tag: Some(class.to_string()),
                            id: None,
                            classes: Vec::new(),
                        })
                    );
                    has_seperator = false;
                    self.advance();
                }
                // Handle Type Selectors (e.g., body, div)
                Ok(Token::Ident(tag)) => {
                    selectors.push(
                        Selector::Simple(SimpleSelector {
                            tag: Some(tag.to_string()),
                            id: None,
                            classes: Vec::new(),
                        })
                    );
                    has_seperator = false;
                    self.advance();
                }
                Ok(Token::Delim(">")) | Ok(Token::Delim("+")) | Ok(Token::Delim("~")) => {
                    // Combinators: Child (>), Adjacent Sibling (+), General Sibling (~)
                    let combinator = match self.current.clone().unwrap().unwrap() {
                        Token::Delim(c) => c,
                        _ => unreachable!(),
                    };
                    self.advance();

                    // Capture the next selector
                    if let Some(token_res) = &self.current {
                        if let Ok(next_token) = token_res {
                            let next_selector = match next_token {
                                Token::Ident(tag) => tag.to_string(),
                                Token::ClassSelector(class) => format!("{}", class),
                                Token::Hash(id) => format!("#{}", id),
                                _ => "".to_string(),
                            };

                            if let Some(selector) = selectors.last_mut() {
                                if let Selector::Simple(selector) = selector {
                                    let combined_tag = match &selector.tag {
                                        Some(tag) =>
                                            format!("{} {} {}", tag, combinator, next_selector),
                                        None => next_selector,
                                    };
                                    selector.tag = Some(combined_tag);
                                }
                            }
                            self.advance();
                        }
                    }
                    has_seperator = true;
                }
                // Handle Comma delimiter between multiple selectors
                Ok(Token::Delim(",")) => {
                    has_seperator = true;
                    self.advance(); // Skip the comma
                }
                // If none of the above, selectors parsing is complete
                _ => {
                    break;
                }
            }
        }

        Ok(selectors)
    }

    fn parse_rule_set(&mut self) -> Result<RuleSet, ParseError> {
        let selectors = self.parse_selectors()?;
        println!("Selectors: {:?}", selectors);
        match &self.current.clone() {
            Some(Ok(Token::CurlyBracketBlock(block))) => {
                self.advance();

                let mut declarations = Vec::new();
                let mut nested_rules = Vec::new();

                // Only remove first { and last }
                let block_content = (
                    if block.starts_with('{') && block.ends_with('}') {
                        &block[1..block.len() - 1]
                    } else {
                        block
                    }
                ).trim();

                let mut block_parser = Parser::new(block_content);

                while let Some(token_res) = &block_parser.current.clone() {
                    match token_res {
                        Ok(Token::Ident(property)) => {
                            println!("{:?}", block_parser.current);
                            block_parser.advance();
                            match &block_parser.current {
                                Some(Ok(Token::Delim(":"))) => {
                                    block_parser.advance();
                                }
                                _ => {
                                    return Err((
                                        "Expected ':' after property".to_string(),
                                        block_parser.span.clone(),
                                    ));
                                }
                            }

                            let value = block_parser.parse_declaration_value()?;
                            println!("{:?}", value);
                            println!("{:?}", block_parser.current);
                            match &block_parser.current {
                                Some(Ok(Token::Delim(";"))) => {
                                    block_parser.advance();
                                }
                                _ => {
                                    return Err((
                                        "Expected ';' after value".to_string(),
                                        block_parser.span.clone(),
                                    ));
                                }
                            }

                            declarations.push(Declaration {
                                property: property.to_string(),
                                value,
                            });
                        }
                        Ok(Token::CustomProperty(val)) => {
                            block_parser.advance();
                            match &block_parser.current {
                                Some(Ok(Token::Delim(":"))) => {
                                    block_parser.advance();
                                }
                                _ => {
                                    return Err((
                                        "Expected ':' after property".to_string(),
                                        block_parser.span.clone(),
                                    ));
                                }
                            }

                            let value = block_parser.parse_declaration_value()?;

                            match &block_parser.current {
                                Some(Ok(Token::Delim(";"))) => {
                                    block_parser.advance();
                                }
                                _ => {
                                    return Err((
                                        "Expected ';' after value".to_string(),
                                        block_parser.span.clone(),
                                    ));
                                }
                            }

                            declarations.push(Declaration {
                                property: val.to_string(),
                                value,
                            });
                        }
                        Ok(Token::Comment(val)) => {
                            block_parser.advance();
                        }
                        | Ok(Token::ClassSelector(_))
                        | Ok(Token::Hash(_))
                        | Ok(Token::ParenthesisBlock(_))
                        | Ok(Token::SquareBracketBlock(_)) => {
                            let nested_rule = block_parser.parse_rule_set()?;
                            nested_rules.push(Rule::RuleSet(nested_rule));
                        }

                        _ => block_parser.advance(),
                    }
                }

                Ok(RuleSet {
                    selectors,
                    declarations,
                    nested_rules,
                })
            }
            _ => {
                println!("{:?}", self.current);
                Err(("Expected '{' after selectors".to_string(), self.span.clone()))
            }
        }
    }

    fn parse_declaration_value(&mut self) -> Result<Vec<Value>, ParseError> {
        let mut values = Vec::new();

        while let Some(token_res) = &self.current.clone() {
            match token_res {
                Ok(Token::Delim(";")) => {
                    break;
                }
                Ok(Token::Hash(val)) => {
                    values.push(Value::Color(ColorValue::Hex(val.to_string())));
                    self.advance();
                }
                Ok(Token::Ident(val)) => {
                    values.push(Value::Identifier(val.to_string()));
                    self.advance();
                }
                Ok(Token::Number(val)) => {
                    values.push(Value::Number(f64::from_str(val).unwrap()));
                    self.advance();
                }
                Ok(Token::Dimension(val)) => {
                    // Parse number and unit
                    let (num, unit) = val.split_at(
                        val.find(|c: char| c.is_alphabetic()).unwrap_or(val.len())
                    );
                    if let Ok(value) = num.parse::<f64>() {
                        values.push(Value::Dimension {
                            value,
                            unit: unit.to_string(),
                        });
                    }

                    self.advance();
                }
                Ok(Token::Function(name)) => {
                    self.advance();
                    let function_value = self.parse_function(name)?;
                    values.push(function_value);
                }
                Ok(Token::Delim(",")) => {
                    self.advance();
                    continue;
                }
                Ok(Token::QuotedString(val)) => {
                    values.push(Value::String(val.to_string()));
                    self.advance();
                }
                Ok(Token::Percentage(val)) => {
                    values.push(
                        Value::Percentage(f64::from_str(val.trim_end_matches('%')).unwrap())
                    );
                    self.advance();
                }
                Ok(Token::CustomProperty(val)) => {
                    values.push(Value::Var(val.to_string()));
                    self.advance();
                }
                Ok(Token::Important) => {
                    values.push(Value::Identifier("!important".to_string()));
                    self.advance();
                }
                Ok(Token::UnquotedUrl(val)) => {
                    values.push(Value::Uri(val.to_string()));
                    self.advance();
                }
                Ok(Token::Delim("/")) => {
                    values.push(Value::Identifier("/".to_string()));
                    self.advance();
                }
                _ => {
                    break;
                }
            }
        }

        Ok(values)
    }

    fn parse_function(&mut self, name: &str) -> Result<Value, ParseError> {
        match name {
            "rgb" | "rgba" => {
                let mut values = Vec::new();

                while let Some(token_res) = &self.current {
                    match token_res {
                        Ok(Token::Number(val)) => {
                            values.push(*val);
                            self.advance();
                        }
                        Ok(Token::Delim(",")) => {
                            self.advance();
                        }
                        Ok(Token::ParenthesisBlock(_)) | Ok(Token::Delim(")")) => {
                            self.advance();
                            break;
                        }
                        _ => {
                            return Err(("Invalid rgb/rgba value".to_string(), self.span.clone()));
                        }
                    }
                }

                if values.len() < 3 || values.len() > 4 {
                    return Err((
                        "Invalid number of arguments for rgb/rgba".to_string(),
                        self.span.clone(),
                    ));
                }

                let r = u8::from_str(values[0]).unwrap();
                let g = u8::from_str(values[1]).unwrap();
                let b = u8::from_str(values[2]).unwrap();
                let a = values.get(3).map(|v| f32::from_str(*v).unwrap());

                if let Some(a) = a {
                    Ok(Value::Color(ColorValue::Rgba { r, g, b, a }))
                } else {
                    Ok(Value::Color(ColorValue::Rgb { r, g, b }))
                }
            }

            "calc" => {
                let mut terms = Vec::new();

                while let Some(token_res) = &self.current {
                    match token_res {
                        Ok(Token::Number(val)) => {
                            terms.push(CalcTerm::Number(f64::from_str(*val).unwrap(), None));
                            self.advance();
                        }
                        Ok(Token::Delim("%")) => {
                            terms.push(CalcTerm::Number(0.0, Some("%".to_string())));
                            self.advance();
                        }
                        Ok(Token::Delim("+")) => {
                            terms.push(CalcTerm::Operator(CalcOperator::Add));
                            self.advance();
                        }
                        Ok(Token::Delim("-")) => {
                            terms.push(CalcTerm::Operator(CalcOperator::Subtract));
                            self.advance();
                        }
                        Ok(Token::Delim("*")) => {
                            terms.push(CalcTerm::Operator(CalcOperator::Multiply));
                            self.advance();
                        }
                        Ok(Token::Delim("/")) => {
                            terms.push(CalcTerm::Operator(CalcOperator::Divide));
                            self.advance();
                        }
                        Ok(Token::ParenthesisBlock(_)) | Ok(Token::Delim(")")) => {
                            self.advance();
                            break;
                        }
                        Ok(Token::Dimension(val)) => {
                            let (num, unit) = val.split_at(
                                val.find(|c: char| c.is_alphabetic()).unwrap_or(val.len())
                            );
                            if let Ok(value) = num.parse::<f64>() {
                                terms.push(CalcTerm::Number(value, Some(unit.to_string())));
                            }
                            self.advance();
                        }
                        _ => {
                            return Err(("Invalid calc term".to_string(), self.span.clone()));
                        }
                    }
                }

                Ok(Value::Calc(CalcExpression { terms }))
            }
            "url" => {
                let mut url = String::new();

                while let Some(token_res) = &self.current {
                    match token_res {
                        Ok(Token::UnquotedUrl(val)) => {
                            url.push_str(val);
                            self.advance();
                        }
                        Ok(Token::QuotedString(val)) => {
                            url.push_str(val);
                            self.advance();
                        }
                        Ok(Token::ParenthesisBlock(_)) | Ok(Token::Delim(")")) => {
                            self.advance();
                            break;
                        }
                        _ => {
                            return Err(("Invalid url value".to_string(), self.span.clone()));
                        }
                    }
                }

                Ok(Value::Uri(url))
            }
            "rect" => {
                let mut values = Vec::new();

                while let Some(token_res) = &self.current {
                    match token_res {
                        Ok(Token::Number(val)) => {
                            values.push(*val);
                            self.advance();
                        }
                        Ok(Token::Delim(",")) => {
                            self.advance();
                        }
                        Ok(Token::ParenthesisBlock(_)) | Ok(Token::Delim(")")) => {
                            self.advance();
                            break;
                        }
                        _ => {
                            return Err(("Invalid rect value".to_string(), self.span.clone()));
                        }
                    }
                }

                if values.len() != 4 {
                    return Err((
                        "Invalid number of arguments for rect".to_string(),
                        self.span.clone(),
                    ));
                }

                Ok(
                    Value::Function(FunctionValue {
                        name: "rect".to_string(),
                        arguments: values
                            .into_iter()
                            .map(|v| Value::Number(f64::from_str(v).unwrap()))
                            .collect(),
                    })
                )
            }
            _ => {
                // For unknown functions, collect all values until closing parenthesis
                let mut arguments = Vec::new();

                while let Some(token_res) = &self.current {
                    match token_res {
                        Ok(Token::ParenthesisBlock("")) => {
                            self.advance();
                            break;
                        }
                        Ok(_) => {
                            let value = self.parse_declaration_value()?;
                            arguments.extend(value);
                            self.advance();
                        }
                        _ => {
                            return Err((
                                "Invalid function argument".to_string(),
                                self.span.clone(),
                            ));
                        }
                    }
                }

                Ok(
                    Value::Function(FunctionValue {
                        name: name.to_string(),
                        arguments,
                    })
                )
            }
        }
    }

    pub fn parse_stylesheet(&mut self) -> Result<Stylesheet, ParseError> {
        let mut rules = Vec::new();

        while let Some(token_res) = &self.current.clone() {
            match token_res {
                // At-rules like @media
                Ok(Token::AtKeyword(keyword)) => {
                    self.advance();
                    let at_rule = self.parse_at_rule(keyword)?;
                    rules.push(Rule::AtRule(at_rule));
                }
                // Regular rule sets (type, class, id selectors)
                | Ok(Token::Ident(_))
                | Ok(Token::ClassSelector(_))
                | Ok(Token::Hash(_))
                | Ok(Token::Delim("*"))
                | Ok(Token::PseudoClassSelector(_))
                | Ok(Token::SquareBracketBlock(_)) => {
                    let rule_set = self.parse_rule_set()?;
                    rules.push(Rule::RuleSet(rule_set));
                }
                /* Ok(Token::PseudoClassSelector(val)) => {
                    // check if the last rule has a empty declaration block if so then add the pseudo class to the last selector classes
                    println!("{:?}", rules);
                    if let Some(Rule::RuleSet(rule)) = rules.last_mut() {
                        if rule.declarations.is_empty() {
                            println!("Adding pseudo class to last selector");
                            println!("{:?}", rule.selectors);
                            println!("{:?}", val);
                            if let Some(Selector::Simple(selector)) = rule.selectors.last_mut() {
                                selector.classes.push(val.to_string());
                            }
                        }
                    } else {
                        let rule_set = self.parse_rule_set()?;
                        rules.push(Rule::RuleSet(rule_set));
                    }
                } */
                // Skip whitespace or unexpected tokens
                _ => {
                    self.advance();
                }
            }
        }

        Ok(Stylesheet { rules })
    }

    fn parse_at_rule(&mut self, name: &str) -> Result<AtRule, ParseError> {
        let mut prelude = Vec::new();

        // Parse prelude until CurlyBracketBlock
        while let Some(token_res) = &self.current {
            match token_res {
                Ok(Token::CurlyBracketBlock(block)) => {
                    let mut block_rules = Vec::new();

                    // Parse block content
                    let block_content = (
                        if block.starts_with('{') && block.ends_with('}') {
                            &block[1..block.len() - 1]
                        } else {
                            block
                        }
                    ).trim();

                    let mut block_parser = Parser::new(block_content);

                    // Parse nested rules
                    while let Some(inner_token) = &block_parser.current.clone() {
                        match inner_token {
                            | Ok(Token::Ident(_))
                            | Ok(Token::ClassSelector(_))
                            | Ok(Token::Hash(_)) => {
                                let rule = block_parser.parse_rule_set()?;
                                block_rules.push(Rule::RuleSet(rule));
                            }
                            Ok(Token::AtKeyword(keyword)) => {
                                block_parser.advance();
                                let at_rule = block_parser.parse_at_rule(keyword)?;
                                block_rules.push(Rule::AtRule(at_rule));
                            }
                            _ => block_parser.advance(),
                        }
                    }

                    self.advance();
                    return Ok(AtRule {
                        name: name.to_string(),
                        prelude,
                        block: Some(Stylesheet { rules: block_rules }),
                    });
                }
                | Ok(Token::Ident(val))
                | Ok(Token::Number(val))
                | Ok(Token::Dimension(val))
                | Ok(Token::ParenthesisBlock(val))
                | Ok(Token::SquareBracketBlock(val)) => {
                    prelude.push(val.to_string());
                    self.advance();
                }
                _ => self.advance(),
            }
        }

        Err(("Expected '{' in @rule".to_string(), self.span.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule_set() {
        let css = "body { color: #fff; }";
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: Some("body".to_string()),
                            id: None,
                            classes: Vec::new(),
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: vec![Value::Color(ColorValue::Hex("fff".to_string()))],
                    }],
                    nested_rules: Vec::new(),
                })
            ],
        };

        assert_eq!(ast, expected);

        let css = "#main { background-color: #fff; }";
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: None,
                            id: Some("main".to_string()),
                            classes: Vec::new(),
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "background-color".to_string(),
                        value: vec![Value::Color(ColorValue::Hex("fff".to_string()))],
                    }],
                    nested_rules: Vec::new(),
                })
            ],
        };

        assert_eq!(ast, expected);

        let css = ".container { width: 100%; }";
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: None,
                            id: None,
                            classes: vec![".container".to_string()],
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "width".to_string(),
                        value: vec![Value::Percentage(100.0)],
                    }],
                    nested_rules: Vec::new(),
                })
            ],
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_multi_value() {
        let css = "body { font-family: Arial, sans-serif; }";
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: Some("body".to_string()),
                            id: None,
                            classes: Vec::new(),
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "font-family".to_string(),
                        value: vec![
                            Value::Identifier("Arial".to_string()),
                            Value::Identifier("sans-serif".to_string())
                        ],
                    }],
                    nested_rules: Vec::new(),
                })
            ],
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_nested_rule_set() {
        let css = "body { color: #fff; .container { width: 100%; } }";
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: Some("body".to_string()),
                            id: None,
                            classes: Vec::new(),
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: vec![Value::Color(ColorValue::Hex("fff".to_string()))],
                    }],
                    nested_rules: vec![
                        Rule::RuleSet(RuleSet {
                            selectors: vec![
                                Selector::Simple(SimpleSelector {
                                    tag: None,
                                    id: None,
                                    classes: vec![".container".to_string()],
                                })
                            ],
                            declarations: vec![Declaration {
                                property: "width".to_string(),
                                value: vec![Value::Percentage(100.0)],
                            }],
                            nested_rules: Vec::new(),
                        })
                    ],
                })
            ],
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_parse_at_rule() {
        let css = "@media screen and (max-width: 600px) { body { color: #000; } }";
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::AtRule(AtRule {
                    name: "media".to_string(),
                    prelude: vec![
                        "screen".to_string(),
                        "and".to_string(),
                        "(max-width: 600px)".to_string()
                    ],
                    block: Some(Stylesheet {
                        rules: vec![
                            Rule::RuleSet(RuleSet {
                                selectors: vec![
                                    Selector::Simple(SimpleSelector {
                                        tag: Some("body".to_string()),
                                        id: None,
                                        classes: Vec::new(),
                                    })
                                ],
                                declarations: vec![Declaration {
                                    property: "color".to_string(),
                                    value: vec![Value::Color(ColorValue::Hex("000".to_string()))],
                                }],
                                nested_rules: Vec::new(),
                            })
                        ],
                    }),
                })
            ],
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_selectors() {
        let css =
            r#"
            *,
            *::before,
            *::after {
                box-sizing: border-box;
            }
        "#;

        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: Some("*".to_string()),
                            id: None,
                            classes: Vec::new(),
                        }),
                        Selector::Simple(SimpleSelector {
                            tag: Some("*".to_string()),
                            id: None,
                            classes: vec!["::before".to_string()],
                        }),
                        Selector::Simple(SimpleSelector {
                            tag: Some("*".to_string()),
                            id: None,
                            classes: vec!["::after".to_string()],
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "box-sizing".to_string(),
                        value: vec![Value::Identifier("border-box".to_string())],
                    }],
                    nested_rules: Vec::new(),
                })
            ],
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_pseudo_selector() {
        let css = ":root { --blue: #007bff; }";
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: Some(":root".to_string()),
                            id: None,
                            classes: Vec::new(),
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "--blue".to_string(),
                        value: vec![Value::Color(ColorValue::Hex("007bff".to_string()))],
                    }],
                    nested_rules: Vec::new(),
                })
            ],
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_values() {
        let css =
            r#"
        :root {
    --font-family-monospace: SFMono-Regular, Menlo, Monaco, Consolas,
        "Liberation Mono", "Courier New", monospace;
}
        "#;
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: Some(":root".to_string()),
                            id: None,
                            classes: Vec::new(),
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "--font-family-monospace".to_string(),
                        value: vec![
                            Value::Identifier("SFMono-Regular".to_string()),
                            Value::Identifier("Menlo".to_string()),
                            Value::Identifier("Monaco".to_string()),
                            Value::Identifier("Consolas".to_string()),
                            Value::String("Liberation Mono".to_string()),
                            Value::String("Courier New".to_string()),
                            Value::Identifier("monospace".to_string())
                        ],
                    }],
                    nested_rules: Vec::new(),
                })
            ],
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_function_args() {
        let css = "body { color: rgb(255, 0, 0); }";
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: Some("body".to_string()),
                            id: None,
                            classes: Vec::new(),
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: vec![Value::Color(ColorValue::Rgb { r: 255, g: 0, b: 0 })],
                    }],
                    nested_rules: Vec::new(),
                })
            ],
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_combinators() {
        let css = ".custom-switch .custom-control-label::before { left: -2.25rem; }";
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: Some(".custom-switch".to_string()),
                            id: None,
                            classes: Vec::new(),
                        }),
                        Selector::Simple(SimpleSelector {
                            tag: Some(".custom-control-label".to_string()),
                            id: None,
                            classes: vec!["::before".to_string()],
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "left".to_string(),
                        value: vec![Value::Dimension { value: -2.25, unit: "rem".to_string() }],
                    }],
                    nested_rules: Vec::new(),
                })
            ],
        };

        assert_eq!(ast, expected);

        let css = "body > .container + .container { margin-top: 1rem; }";
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: Some("body > .container + .container".to_string()),
                            id: None,
                            classes: Vec::new(),
                        })
                    ],
                    declarations: vec![Declaration {
                        property: "margin-top".to_string(),
                        value: vec![Value::Dimension { value: 1.0, unit: "rem".to_string() }],
                    }],
                    nested_rules: Vec::new(),
                })
            ],
        };

        assert_eq!(ast, expected);

        let css =
            r#"
            .custom-switch .custom-control-input:checked ~ .custom-control-label::after {
                    background-color: #fff;
                    -webkit-transform: translateX(0.75rem);
                    transform: translateX(0.75rem);
            }"#;
        let mut parser = Parser::new(css);
        let ast = parser.parse_stylesheet().unwrap();

        let expected = Stylesheet {
            rules: vec![
                Rule::RuleSet(RuleSet {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag: Some(".custom-switch".to_string()),
                            id: None,
                            classes: Vec::new(),
                        }),
                        Selector::Simple(SimpleSelector {
                            tag: Some(".custom-control-input".to_string()),
                            id: None,
                            classes: vec![":checked".to_string()],
                        }),
                        Selector::Simple(SimpleSelector {
                            tag: None,
                            id: None,
                            classes: vec![".custom-control-label".to_string(), "::after".to_string()],
                        })
                    ],
                    declarations: vec![
                        
                    ],
                    nested_rules: Vec::new(),
                })
            ],
        };
    }
}
