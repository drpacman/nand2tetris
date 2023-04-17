
use crate::compiler;
use std::iter::{ Iterator, Peekable };
use compiler::{Token, KeyWord};
use std::fs::File;
use std::io::prelude::*;

pub struct XMLWriter {
    filename : String
}

impl XMLWriter {
    pub fn new(filestem : &str) -> XMLWriter {
        XMLWriter { 
            filename:  format!("{}.xml", filestem)
        }
    }

    pub fn write(&self, ins : &Vec<String>) {
        let mut f = File::create(&self.filename).expect("unable to create file");
        write!(f, "{}", ins.join("\r")).expect("Failed to write XML to file")
    }
}

pub struct XMLCompilationEngine<T> where T:Iterator {
    tokens : Peekable<T>,
    output: Vec<String>
}


impl<T: Iterator<Item=Token>> XMLCompilationEngine<T> {

    pub fn new(tokens : Peekable<T>) -> XMLCompilationEngine<T> {
        XMLCompilationEngine { 
            tokens,
            output : vec![] 
        }
    }

    fn add_node(&mut self, node : &str, value: String){
        self.output.push(format!("<{}> {} </{}>", node, value, node));
    }

    fn add_keyword_node(&mut self, keyword: KeyWord){
        self.output.push(format!("<keyword> {:?} </keyword>", keyword));
    }

    fn open_node(&mut self, node : &str){
        self.output.push(format!("<{}>", node));
    }

    fn close_node(&mut self, node : &str){
        self.output.push(format!("</{}>", node));
    }

    fn process_keyword(&mut self, value : KeyWord){
        match self.tokens.next() {
            Some(Token::KeyWord(k)) if k == value => {
                self.add_keyword_node(value);
            },
            invalid => panic!("Parser error - expected keyword {:?}, got {:?}", value, invalid)
        }
    }

    fn process_symbol(&mut self, value : char){
        match self.tokens.next() {
            Some(Token::Symbol(c)) if c == value => {
                self.add_node("symbol", value.to_string());
            },
            invalid => panic!("Parser error - expected symbol {}, got {:?}", value, invalid)
        }
    }

    fn process_identifier(&mut self){
        match self.tokens.next() {
            Some(Token::Identifier(id)) => {
                self.add_node("identifier", id);
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
                    => {  self.add_keyword_node(k) },
            Some(Token::Identifier(id)) => self.add_node("identifier", id),
            _ => panic!("missing type token")
        }      
    }
}

impl<T: Iterator<Item=Token>> compiler::CompilationEngine<String> for XMLCompilationEngine<T> {
    
    fn compile_class(&mut self) -> Vec<String> {
        self.open_node("class");
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
        self.close_node("class");
        self.output.clone()                             
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
            self.open_node("varDec");      
            self.process_keyword(KeyWord::Var);
            self.compile_type_dec();
            self.close_node("varDec");      
        }
    }
        
    fn compile_subroutine(&mut self){
        if let Some(Token::KeyWord(k)) = self.tokens.next() {
            self.open_node("subroutineDec");      
            self.add_keyword_node(k);
            match self.tokens.next().unwrap() {
                Token::KeyWord(k) => self.add_keyword_node(k),
                Token::Identifier(i) => self.add_node("identifier", i),
                err => panic!("Unexpected token for subroutine type {:?}", err)
            }
            self.process_identifier();
            self.process_symbol('(');
            self.compile_parameter_list();
            self.process_symbol(')');
            self.compile_subroutine_body(); 
            self.close_node("subroutineDec");             
        } else {
            panic!("Missing keyword for subroutine dec");
        }       
    }    

    fn compile_parameter_list(&mut self){
        self.open_node("parameterList");                  
        loop {
            match self.tokens.peek() {
                Some(Token::Symbol(')')) => break,
                Some(Token::Symbol(',')) => self.process_symbol(','),
                _ => {
                    self.compile_type();
                    if let Some(Token::Identifier(id)) = self.tokens.next() {
                        self.add_node("identifier", id);  
                    } else {
                        panic!("Missing identifier for param list")
                    }
                }
            }
        }
        self.close_node("parameterList");                  
    }

    fn compile_statements(&mut self){
        self.open_node("statements");                  
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
        self.close_node("statements");                                
    }

    fn compile_subroutine_body(&mut self){
        self.open_node("subroutineBody");                
        self.process_symbol('{');
        self.compile_var_dec();
        self.compile_statements();
        self.process_symbol('}');
        self.close_node("subroutineBody");                
    }

    fn compile_do(&mut self){
        self.open_node("doStatement");  
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
        self.close_node("doStatement");          
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
        self.open_node("letStatement");
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
        self.close_node("letStatement");
    }

    fn compile_while(&mut self){
        self.open_node("whileStatement");
        self.process_keyword(KeyWord::While);
        self.process_symbol('(');
        self.compile_expression();
                
        self.process_symbol(')');
        self.process_symbol('{');
        self.compile_statements();
        self.process_symbol('}');
        self.close_node("whileStatement");                
    }    

    fn compile_return(&mut self){
        self.open_node("returnStatement");
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
        self.close_node("returnStatement");        
    }   

    fn compile_expression_list(&mut self) -> u8 {
        self.open_node("expressionList");        
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
        self.close_node("expressionList");        
        expression_count      
    }
    
    fn compile_expression(&mut self) {
        self.open_node("expression");        
        self.compile_term();
        match self.tokens.peek() {
            Some(Token::Symbol(c)) if XMLCompilationEngine::<T>::BINARY_OP.contains(c) => {
                if *c == '<' {
                    self.add_node("symbol", "&lt;".to_string());
                } else if *c == '>' {
                    self.add_node("symbol", "&gt;".to_string());  
                } else {
                    let symbol = c.to_string();
                    self.add_node("symbol", symbol);
                }
                self.tokens.next();        
                self.compile_term();
            },
            _ => {}
        }
        self.close_node("expression");                       
    }

    fn compile_term(&mut self) {
        self.open_node("term");
        let token = self.tokens.next().unwrap();
        match token {
            Token::Int(i) => self.add_node("integerConstant", i.to_string()),
            Token::String(s) => self.add_node("stringConstant", s),
            Token::KeyWord(k) => self.add_keyword_node(k),
            Token::Symbol('(') => { 
                self.compile_expression();
                self.process_symbol(')');  
            },
            Token::Symbol(s) if XMLCompilationEngine::<T>::UNARY_OP.contains(&s) => {
                self.add_node("symbol", s.to_string());
                self.compile_term();
            },            
            Token::Identifier(id) => {
                self.add_node("identifier", id);                        
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
                        let symbol = s.to_string();
                        self.add_node("symbol", symbol);
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
                panic!("Unexpected token {:?} - instruction were {:?}", token, self.output)

            }
        }
        self.close_node("term");        
    }
}