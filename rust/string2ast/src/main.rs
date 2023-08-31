#![allow(dead_code)]

use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenType {
    TokenUnknown,
    TokenVarIndex(usize),
    TokenComma,
    TokenAdd,
    TokenMul,
    TokenLeftBracket,
    TokenRightBracket,
}

#[derive(Debug, PartialEq, Eq)]
struct Token {
    token_type: TokenType,
    start_pos: usize,
    end_pos: usize,
}

impl Token {
    pub fn new(token_type: TokenType, start_pos: usize, end_pos: usize) -> Self {
        Self {
            token_type,
            start_pos,
            end_pos,
        }
    }
}

type Link = Rc<RefCell<TokenAST>>;

#[derive(Debug, PartialEq, Eq)]
struct TokenAST {
    token_type: Option<TokenType>,
    left: Option<Link>,
    right: Option<Link>,
}

impl TokenAST {
    pub fn new(token_type: Option<TokenType>, left: Option<Link>, right: Option<Link>) -> Self {
        Self {
            token_type,
            left,
            right,
        }
    }
}

struct ExpressionParser {
    tokens: Vec<Token>,
    tokens_str: Vec<String>,
    statement_: String,
}

impl ExpressionParser {
    pub fn new(statement_: String) -> Self {
        Self {
            tokens: Vec::new(),
            tokens_str: Vec::new(),
            statement_,
        }
    }

    pub fn tokenizer(&mut self) {
        assert!(!self.statement_.is_empty());

        let chars = self.statement_.chars().collect::<Vec<char>>();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                'a' => {
                    // parse add
                    assert!(i + 2 < chars.len());
                    assert!(chars[i + 1] == 'd');
                    assert!(chars[i + 2] == 'd');

                    self.tokens.push(Token::new(TokenType::TokenAdd, i, i + 3));
                    self.tokens_str.push(chars[i..i + 3].into_iter().collect());
                    i += 3;
                }
                'm' => {
                    // parse mul
                    assert!(i + 2 < chars.len());
                    assert!(chars[i + 1] == 'u');
                    assert!(chars[i + 2] == 'l');

                    self.tokens.push(Token::new(TokenType::TokenMul, i, i + 3));
                    self.tokens_str.push(chars[i..i + 3].into_iter().collect());
                    i += 3;
                }
                '@' => {
                    // parse operand
                    assert!(i + 1 < chars.len() && chars[i + 1].is_digit(10));
                    let mut j = i + 1;
                    while j < chars.len() {
                        if !chars[j].is_digit(10) {
                            break;
                        }
                        j += 1;
                    }
                    println!("{}", j);
                    let num = chars[i + 1..j]
                        .into_iter()
                        .collect::<String>()
                        .parse::<usize>()
                        .unwrap();
                    self.tokens
                        .push(Token::new(TokenType::TokenVarIndex(num), i, j));
                    self.tokens_str.push(chars[i..j].into_iter().collect());
                    i = j;
                }
                ',' => {
                    // parse comma
                    self.tokens
                        .push(Token::new(TokenType::TokenComma, i, i + 1));
                    self.tokens_str.push(chars[i..i + 1].into_iter().collect());
                    i += 1;
                }
                '(' => {
                    // parse (
                    self.tokens
                        .push(Token::new(TokenType::TokenLeftBracket, i, i + 1));
                    self.tokens_str.push(chars[i..i + 1].into_iter().collect());
                    i += 1;
                }
                ')' => {
                    // parse )
                    self.tokens
                        .push(Token::new(TokenType::TokenRightBracket, i, i + 1));
                    self.tokens_str.push(chars[i..i + 1].into_iter().collect());
                    i += 1;
                }
                ' ' => {
                    // skip whitespace
                    i += 1;
                }
                c => {
                    println!("Unexpected char {}", c);
                    self.dump_tokens();
                    unreachable!();
                }
            }
        }
    }

    pub fn generate(&mut self) -> Option<Link> {
        if self.tokens.is_empty() {
            panic!("Expected tokenize first");
        }
        let mut index = 0;
        self.generate_(&mut index)
    }

    // Generate sub tree
    // @brief generate ast
    // @param index, index of current token
    pub fn generate_(&mut self, index: &mut usize) -> Option<Link> {
        // get current token
        let current_token = &self.tokens[*index];
        match current_token.token_type {
            TokenType::TokenVarIndex(_) => {
                // parse index of the target variable and return
                let start_pos = current_token.start_pos + 1;
                let end_pos = current_token.end_pos;
                assert!(end_pos > start_pos);
                assert!(end_pos < self.statement_.len());

                // return a node that without children
                return Some(Rc::new(RefCell::new(TokenAST::new(
                    Some(current_token.token_type),
                    None,
                    None,
                ))));
            }
            TokenType::TokenAdd | TokenType::TokenMul => {
                // parse operator and corresponding operand
                let mut new_node = TokenAST::new(Some(current_token.token_type), None, None);

                // eat left bracket
                *index += 1;
                assert!(self.tokens[*index].token_type == TokenType::TokenLeftBracket);
                assert!(*index < self.tokens.len());

                // recursively parse operand 1
                *index += 1;
                match self.tokens[*index].token_type {
                    TokenType::TokenVarIndex(_) | TokenType::TokenAdd | TokenType::TokenMul => {
                        new_node.left = self.generate_(index);
                    }
                    _ => {
                        panic!("Expected operand after operator!");
                    }
                }

                // eat comma
                *index += 1;
                assert!(self.tokens[*index].token_type == TokenType::TokenComma);
                assert!(*index < self.tokens.len());

                // recursively parse operand 2
                *index += 1;
                match self.tokens[*index].token_type {
                    TokenType::TokenVarIndex(_) | TokenType::TokenAdd | TokenType::TokenMul => {
                        new_node.right = self.generate_(index);
                    }
                    _ => {
                        panic!("Expected operand after operator!");
                    }
                }

                // eat right bracket
                *index += 1;
                assert!(self.tokens[*index].token_type == TokenType::TokenRightBracket);
                assert!(*index < self.tokens.len());

                return Some(Rc::new(RefCell::new(new_node)));
            }
            TokenType::TokenLeftBracket
            | TokenType::TokenRightBracket
            | TokenType::TokenComma
            | TokenType::TokenUnknown => unreachable!(),
        }
    }

    pub fn dump_tokens(&self) {
        for i in 0..self.tokens_str.len() {
            println!("{:#?}", self.tokens_str[i]);
            println!("{:#?}", self.tokens[i]);
        }
    }
}

