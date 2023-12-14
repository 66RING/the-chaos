// h1, h2, h3 { margin: auto; color: #cc0000; }
// div.note { margin-bottom: 20px; padding: 10px; }
// #answer { display: none; }

/// A CSS stylesheet is a series of rules.
/// (In the example stylesheet above, each line contains one rule.)
#[derive(Debug)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

/// A rule includes one or more selectors separated by commas, followed by a series of declarations enclosed in braces.
#[derive(Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

/// TODO: Supports only simple selectors for now.
#[derive(Debug, PartialEq)]
pub enum Selector {
    Simple(SimpleSelector),
}

#[derive(Debug, PartialEq)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

/// A declaration is just a name/value pair, separated by a colon and ending with a semicolon.
#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

/// Toy engine supports only a handful of CSS’s many value types.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
    // insert more values here
}

impl Value {
    /// Return the size of a length in px, or zero for non-lengths.
    pub fn to_px(&self) -> f32 {
        match *self {
            Value::Length(f, Unit::Px) => f,
            _ => 0.0
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unit {
    Px,
    // insert more units here
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Specificity is one of the ways a rendering engine decides which style overrides the other in a conflict. If a stylesheet contains two rules that match an element, the rule with the matching selector of higher specificity can override values from the one with lower specificity.
/// An ID selector is more specific than a class selector, which is more specific than a tag selector. Within each of these “levels,” more selectors beats fewer.
pub type Specificity = (usize, usize, usize);

impl Selector {
    /// Return id, class, tag_name
    pub fn specificity(&self) -> Specificity {
        // http://www.w3.org/TR/selectors/#specificity
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a, b, c)
    }
}

pub struct Parser {
    pos: usize, // "usize" is an unsigned integer, similar to "size_t" in C
    input: String,
}

/// Parse a whole CSS stylesheet.
pub fn parse(source: String) -> Stylesheet {
    let mut parser = Parser { pos: 0, input: source };
    Stylesheet { rules: parser.parse_rules() }
}


impl Parser {
    /// Parse a list of rule sets, separated by optional whitespace.
    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() { break }
            rules.push(self.parse_rule());
        }
        rules
    }

    /// Parse a rule set: `<selectors> { <declarations> }`.
    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    /// Parse a comma-separated list of selectors.
    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => { self.consume_char(); self.consume_whitespace(); }
                '{' => break,
                c   => panic!("Unexpected character {} in selector list", c)
            }
        }
        // Return selectors with highest specificity first, for use in matching.
        selectors.sort_by(|a,b| b.specificity().cmp(&a.specificity()));
        selectors
    }

    /// Parse one simple selector, e.g.: `type#id.class1.class2.class3`
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector { tag_name: None, id: None, class: Vec::new() };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    // universal selector
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break
            }
        }
        selector
    }

    /// Parse a list of declarations enclosed in `{ ... }`.
    fn parse_declarations(&mut self) -> Vec<Declaration> {
        assert_eq!(self.consume_char(), '{');
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    /// Parse one `<property>: <value>;` declaration.
    fn parse_declaration(&mut self) -> Declaration {
        let property_name = self.parse_identifier();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ';');

        Declaration {
            name: property_name,
            value: value,
        }
    }

    // Methods for parsing values:

    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier())
        }
    }

    fn parse_length(&mut self) -> Value {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        let s = self.consume_while(|c| match c {
            '0'..='9' | '.' => true,
            _ => false
        });
        s.parse().unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            _ => panic!("unrecognized unit")
        }
    }

    fn parse_color(&mut self) -> Value {
        assert_eq!(self.consume_char(), '#');
        Value::ColorValue(Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255 })
    }

    /// Parse two hexadecimal digits.
    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos .. self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    /// Parse a property name or keyword.
    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    /// Consume and discard zero or more whitespace characters.
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    /// Consume characters until `test` returns false.
    fn consume_while<F>(&mut self, test: F) -> String
            where F: Fn(char) -> bool {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    /// Return the current character, and advance self.pos to the next character.
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    /// Read the current character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    /// Return true if all input is consumed.
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

fn valid_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true, // TODO: Include U+00A0 and higher.
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn css_parser_should_work() {
        let source = String::from(
            r#"
            h1, h2, h3 { margin: auto; color: #cc0000; }
            div.note { margin-bottom: 20px; padding: 10px; }
            #answer { display: none; height: 100px; }
            "#);
        let stylesheet = parse(source);
        assert_eq!(stylesheet.rules.len(), 3);
        assert_eq!(stylesheet.rules[0].selectors.len(), 3);
        assert_eq!(stylesheet.rules[0].selectors[0], Selector::Simple(SimpleSelector {
            tag_name: Some(String::from("h1")),
            id: None,
            class: vec![]
        }));
        assert_eq!(stylesheet.rules[0].declarations.len(), 2);
        assert_eq!(stylesheet.rules[0].declarations[0], Declaration {
            name: String::from("margin"),
            value: Value::Keyword(String::from("auto"))
        });
        assert_eq!(stylesheet.rules[0].declarations[1], Declaration {
            name: String::from("color"),
            value: Value::ColorValue(Color { r: 204, g: 0, b: 0, a: 255 })
        });

        assert_eq!(stylesheet.rules[1].selectors.len(), 1);
        assert_eq!(stylesheet.rules[1].selectors[0], Selector::Simple(SimpleSelector {
            tag_name: Some(String::from("div")),
            id: None,
            class: vec![String::from("note")]
        }));
        assert_eq!(stylesheet.rules[1].declarations.len(), 2);
        assert_eq!(stylesheet.rules[1].declarations[0], Declaration {
            name: String::from("margin-bottom"),
            value: Value::Length(20.0, Unit::Px)
        });
        assert_eq!(stylesheet.rules[1].declarations[1], Declaration {
            name: String::from("padding"),
            value: Value::Length(10.0, Unit::Px)
        });

        assert_eq!(stylesheet.rules[2].selectors.len(), 1);
        assert_eq!(stylesheet.rules[2].selectors[0], Selector::Simple(SimpleSelector {
            tag_name: None,
            id: Some(String::from("answer")),
            class: vec![]
        }));
        assert_eq!(stylesheet.rules[2].declarations.len(), 2);
        assert_eq!(stylesheet.rules[2].declarations[0], Declaration {
            name: String::from("display"),
            value: Value::Keyword(String::from("none"))
        });
        assert_eq!(stylesheet.rules[2].declarations[1], Declaration {
            name: String::from("height"),
            value: Value::Length(100.0, Unit::Px)
        });

        assert_eq!(stylesheet.rules[0].selectors[0].specificity(), (0, 0, 1));
        assert_eq!(stylesheet.rules[0].selectors[1].specificity(), (0, 0, 1));
        assert_eq!(stylesheet.rules[0].selectors[2].specificity(), (0, 0, 1));
        assert_eq!(stylesheet.rules[1].selectors[0].specificity(), (0, 1, 1));
        assert_eq!(stylesheet.rules[2].selectors[0].specificity(), (1, 0, 0));
    }
}
