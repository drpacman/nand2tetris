use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::iter::{ Iterator, Peekable };

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

trait CompilationEngine {
    fn compile_class(&mut self);
    fn compile_class_var_dec(&mut self);
    fn compile_subroutine(&mut self);
    fn compile_param_list(&mut self);
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

pub struct XMLCompilationEngine<T> where T:Iterator {
    tokens : Peekable<T>
}

impl<T : Iterator<Item=Token>> XMLCompilationEngine<T> {
    const binary_op :&'static [char] = &['+','-','*','/','&','|','<','>','='];
    const unary_op :&'static [char] = &['-','~'];

    pub fn new(tokens : Peekable<T>) -> XMLCompilationEngine<T> {
        XMLCompilationEngine { tokens }
    }

    fn process_keyword(&mut self, value : KeyWord){
        match self.tokens.next() {
            Some(Token::KeyWord(k)) if k == value => {
                println!("<keyword> {:?} </keyword>", value);  
            },
            invalid => panic!("Parser error - expected keyword {:?}, got {:?}", value, invalid)
        }
    }

    fn process_symbol(&mut self, value : char){
        match self.tokens.next() {
            Some(Token::Symbol(c)) if c == value => {
                println!("<symbol> {} </symbol>", value);  
            },
            invalid => panic!("Parser error - expected symbol {}, got {:?}", value, invalid)
        }
    }

    fn process_identifier(&mut self){
        match self.tokens.next() {
            Some(Token::Identifier(id)) => {
                println!("<identifier> {} </identifier>", id);  
            },
            invalid => panic!("Parser error - expected identifier, got {:?}", invalid)
        }
    }
    
    pub fn compile_class(&mut self){
        println!("<class>");
        self.process_keyword(KeyWord::Class);
        self.process_identifier();
        self.process_symbol('{');
        loop {
            match self.tokens.peek() {
                Some(Token::KeyWord(KeyWord::Static)) |
                Some(Token::KeyWord(KeyWord::Field)) 
                    => self.compile_class_var_dec(),
                Some(Token::KeyWord(KeyWord::Constructor)) |
                Some(Token::KeyWord(KeyWord::Function)) | 
                Some(Token::KeyWord(KeyWord::Method))
                    => self.compile_subroutine(),
                Some(Token::Symbol('}'))
                    => {
                        self.process_symbol('}');
                        break;
                    },
                err => panic!("Unexpected class {:?}", err)
            }
        }
        println!("</class>");                        
    }
    
    fn compile_class_var_dec(&mut self){
        match self.tokens.peek() {
            Some(Token::KeyWord(KeyWord::Field)) | Some(Token::KeyWord(KeyWord::Static)) => {
                println!("<keyword> {:?} </keyword>", self.tokens.next().unwrap())
            },
            err => panic!("invalid class var dec {:?}", err)
        } 

        self.compile_type_dec();
    }

    fn compile_var_dec(&mut self) {
        while let Some(Token::KeyWord(KeyWord::Var)) = self.tokens.peek() {
            println!("<varDec>");
            self.process_keyword(KeyWord::Var);
            self.compile_type_dec();
            println!("</varDec>");        
        }
    }
    
    fn compile_type_dec(&mut self) {
        self.compile_type();
        self.process_identifier();
        while let Some(Token::Symbol(',')) = self.tokens.peek() {
            self.process_symbol(',');
            self.process_identifier();        
        }
        self.process_symbol(';'); 
    }

    fn compile_type(&mut self) {
        let type_token = self.tokens.next();
        match type_token {
            Some(Token::KeyWord(k)) 
                if k == KeyWord::Int || k == KeyWord::Char || k==KeyWord::Boolean  
                    => { println!("<keyword> {:?} </keyword>", k) },
            Some(Token::Identifier(id)) => println!("<identifier> {} </identifier>", id),
            _ => panic!("missing type token")
        }      
    }

        
    fn compile_subroutine(&mut self){
        println!("<subroutineDec>");
        if let Some(Token::KeyWord(k)) = self.tokens.next() {
            println!("<keyword> {:?} </keyword>", k);
            match self.tokens.next().unwrap() {
                Token::KeyWord(k) => println!("<keyword> {:?} </keyword>", k),
                Token::Identifier(i) => println!("<identifier> {} </identifier>", i),
                err => panic!("Unexpected token for subroutine type {:?}", err)
            }
            self.process_identifier();
            self.process_symbol('(');
            self.compile_parameter_list();
            self.process_symbol(')');
            self.compile_subroutine_body(); 
            println!("</subroutineDec>");   
        } else {
            panic!("Missing keyword for subroutine dec");
        }       
    }
    
    fn compile_parameter_list(&mut self){
        println!("<parameterList>");
        loop {
            match self.tokens.peek() {
                Some(Token::Symbol(')')) => break,
                Some(Token::Symbol(',')) => self.process_symbol(','),
                _ => {
                    self.compile_type();
                    println!("<identifier> {:?} </identifier>", self.tokens.next().unwrap());                    
                }
            }
        }
        println!("</parameterList>");        
    }

