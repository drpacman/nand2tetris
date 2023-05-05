use crate::compiler;
use std::{iter::Peekable, collections::HashMap };
use compiler::{KeyWord, Token, CompilationEngine};
use vmtranslator::VMInstruction;
use std::fs::File;
use std::io::prelude::*;

pub struct VMCompiler<T> where T:Iterator {
    tokens : Peekable<T>,
    class_symbol_table : HashMap<String, SymbolTableEntry>,
    subroutine_symbol_table : HashMap<String, SymbolTableEntry>,
    static_symbol_counter: u8,
    field_symbol_counter: u8,
    arg_symbol_counter: u8,    
    local_symbol_counter: u8,
    class_name : Option<String>,
    instruction_buffer : Vec<VMInstruction>,
    instructions : Vec<VMInstruction>,
    while_counter : u32,
    if_counter: u32
}

pub struct VMWriter {
    dir : String
}

impl VMWriter {
    pub fn new(dir : &str) -> VMWriter {
        VMWriter { 
            dir:  dir.to_string()
        }
    }

    pub fn write(&self, filestem:&str, ins : &Vec<VMInstruction>) {
        self.write_file(format!("{}.vm", filestem).as_str(), ins);
    }

    fn write_file(&self, filename:&str, ins : &Vec<VMInstruction>) {
        let mut f = File::create(format!("{}/{}", self.dir, filename).as_str()).expect("unable to create file");
        let asm : Vec<String> = ins.iter().map(|i| i.to_string()).collect();
        write!(f, "{}", asm.join("\r")).expect("Failed to write VM instructions to file")
    }
}

impl<T: Iterator<Item=Token>> VMCompiler<T> {

    pub fn new(tokens : Peekable<T>) -> VMCompiler<T> {
        VMCompiler { 
            tokens, 
            class_symbol_table: HashMap::new(), 
            subroutine_symbol_table: HashMap::new(),
            static_symbol_counter: 0,
            field_symbol_counter: 0,
            arg_symbol_counter: 0,    
            local_symbol_counter: 0,
            class_name : None,
            instruction_buffer : vec![],
            instructions : vec![],       
            while_counter: 0,
            if_counter: 0
        }
    }

    fn process_identifier(&mut self) -> String {
        match self.tokens.next() {
            Some(Token::Identifier(id)) => {
                return id  
            },
            invalid => panic!("Parser error - expected identifier, got {:?}", invalid)
        }
    }    
    
    fn consume_symbol(&mut self, value : char){
        match self.tokens.next() {
            Some(Token::Symbol(c)) if c == value => {}
            invalid => panic!("Parser error - expected symbol {}, got {:?}", value, invalid)
        }
    }

    fn consume_keyword(&mut self, keyword : KeyWord) {
        match self.tokens.next() {
            Some(Token::KeyWord(k)) if k == keyword => {}
            invalid => panic!("Parser error - expected keyword, got {:?}", invalid)
        }
    }    
    
    fn lookup_symbol(&self, name: &String) -> Option<&SymbolTableEntry> {
        if self.subroutine_symbol_table.contains_key(name) {
            self.subroutine_symbol_table.get(name)
        } else {
            self.class_symbol_table.get(name)
        }
    }
    
    fn register_class_symbol(&mut self, kind: String, field_type: String, name: String){
        if self.class_symbol_table.get(&name).is_none() {
            match kind.as_str() {
                "static" => {
                    self.class_symbol_table.insert(name, SymbolTableEntry { symbol_type: field_type, kind: "static".to_string(), index: self.static_symbol_counter });
                    self.static_symbol_counter+=1;                
                },
                "field" => {
                    self.class_symbol_table.insert(name, SymbolTableEntry { symbol_type: field_type, kind: "this".to_string(), index: self.field_symbol_counter });
                    self.field_symbol_counter+=1;                
                },
                err => panic!("Unexpected registration kind for class {:?}", err)
            }
        }
    }

