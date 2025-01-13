use serde::Serialize;

/// Represents the entire CSS stylesheet.
#[derive(Debug, PartialEq, Serialize)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

/// Represents a single rule in the stylesheet.
#[derive(Debug, PartialEq, Serialize)]
pub enum Rule {
    /// A standard CSS rule (selector + declarations).
    RuleSet(RuleSet),
    /// An @-rule (e.g., @media, @import).
    AtRule(AtRule),
}

/// Represents a standard CSS rule set.
#[derive(Debug, PartialEq, Serialize)]
pub struct RuleSet {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
    pub nested_rules: Vec<Rule>, // Support for nested rules
}

/// Represents an at-rule.
#[derive(Debug, PartialEq, Serialize)]
pub struct AtRule {
    pub name: String,
    pub prelude: Vec<String>, // Parameters or conditions
    pub block: Option<Stylesheet>, // Nested rules within the at-rule
}

/// Represents a CSS selector.
#[derive(Debug, PartialEq, Serialize)]
pub enum Selector {
    /// A simple selector (e.g., `.class`, `#id`, `tag`).
    Simple(SimpleSelector),
    /// An attribute selector (e.g., `[type="text"]`).
    Attribute(AttributeSelector),
    /// A pseudo-class selector (e.g., `:hover`, `:nth-child(2)`).
    PseudoClass(PseudoClassSelector),
    /// A pseudo-element selector (e.g., `::before`, `::after`).
    PseudoElement(PseudoElementSelector),
    /// A combinator selector (e.g., descendant, child, sibling).
    Combinator(CombinatorSelector),
    // Extend with more complex selectors as needed
}

/// Represents a simple selector.
#[derive(Debug, PartialEq, Serialize)]
pub struct SimpleSelector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    // Add more fields as necessary (e.g., attributes, pseudo-classes)
}

impl Default for SimpleSelector {
    fn default() -> Self {
        SimpleSelector {
            tag: None,
            id: None,
            classes: Vec::new(),
        }
    }
}

impl SimpleSelector {
    pub fn is_empty(&self) -> bool {
        self.tag.is_none() && self.id.is_none() && self.classes.is_empty()
    }
}

/// Represents an attribute selector.
#[derive(Debug, PartialEq, Serialize)]
pub struct AttributeSelector {
    pub attribute: String,
    pub operator: Option<AttributeOperator>,
    pub value: Option<String>,
}

/// Represents the operator in an attribute selector.
#[derive(Debug, PartialEq, Serialize)]
pub enum AttributeOperator {
    Equals,
    Includes,
    DashMatch,
    PrefixMatch,
    SuffixMatch,
    SubstringMatch,
}

/// Represents a pseudo-class selector.
#[derive(Debug, PartialEq, Serialize)]
pub struct PseudoClassSelector {
    pub name: String,
    pub argument: Option<String>, // e.g., nth-child(2)
}

/// Represents a pseudo-element selector.
#[derive(Debug, PartialEq, Serialize)]
pub struct PseudoElementSelector {
    pub name: String,
}

/// Represents a combinator selector.
#[derive(Debug, PartialEq, Serialize)]
pub struct CombinatorSelector {
    pub combinator: Combinator,
    pub selector: Option<Box<Selector>>,
}

/// Represents different types of combinators.
#[derive(Debug, PartialEq, Serialize)]
pub enum Combinator {
    Descendant,    // (space)
    Child,         // '>'
    AdjacentSibling, // '+'
    GeneralSibling,  // '~'
}

impl Combinator {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            " " => Some(Combinator::Descendant),
            ">" => Some(Combinator::Child),
            "+" => Some(Combinator::AdjacentSibling),
            "~" => Some(Combinator::GeneralSibling),
            _ => None,
        }
    }
}

/// Represents a CSS declaration.
#[derive(Debug, PartialEq, Serialize)]
pub struct Declaration {
    pub property: String,
    pub value: Vec<Value>,
}