fn main() {
    print!("> ");
    io::stdout().flush().unwrap();
    for line in io::stdin().lines() {
        println!("read: {}", line.as_ref().unwrap());

        let mut e = ExpressionParser::new(line.unwrap());
        e.tokenizer();
        e.dump_tokens();
        let root = e.generate();
        println!("{:#?}", root);
        // add(   mul(@1, @2), mul(@3, @4))

        print!("\n> ");
        io::stdout().flush().unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let mut e = ExpressionParser::new("add(   mul(@1, @2), mul(@3, @4))".to_string());
        e.tokenizer();
        assert_eq!(e.tokens.len(), 16);
        assert_eq!(e.tokens_str.len(), 16);

        let expected_token = vec![
            TokenType::TokenAdd,
            TokenType::TokenLeftBracket,
            TokenType::TokenMul,
            TokenType::TokenLeftBracket,
            TokenType::TokenVarIndex(1),
            TokenType::TokenComma,
            TokenType::TokenVarIndex(2),
            TokenType::TokenRightBracket,
            TokenType::TokenComma,
            TokenType::TokenMul,
            TokenType::TokenLeftBracket,
            TokenType::TokenVarIndex(3),
            TokenType::TokenComma,
            TokenType::TokenVarIndex(4),
            TokenType::TokenRightBracket,
            TokenType::TokenRightBracket,
        ];

        let expected_token_str = vec![
            "add", "(", "mul", "(", "@1", ",", "@2", ")", ",", "mul", "(", "@3", ",", "@4", ")",
            ")",
        ];

        for i in 0..e.tokens_str.len() {
            assert_eq!(e.tokens_str[i], expected_token_str[i].to_string());
            assert_eq!(e.tokens[i].token_type, expected_token[i]);
        }
    }

    #[test]
    fn test_lexer() {
        let mut e = ExpressionParser::new("add(   mul(@1, @2), mul(@3, @4))".to_string());
        e.tokenizer();
        let root = e.generate().unwrap().clone();
        let l = root.borrow().left.clone().unwrap();
        let r = root.borrow().right.clone().unwrap();

        let ll = l.borrow().left.clone().unwrap();
        let lr = l.borrow().right.clone().unwrap();

        let rl = r.borrow().left.clone().unwrap();
        let rr = r.borrow().right.clone().unwrap();

        assert_eq!(root.borrow().token_type, Some(TokenType::TokenAdd));
        assert_eq!(l.borrow().token_type, Some(TokenType::TokenMul));
        assert_eq!(r.borrow().token_type, Some(TokenType::TokenMul));

        assert_eq!(ll.borrow().token_type, Some(TokenType::TokenVarIndex(1)));
        assert_eq!(lr.borrow().token_type, Some(TokenType::TokenVarIndex(2)));
        assert_eq!(rl.borrow().token_type, Some(TokenType::TokenVarIndex(3)));
        assert_eq!(rr.borrow().token_type, Some(TokenType::TokenVarIndex(4)));

        assert_eq!(ll.borrow().left, None);
        assert_eq!(lr.borrow().left, None);
        assert_eq!(rl.borrow().left, None);
        assert_eq!(rr.borrow().left, None);

        assert_eq!(ll.borrow().right, None);
        assert_eq!(lr.borrow().right, None);
        assert_eq!(rl.borrow().right, None);
        assert_eq!(rr.borrow().right, None);
    }
}
