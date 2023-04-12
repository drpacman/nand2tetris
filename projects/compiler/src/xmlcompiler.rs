
use crate::compiler;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::iter::{ Iterator, Peekable };

use compiler::{Token, KeyWord};

pub struct XMLCompilationEngine<T> where T:Iterator {
    tokens : Peekable<T>
}

impl<T: Iterator<Item=Token>> XMLCompilationEngine<T> {
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
}

impl<T: Iterator<Item=Token>> compiler::CompilationEngine for XMLCompilationEngine<T> {
    
    fn compile_class(&mut self){
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
        self.compile_expression();
        self.process_symbol(')');
        self.process_symbol('{');
        self.compile_statements();
        self.process_symbol('}');
        if let Some(Token::KeyWord(KeyWord::Else)) = self.tokens.peek() {
            self.process_keyword(KeyWord::Else);
            self.process_symbol('{');
            self.compile_statements();
            self.process_symbol('}');
        }                
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

    fn compile_expression_list(&mut self) -> u8 {
        println!("<expressionList>");
        let mut expression_count = 0;
        loop {
            if let Some(Token::Symbol(')')) = self.tokens.peek() {
                break
            }
            self.compile_expression();
            expression_count+=1;
            if let Some(Token::Symbol(',')) = self.tokens.peek() {
                self.process_symbol(',');
            }
        }
        println!("</expressionList>");  
        expression_count      
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