/// Represents the value of a declaration.
#[derive(Debug, PartialEq, Serialize)]
pub enum Value {
    Identifier(String),
    ClassSelector(String),
    String(String),
    Number(f64),
    Percentage(f64),
    Dimension { value: f64, unit: String },
    Url(String),
    Function(FunctionValue),
    Calc(CalcExpression),
    Var(String),               // CSS Custom Properties (variables)
    Color(ColorValue),         // Color values
    Gradient(GradientValue),   // Gradient functions
    Angle(f64, AngleUnit),     // Angle values
    Time(f64, TimeUnit),       // Time values
    Frequency(f64, FrequencyUnit), // Frequency values
    Resolution(f64, ResolutionUnit), // Resolution values
    Uri(String),               // URI values
    // Add more value types as needed
}

/// Represents a function value (e.g., rgba(), linear-gradient()).
#[derive(Debug, PartialEq, Serialize)]
pub struct FunctionValue {
    pub name: String,
    pub arguments: Vec<Value>,
}

/// Represents a calc() expression.
#[derive(Debug, PartialEq, Serialize)]
pub struct CalcExpression {
    pub terms: Vec<CalcTerm>,
}

/// Represents a term within a calc() expression.
#[derive(Debug, PartialEq, Serialize)]
pub enum CalcTerm {
    Number(f64, Option<String>), // e.g., 100%, 20px
    Operator(CalcOperator),      // e.g., '-', '+', '*', '/'
}

/// Represents an operator in a calc() expression.
#[derive(Debug, PartialEq, Serialize)]
pub enum CalcOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// Represents CSS custom properties (variables).
#[derive(Debug, PartialEq, Serialize)]
pub struct CustomProperty {
    pub name: String,
    pub value: Value,
}

/// Represents different types of color values.
#[derive(Debug, PartialEq, Serialize)]
pub enum ColorValue {
    Hex(String), // e.g., "#fff", "#ffffff"
    Rgb { r: u8, g: u8, b: u8 },
    Rgba { r: u8, g: u8, b: u8, a: f32 },
    Hsl { h: f32, s: f32, l: f32 },
    Hsla { h: f32, s: f32, l: f32, a: f32 },
    Named(String), // e.g., "red", "blue"
}

/// Represents different gradient functions.
#[derive(Debug, PartialEq, Serialize)]
pub enum GradientValue {
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    RepeatingLinearGradient(LinearGradient),
    RepeatingRadialGradient(RadialGradient),
    // Add more gradient types as needed
}

/// Represents a linear gradient.
#[derive(Debug, PartialEq, Serialize)]
pub struct LinearGradient {
    pub direction: Option<Angle>, // e.g., 45deg, to right
    pub color_stops: Vec<ColorStop>,
}

/// Represents a radial gradient.
#[derive(Debug, PartialEq, Serialize)]
pub struct RadialGradient {
    pub shape: Option<String>, // e.g., circle, ellipse
    pub size: Option<String>,  // e.g., closest-side
    pub position: Option<Position>, // e.g., center, top left
    pub color_stops: Vec<ColorStop>,
}

/// Represents a color stop in a gradient.
#[derive(Debug, PartialEq, Serialize)]
pub struct ColorStop {
    pub color: Box<Value>,           // Color value
    pub position: Option<Box<Value>>, // Position value (e.g., 10%, 20px)
}

/// Represents different angle units.
#[derive(Debug, PartialEq, Serialize)]
pub enum AngleUnit {
    Degree,
    Grad,
    Radian,
    Turn,
}

/// Represents different time units.
#[derive(Debug, PartialEq, Serialize)]
pub enum TimeUnit {
    Second,
    Millisecond,
}

/// Represents different frequency units.
#[derive(Debug, PartialEq, Serialize)]
pub enum FrequencyUnit {
    Hertz,
    Kilohertz,
}

/// Represents different resolution units.
#[derive(Debug, PartialEq, Serialize)]
pub enum ResolutionUnit {
    Dpi,   // Dots per inch
    Dpcm,  // Dots per centimeter
    Dppx,  // Dots per pixel
}

/// Represents an angle value with its unit.
#[derive(Debug, PartialEq, Serialize)]
pub struct Angle {
    pub value: f64,
    pub unit: AngleUnit,
}

/// Represents a position value.
#[derive(Debug, PartialEq, Serialize)]
pub struct Position {
    pub x: Option<Box<Value>>, // e.g., "center", "10px"
    pub y: Option<Box<Value>>, // e.g., "top", "20px"
}