    fn register_subroutine_symbol(&mut self, kind: String, field_type: String, name: String){
        if self.subroutine_symbol_table.get(&name).is_none() {
            match kind.as_str() {
                "local" => {
                    self.subroutine_symbol_table.insert(name, SymbolTableEntry { symbol_type: field_type, kind: "local".to_string(), index: self.local_symbol_counter});
                    self.local_symbol_counter+=1;                
                },
                "argument" => {
                    self.subroutine_symbol_table.insert(name, SymbolTableEntry { symbol_type: field_type, kind: "argument".to_string(), index: self.arg_symbol_counter });
                    self.arg_symbol_counter+=1;                
                },
                err => panic!("Unexpected registration kind for subroutine {:?}", err)
            }           
        }
    }

    fn compile_type(&mut self) -> String {
        let type_token = self.tokens.next();
        match type_token {
            Some(Token::KeyWord(k)) if k == KeyWord::Int => "int".to_string(),
            Some(Token::KeyWord(k)) if k == KeyWord::Char => "char".to_string(),
            Some(Token::KeyWord(k)) if k == KeyWord::Boolean => "boolean".to_string(),
            Some(Token::Identifier(id)) => id,
            _ => panic!("missing type token")
        }      
    }    
}

struct SymbolTableEntry {
    symbol_type : String,
    kind: String,
    index : u8
}

