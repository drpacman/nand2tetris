use std::fs::File;
use std::io::Read;
use std::iter::{ Iterator };

pub struct JackTokenizer {
    filename : String,
    block_comment : bool
}
#[derive(Debug)]
pub enum Token {
    KeyWord(KeyWord),
    Symbol(char),
    Identifier(String),
    Int(i16),
    String(String)
}

#[derive(Debug, PartialEq)]
pub enum KeyWord {
    Class, Constructor, Function, Method, Field,
    Static, Var, Int, Char, Boolean, Void,
    True, False, Null, This,
    Let, Do, If, Else, While, Return
}

impl JackTokenizer {

    pub fn new(filename : &str ) -> JackTokenizer {
        JackTokenizer { filename: filename.to_string(), block_comment: false }
    }

    pub fn parse(&mut self) -> Result<Vec<Token>,std::io::Error> {
        let mut f = File::open(&self.filename).expect("Missing file");
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let tokens = contents.to_string()
                            .split('\n')
                            .map(|s| self.parse_line(s))
                            .flatten().collect();
        Ok(tokens)
    }

    fn tokenize_string(text : String) -> Token {                            
        let numbers = vec!['0','1','2','3','4','5','6','7','8','9'];
        if numbers.contains(&text.chars().next().unwrap()) {
            Token::Int( text.parse::<i16>().unwrap() )
        } else {
            match text.as_str() {
                "class" => Token::KeyWord(KeyWord::Class),
                "constructor" => Token::KeyWord(KeyWord::Constructor),
                "function" => Token::KeyWord(KeyWord::Function),
                "method" => Token::KeyWord(KeyWord::Method),
                "field" => Token::KeyWord(KeyWord::Field),
                "static" => Token::KeyWord(KeyWord::Static),
                "var" => Token::KeyWord(KeyWord::Var),
                "int" => Token::KeyWord(KeyWord::Int),
                "char" => Token::KeyWord(KeyWord::Char),
                "boolean" => Token::KeyWord(KeyWord::Boolean),
                "void" => Token::KeyWord(KeyWord::Void),
                "true" => Token::KeyWord(KeyWord::True),
                "false" => Token::KeyWord(KeyWord::False),
                "null" => Token::KeyWord(KeyWord::Null),
                "this" => Token::KeyWord(KeyWord::This),
                "let" => Token::KeyWord(KeyWord::Let),
                "do" => Token::KeyWord(KeyWord::Do),
                "if" => Token::KeyWord(KeyWord::If),
                "else" => Token::KeyWord(KeyWord::Else),
                "while" => Token::KeyWord(KeyWord::While),
                "return" => Token::KeyWord(KeyWord::Return),
                _ => {
                    Token::Identifier( text )
                }
            }
        }
    }
    
    fn parse_line(&mut self, line : &str) -> Vec<Token> {
        let mut tokens : Vec<Token> = Vec::new();
        let mut text = Vec::new();
        let mut is_str = false;
        let mut c_last : char = '0';
        for c in line.chars() {
            if self.block_comment {
                if c == '/' && c_last == '*' {
                    self.block_comment = false;
                } 
            } else {
                match c {
                    ' ' | '\t'  if !is_str => { 
                        if text.len() > 0 {
                            tokens.push(JackTokenizer::tokenize_string(text.clone().into_iter().collect()));
                        }
                        text.clear();                 
                    },
                    '{' | '}' | '(' | ')' | '[' | ']' | '.' | ',' | ';' | '+' | '-' | '*' | '/' | '&' | '|' | '<' | '>' | '=' | '~' => {
                        if text.len() > 0 {
                            tokens.push(JackTokenizer::tokenize_string(text.clone().into_iter().collect()));
                        }
                        text.clear();
                        if c == '/' {
                            if c_last == '/' {
                                // rest of line is commented out - return what we have, removing the first /
                                tokens.pop();
                                return tokens
                            } else {
                                tokens.push(Token::Symbol(c));                        
                            } 
                        } else if c == '*' && c_last == '/' {
                            tokens.pop();
                            self.block_comment = true;
                        } else {
                            tokens.push(Token::Symbol(c));                        
                        }                                 
                    },
                    '"' => {
                        if is_str {
                            tokens.push(Token::String( text.clone().into_iter().collect() ));
                            text.clear();
                            is_str = false;
                        } else {
                            text.clear();
                            is_str = true;
                        }
                    }
                    _ => {
                        text.push(c);
                    }
                }
            }
            c_last = c;
        }
        tokens
    }
}

pub trait CompilationEngine {
    fn compile_class(&mut self);
    fn compile_class_var_dec(&mut self);
    fn compile_subroutine(&mut self);
    fn compile_parameter_list(&mut self);
    fn compile_subroutine_body(&mut self);
    fn compile_var_dec(&mut self);
    fn compile_statements(&mut self);
    fn compile_if(&mut self);
    fn compile_let(&mut self);
    fn compile_while(&mut self);
    fn compile_do(&mut self);
    fn compile_return(&mut self);
    fn compile_expression(&mut self);
    fn compile_term(&mut self);
    fn compile_expression_list(&mut self) -> u8;
}