    fn compile_statements(&mut self){
        println!("<statements>");  
        loop {
            match self.tokens.peek() {
                Some(Token::KeyWord(KeyWord::Let)) => self.compile_let(),
                Some(Token::KeyWord(KeyWord::Do)) => self.compile_do(),
                Some(Token::KeyWord(KeyWord::While)) => self.compile_while(),
                Some(Token::KeyWord(KeyWord::If)) => self.compile_if(),
                Some(Token::KeyWord(KeyWord::Return)) => self.compile_return(),
                Some(Token::Symbol('}')) => break,
                _ => panic!("Missing statement token - found {:?}", self.tokens.peek())           
            }
        }
        println!("</statements>");        
    }

    fn compile_subroutine_body(&mut self){
        println!("<subroutineBody>");                
        self.process_symbol('{');
        self.compile_var_dec();
        self.compile_statements();
        self.process_symbol('}');
        println!("</subroutineBody>");                
    }

    fn compile_do(&mut self){
        println!("<doStatement>");  
        self.process_keyword(KeyWord::Do);
        self.process_identifier();        
        match self.tokens.peek().unwrap() {
            Token::Symbol(s) if *s == '(' => {
                self.process_symbol('(');
                self.compile_expression_list();
                self.process_symbol(')');                        
            },
            Token::Symbol(s) if *s == '.' => {
                self.process_symbol('.');
                self.process_identifier();        
                self.process_symbol('(');
                self.compile_expression_list();
                self.process_symbol(')');                        
            }
            _ => {
                panic!("Unexpected subroutine call token {:?}", self.tokens.peek())
            }
        }
        self.process_symbol(';'); 
        println!("</doStatement>");                 
    }    

    fn compile_if(&mut self){
        self.process_keyword(KeyWord::If);
        self.process_symbol('(');
        // compile_expression(self);
        self.process_symbol(')');
        self.process_symbol('{');
        self.compile_statements();
        self.process_symbol('}');
    }

    fn compile_let(&mut self){
        println!("<letStatement>");  
        self.process_keyword(KeyWord::Let);
        self.process_identifier();        
        if let Some(Token::Symbol('[')) = self.tokens.peek() {
            self.process_symbol('[');
            self.compile_expression();
            self.process_symbol(']');
        }
        self.process_symbol('=');
        self.compile_expression();
        self.process_symbol(';');
        println!("</letStatement>");
    }

    fn compile_while(&mut self){
        println!("<whileStatement>");  
        self.process_keyword(KeyWord::While);
        self.process_symbol('(');
        self.compile_expression();
                
        self.process_symbol(')');
        self.process_symbol('{');
        self.compile_statements();
        self.process_symbol('}');
        println!("</whileStatement>");          
    }    

    fn compile_return(&mut self){
        println!("<returnStatement>");
        self.process_keyword(KeyWord::Return);
        match self.tokens.peek() {
            Some(Token::Symbol(c)) if *c == ';' => {
                self.process_symbol(';');
            },
            _ => {
                self.compile_expression();
                self.process_symbol(';');
            }
        }
        println!("</returnStatement>");        
    }   

    fn compile_expression_list(&mut self) {
        println!("<expressionList>");
        loop {
            if let Some(Token::Symbol(')')) = self.tokens.peek() {
                break
            }
            self.compile_expression();
            if let Some(Token::Symbol(',')) = self.tokens.peek() {
                self.process_symbol(',');
            }
        }
        println!("</expressionList>");
        
    }
    
    fn compile_expression(&mut self) {
        println!("<expression>");
        self.compile_term();
        match self.tokens.peek() {
            Some(Token::Symbol(c)) if XMLCompilationEngine::<T>::binary_op.contains(c) => {
                if *c == '<' {
                    println!("<symbol> &lt; </symbol>");   
                } else if *c == '>' {
                    println!("<symbol> &gt; </symbol>");   
                } else {
                    println!("<symbol> {} </symbol>", c);   
                }
                self.tokens.next();        
                self.compile_term();
            },
            _ => {}
        }
        println!("</expression>");        
    }

    fn compile_term(&mut self) {
        println!("<term>");
        let token = self.tokens.next().unwrap();
        match token {
            Token::Int(i) => { println!("<integerConstant> {:?} </integerConstant>", i);},
            Token::String(s) => { println!("<stringConstant> {:?} </stringConstant>", s);},
            Token::KeyWord(k) => { println!("<keyword> {:?} </keyword>", k); },
            Token::Symbol('(') => { 
                self.process_symbol('(');
                self.compile_expression();
                self.process_symbol(')');  
            },
            Token::Symbol(s) if XMLCompilationEngine::<T>::unary_op.contains(&s) => {
                println!("<symbol> {} </symbol>", s);
                self.compile_term();
            },            
            Token::Identifier(id) => {
                println!("<identifier> {} </identifier>", id);                        
                match self.tokens.peek().unwrap() {
                    Token::Symbol(s) if *s == '[' => {
                        self.process_symbol('[');
                        self.compile_expression();
                        self.process_symbol(']');                        
                    },
                    Token::Symbol(s) if *s == '(' => {
                        self.process_symbol('(');
                        self.compile_expression_list();
                        self.process_symbol(')');                        
                    },
                    Token::Symbol(s) if *s == '.' => {
                        println!("<symbol> . </symbol>");
                        self.tokens.next();
                        self.process_identifier();        
                        self.process_symbol('(');
                        self.compile_expression_list();
                        self.process_symbol(')');   
                    },
                    _ => {}
                }
            },
            _ => {
                panic!("Unexpected token {:?}", token)
            }
        }
        println!("</term>");        
    }
}