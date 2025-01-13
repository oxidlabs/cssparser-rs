use cssparser_rs::{parse_css, parser::Parser};

fn main() {
    let input = include_str!("../bootstrap-4.css");
    let start = std::time::Instant::now();
    let mut parser = Parser::new(input);
    let style_sheet = parser.parse_stylesheet().unwrap();
    let end = start.elapsed();
    println!("Time: {:?}", end);
    //minify_css(css);
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