impl<T: Iterator<Item=Token>> CompilationEngine<VMInstruction> for VMCompiler<T> {        
    fn compile_class(&mut self) -> Vec<VMInstruction> {
        self.consume_keyword(KeyWord::Class);
        self.class_name = Some(self.process_identifier());        
        self.consume_symbol('{');
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
                        self.consume_symbol('}');
                        break;
                    },
                err => panic!("Unexpected class {:?}", err)
            }
        }
        self.instructions.clone()
    }
    
    fn compile_class_var_dec(&mut self){
        let kind = match self.tokens.next() {
            Some(Token::KeyWord(KeyWord::Field)) => "field",
            Some(Token::KeyWord(KeyWord::Static)) => "static",
            err => panic!("invalid class var dec {:?}", err)
        }; 

        let field_type = self.compile_type();
        let mut id = self.process_identifier();
        self.register_class_symbol(kind.to_string(), field_type.clone(), id);
        while let Some(Token::Symbol(',')) = self.tokens.peek() {
            self.consume_symbol(',');
            id = self.process_identifier();
            self.register_class_symbol(kind.to_string(), field_type.clone(), id);
        }
        self.consume_symbol(';');     
    }

    fn compile_var_dec(&mut self) {}
        
    fn compile_subroutine(&mut self){
        // reset symbol table
        self.local_symbol_counter=0;                
        self.arg_symbol_counter=0;   
        self.subroutine_symbol_table.clear();

        self.instruction_buffer.clear();

        let sub_routine_type = self.tokens.next().unwrap();
        let has_return_type = self.tokens.next().unwrap() != Token::KeyWord(KeyWord::Void);
        let id = self.process_identifier();        
        match sub_routine_type {
            Token::KeyWord(KeyWord::Function) => {}
            Token::KeyWord(KeyWord::Method) => {   
                self.register_subroutine_symbol("argument".to_string(), self.class_name.as_ref().unwrap().clone(), "this".to_string());             
                self.instruction_buffer.push(
                    VMInstruction::CPush{ segment: "argument".to_string(), value: 0 }
                );
                self.instruction_buffer.push(
                    VMInstruction::CPop{  segment: "pointer".to_string(),  value: 0 }
                );
            },
            Token::KeyWord(KeyWord::Constructor) => {
                self.instruction_buffer.push(
                    VMInstruction::CPush{ segment: "constant".to_string(), value: self.field_symbol_counter as u32 }
                );
                self.instruction_buffer.push(
                    VMInstruction::CCall{ symbol: "Memory.alloc".to_string(), n_args: 1 }
                );
                self.instruction_buffer.push(
                    VMInstruction::CPop{ segment: "pointer".to_string(), value: 0 }
                );
            },
            err => panic!("Unexpected subroutine type {:?}", err)
        }    
        self.consume_symbol('(');
        self.compile_parameter_list();
        self.consume_symbol(')');
        self.compile_subroutine_body(); 
        // drop the value returned
        if !has_return_type {
            self.instruction_buffer.push(VMInstruction::CPop{ segment: "temp".to_string(), value: 0 })
        }
        let class_name = self.class_name.as_ref().unwrap();
        self.instructions.push(
            VMInstruction::CFunction{ 
                symbol: format!("{}.{}", class_name, id).to_string(), 
                n_vars: self.local_symbol_counter as u32
            }
        );
        self.instructions.append(&mut self.instruction_buffer); 
        self.instruction_buffer.clear();       
    }    

    fn compile_parameter_list(&mut self){
       loop {
            match self.tokens.peek() {
                Some(Token::Symbol(')')) => break,
                Some(Token::Symbol(',')) => self.consume_symbol(','),
                _ => {
                    let arg_type = self.compile_type();
                    let id = self.process_identifier();
                    self.register_subroutine_symbol("argument".to_string(), arg_type, id);
                }
            }
        }
    }

    fn compile_statements(&mut self){
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
    }

    fn compile_subroutine_body(&mut self){
        self.consume_symbol('{');
        loop {
            if let Some(Token::KeyWord(KeyWord::Var)) = self.tokens.peek() {
                self.consume_keyword(KeyWord::Var);
                let arg_type = self.compile_type();
                loop {
                    match self.tokens.next().unwrap() {
                        Token::Symbol(';') => break,
                        Token::Symbol(',') => continue,
                        Token::Identifier(id) => {
                            self.register_subroutine_symbol("local".to_string(), arg_type.clone(), id);
                        },
                        err => panic!("unexpected variable type {:?}", err)
                    }
                } 
            } else {
                break;
            }
        }
        
        self.compile_statements();
        self.consume_symbol('}');
    }

    fn compile_do(&mut self){
        self.consume_keyword(KeyWord::Do);
        self.compile_expression();
        self.instruction_buffer.push (
            VMInstruction::CPop { segment: "temp".to_string(), value: 0 }
        );
        self.consume_symbol(';'); 
    }    

    fn compile_if(&mut self){
        self.consume_keyword(KeyWord::If);
        self.consume_symbol('(');
        self.compile_expression();
        self.instruction_buffer.push(
            VMInstruction::CArithmetic { cmd: "not".to_string() }
        ); 
        let if_counter = self.if_counter;
        self.if_counter+=1;                       
       
        self.instruction_buffer.push( 
            VMInstruction::CIf {  label: format!("IF_FALSE_{}", if_counter) }
        );
        self.consume_symbol(')');
        self.consume_symbol('{');
        self.compile_statements();
        self.instruction_buffer.push( 
            VMInstruction::CGoto { label: format!("IF_END_{}", if_counter)}
        );
        self.consume_symbol('}');
        self.instruction_buffer.push( 
            VMInstruction::CLabel { label: format!("IF_FALSE_{}", if_counter)}
        );
        if let Some(Token::KeyWord(KeyWord::Else)) = self.tokens.peek() {
            self.consume_keyword(KeyWord::Else);
            self.consume_symbol('{');
            self.compile_statements();
            self.consume_symbol('}');
        }
        self.instruction_buffer.push( 
            VMInstruction::CLabel{ label: format!("IF_END_{}", if_counter)}
        );
    }

    fn compile_let(&mut self){
        self.consume_keyword(KeyWord::Let);
        let id = self.process_identifier();
        let target = self.lookup_symbol(&id).unwrap();
        let kind = target.kind.clone();
        let index = target.index.clone();
        if let Some(Token::Symbol('[')) = self.tokens.peek() {
            self.instruction_buffer.push(VMInstruction::CPush { segment: kind, value: index as u32});
            self.consume_symbol('[');
            self.compile_expression();
            self.instruction_buffer.push(VMInstruction::CArithmetic { cmd: "add".to_string() });
            self.consume_symbol(']');
            self.consume_symbol('=');
            self.compile_expression();
            self.instruction_buffer.push(VMInstruction::CPop { segment: "temp".to_string(), value: 0 });
            self.instruction_buffer.push(VMInstruction::CPop { segment: "pointer".to_string(), value: 1 });
            self.instruction_buffer.push(VMInstruction::CPush { segment: "temp".to_string(), value: 0 });
            self.instruction_buffer.push(VMInstruction::CPop { segment: "that".to_string(), value: 0 });
        } else {
            self.consume_symbol('=');
            self.compile_expression();
            self.instruction_buffer.push(VMInstruction::CPop { segment: kind, value: index as u32 });            
        }
        self.consume_symbol(';');
    }

    fn compile_while(&mut self){        
        self.consume_keyword(KeyWord::While);
        let while_counter = self.while_counter;
        self.while_counter += 1;
        self.instruction_buffer.push(
            VMInstruction::CLabel{ label: format!("while_loop_{}", while_counter) }
        );
        self.consume_symbol('(');
        self.compile_expression();
        self.instruction_buffer.push(
            VMInstruction::CArithmetic { cmd: "not".to_string() }
        );
        self.instruction_buffer.push(
            VMInstruction::CIf{ label: format!("end_while_{}", while_counter) }
        );
        self.consume_symbol(')');
        self.consume_symbol('{');
        self.compile_statements();
        self.instruction_buffer.push(
            VMInstruction::CGoto{ label: format!("while_loop_{}", while_counter) }
        );
        self.consume_symbol('}');
        self.instruction_buffer.push(
            VMInstruction::CLabel{ label: format!("end_while_{}", while_counter) }
        );
        
    }    

    fn compile_return(&mut self){
        self.consume_keyword(KeyWord::Return);
        match self.tokens.peek() {
            Some(Token::Symbol(c)) if *c == ';' => {
                self.instruction_buffer.push( 
                    VMInstruction::CPush{ segment: "constant".to_string(), value: 0}
                );
            },
            _ => {
                self.compile_expression();
            }
        }
        self.consume_symbol(';');
        self.instruction_buffer.push(VMInstruction::CReturn);
    }   

    fn compile_expression_list(&mut self) -> u8 {
        let mut expression_count = 0;
        loop {
            if let Some(Token::Symbol(')')) = self.tokens.peek() {
                break
            }
            self.compile_expression();
            expression_count+=1;
            if let Some(Token::Symbol(',')) = self.tokens.peek() {
                self.consume_symbol(',');
            }
        }
        expression_count      
    }
    
    fn compile_expression(&mut self) {
        self.compile_term();
        match self.tokens.peek() {
            Some(Token::Symbol(c)) if VMCompiler::<T>::BINARY_OP.contains(c) => {},
            _ => return
        }
        if let Some(Token::Symbol(op))= self.tokens.next() {
            self.compile_term();        
            match op {
                '<' => self.instruction_buffer.push(
                    VMInstruction::CArithmetic{ cmd: "lt".to_string() }
                ),
                '>' => self.instruction_buffer.push(
                    VMInstruction::CArithmetic{ cmd: "gt".to_string() }
                ),
                '=' => self.instruction_buffer.push(
                    VMInstruction::CArithmetic{ cmd: "eq".to_string() }
                ),
                '+' => self.instruction_buffer.push(
                    VMInstruction::CArithmetic{ cmd: "add".to_string() }
                ),
                '-' => self.instruction_buffer.push(
                    VMInstruction::CArithmetic{ cmd: "sub".to_string() }
                ),
                '*' => self.instruction_buffer.push(
                    VMInstruction::CCall{ symbol: "Math.multiply".to_string(), n_args: 2}
                ),
                '/' => self.instruction_buffer.push(
                    VMInstruction::CCall{ symbol: "Math.divide".to_string(), n_args: 2}
                ),
                '&' => self.instruction_buffer.push(
                    VMInstruction::CArithmetic{ cmd: "and".to_string() }
                ),
                '|' => self.instruction_buffer.push(
                    VMInstruction::CArithmetic{ cmd: "or".to_string() }
                ),
                _ => panic!("Unexpected binary operator {}", op)
            }
        }
    }

    fn compile_term(&mut self) {
        let token = self.tokens.next().unwrap();
        match token {
            Token::Int(i) => self.instruction_buffer.push(
                VMInstruction::CPush{ segment: "constant".to_string(), value: i as u32 }
            ),
            Token::String(s) => { 
                self.instruction_buffer.push(
                    VMInstruction::CPush{ segment: "constant".to_string(), value: s.len() as u32 }
                );
                self.instruction_buffer.push(
                    VMInstruction::CCall { symbol: "String.new".to_string(), n_args: 1 }
                );
                for c in s.chars() {
                    self.instruction_buffer.push(
                        VMInstruction::CPush{ segment: "constant".to_string(), value: c as u32 }
                    );
                    self.instruction_buffer.push(
                        VMInstruction::CCall { symbol: "String.appendChar".to_string(), n_args: 2 }
                    );
                }                
            },
            Token::KeyWord(KeyWord::True) => {
                self.instruction_buffer.push(
                    VMInstruction::CPush{ segment: "constant".to_string(), value: 1 }
                );
                self.instruction_buffer.push(
                    VMInstruction::CArithmetic{ cmd: "neg".to_string() }
                );
            },
            Token::KeyWord(KeyWord::False) => self.instruction_buffer.push(
                VMInstruction::CPush{ segment: "constant".to_string(), value: 0 }
            ),
            Token::KeyWord(KeyWord::Null) => self.instruction_buffer.push(
                VMInstruction::CPush{ segment: "constant".to_string(), value: 0 }
            ),
            Token::KeyWord(KeyWord::This) => self.instruction_buffer.push(
                VMInstruction::CPush{ segment: "pointer".to_string(), value: 0 }
            ),
            Token::Symbol('(') => { 
                self.compile_expression();
                self.consume_symbol(')');  
            },
            Token::Symbol(s) if s == '~' => {
                self.compile_term();
                self.instruction_buffer.push(
                    VMInstruction::CArithmetic{ cmd: "not".to_string() }
                );
            },
            Token::Symbol(s) if s == '-' => {
                self.compile_term();
                self.instruction_buffer.push(
                    VMInstruction::CArithmetic{ cmd: "neg".to_string() }
                );
            },            
            Token::Identifier(id) => {
                match self.tokens.peek().unwrap() {
                    Token::Symbol(s) if *s == '[' => {
                        let symbol_entry = self.lookup_symbol(&id).unwrap().clone();
                        let segment = symbol_entry.kind.clone();
                        let value = symbol_entry.index;
                        self.instruction_buffer.push( 
                            VMInstruction::CPush{ segment, value: value as u32 } 
                        );
                        self.consume_symbol('[');
                        self.compile_expression();
                        self.instruction_buffer.push( 
                            VMInstruction::CArithmetic{ cmd: "add".to_string() }
                        );
                        self.instruction_buffer.push( 
                            VMInstruction::CPop{ segment : "pointer".to_string(), value: 1 } 
                        );                        
                        self.instruction_buffer.push( 
                            VMInstruction::CPush{ segment : "that".to_string(), value: 0 } 
                        );
                        self.consume_symbol(']');                        
                    },
                    Token::Symbol(s) if *s == '.' => {
                        let symbol_table_entry = self.lookup_symbol(&id);
                        let mut class_name = id.clone();
                        let method_instruction = symbol_table_entry.map( |entry| {
                                class_name = entry.symbol_type.clone();
                                VMInstruction::CPush{ segment: entry.kind.clone(), value : entry.index as u32 }
                            }
                        );
                        
                        self.tokens.next();
                        let function_name = self.process_identifier().clone(); 
                        self.consume_symbol('(');
                        // push ref to instance if needed
                        let n_args = if let Some(ins) = method_instruction {
                            self.instruction_buffer.push( ins );
                            self.compile_expression_list() + 1
                        } else {   
                            self.compile_expression_list()
                        };                
                        // call function
                        self.instruction_buffer.push( 
                            VMInstruction::CCall{ 
                                symbol : format!("{}.{}", class_name, function_name),
                                n_args : n_args as u32
                            }
                        );
                        self.consume_symbol(')');   
                    },
                    Token::Symbol(s) if *s == '(' => {
                        self.consume_symbol('(');                           
                        let class_name = self.class_name.as_ref().unwrap().clone();
                        // push pointer to current object
                        self.instruction_buffer.push( 
                            VMInstruction::CPush{ segment : "pointer".to_string(), value: 0 }
                        );
                        // add the args
                        let n_args = self.compile_expression_list();
                        // call the method
                        self.instruction_buffer.push( 
                            VMInstruction::CCall{ 
                                symbol : format!("{}.{}", &class_name, id),
                                n_args : (n_args as u32) + 1
                            }
                        );
                        self.consume_symbol(')');   
                    },
                    _ => {
                        let symbol_entry = self.lookup_symbol(&id).unwrap().clone();
                        let segment = symbol_entry.kind.clone();
                        let value = symbol_entry.index;
                        self.instruction_buffer.push( 
                            VMInstruction::CPush{ segment, value: value as u32 }
                        );
                    }
                }
            },
            _ => {}
        }
    }
}   