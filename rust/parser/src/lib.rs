#[derive(Default)]
struct Parser {
    data: Vec<char>,
    position: usize,

    /// Infomation message for error report
    line: usize,
    /// Position in current line
    line_position: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sexp {
    /// plain String symbolic-expression
    String(String),
    /// list symbolic-expression
    List(Vec<Sexp>),
    /// empty, trivial symbolic-expression
    Empty,
}


#[derive(Debug)]
pub enum SexpError {
    String(String),
}

impl Default for Sexp {
    fn default() -> Sexp {
        Sexp::Empty
    }
}

impl Parser {
    fn peek(&self) -> Result<char, SexpError> {
        self.fail_on_eof()?;
        Ok(self.data[self.position])
    }

    /// Consume next char
    fn get(&mut self) -> Result<char, SexpError> {
        self.fail_on_eof()?;
        let c = self.data[self.position];
        self.position += 1;
        self.line_position += 1;
        if c == '\n' {
            self.line += 1;
            self.line_position = 0;
        }
        Ok(c)
    }

    /// Move current cursor to next char
    fn inc(&mut self) {
        let c = self.data[self.position];
        self.position += 1;
        self.line_position += 1;
        if c == '\n' {
            self.line += 1;
            self.line_position = 0;
        }
    }

    /// Consume all space until a non-space char or eof
    fn eat_space(&mut self) {
        while !self.eof() {
            let c = self.data[self.position];
            if c == ' ' || c == '\t' {
                self.inc();
                continue;
            }
            break;
        }
    }

    /// Consume a specific char or return error
    fn eat_char(&mut self, c: char) -> Result<(), SexpError> {
        let c2 = self.get()?;
        if c != c2 {
            self.parse_error(&format!("Expect {} but found {}", c, c2))
        } else {
            Ok(())
        }
    }

    fn fail_on_eof(&self) -> Result<(), SexpError> {
        if self.eof() {
            return self.parse_error("EOF")
        }
        Ok(())
    }

    fn eof(&self) -> bool {
        return self.position >= self.data.len()
    }

    fn parse_error<T>(&self, msg: &str) -> Result<T, SexpError> {
        Err(SexpError::String(msg.to_string()))
    }

}

pub fn parse_str(sexp: &str) -> Result<Sexp, SexpError> {
    if sexp.is_empty() {
        return Ok(Sexp::default());
    }
    let mut parser = Parser::default();
    parser.data = sexp.chars().collect();
    parse(&mut parser)
}

fn parse(parser: &mut Parser) -> Result<Sexp, SexpError> {
    parser.eat_space();
    let c = parser.peek()?;
    if c == '(' {
        parse_list(parser)
    } else if c == '"' {
        parse_quoted_string(parser)
    } else if c == ')' {
        parser.parse_error("Unexpected )")
    } else {
        parse_bare_string(parser)
    }
}

/// Expected pattern: ( expr )
fn parse_list(parser: &mut Parser) -> Result<Sexp, SexpError> {
    let mut v = vec![];
    // expect fixed pattern (
    parser.eat_char('(')?;
    while !parser.eof() {
        let c = parser.peek()?;
        if c == ')' {
            break;
        } else if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
            parser.inc();
        } else {
            let s = parse(parser)?;
            v.push(s)
        }
    }
    // expect fixed pattern )
    parser.eat_char(')')?;
    parser.eat_space();
    Ok(Sexp::List(v))
}

/// Expected pattern: " expr "
fn parse_quoted_string(parser: &mut Parser) -> Result<Sexp, SexpError> {
    let mut s = String::new();
    // expect fixed pattern ", left quote
    parser.eat_char('"')?;
    // \" to escape next "
    let mut escape = false;
    while !parser.eof() {
        let c = parser.peek()?;
        println!("{}", c);
        if c == '\\' {
            escape = true;
        } else if c == '"' {
            if !escape {
                break;
            } else {
                // TODO: recursively parse quoted string
                escape = false;
            }
        } else {
            escape = false;
        }
        s.push(c);
        parser.inc();
    }

    // expect fixed pattern ", right quote
    parser.eat_char('"')?;
    Ok(Sexp::String(s))
}

fn parse_bare_string(parser: &mut Parser) -> Result<Sexp, SexpError> {
    let mut s = String::new();
    while !parser.eof() {
        let c = parser.peek()?;
        if c == ' ' || c == '(' || c == ')' || c == '\r' || c == '\n' {
            break;
        }
        s.push(c);
        parser.inc();
    }
    Ok(Sexp::String(s))
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_int() {
        let s = "(a (b c) (d \"42\"))";
        let s = parse_str(s).unwrap();
        println!("{:#?}", s);

        let a = Sexp::List(vec![
            Sexp::String("a".to_string()),
            Sexp::List(vec![Sexp::String("b".to_string()), 
                            Sexp::String("c".to_string())]),
            Sexp::List(vec![Sexp::String("d".to_string()), 
                            Sexp::String("42".to_string())]),
        ]);
        assert_eq!(a, s);
    }
}

