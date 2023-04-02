use std::fs::File;
use std::io::prelude::*;
use std::env;
// use std::io::Write;
use regex::Regex;
use lazy_static::lazy_static;
use assembler::Instruction;
use std::fmt;


#[derive(Debug)]
pub enum VMInstruction {
    CReturn,
    CArithmetic{ cmd : String },
    CLabel{ label : String  },
    CGoto{ label : String  },
    CIf{ label : String  },
    CPush{ segment : String , value: u16 },
    CPop{ segment : String, value: u16 },
    CFunction{ symbol : String, n_vars : u16 },
    CCall{ symbol : String, n_args : u16 },
}

impl fmt::Display for VMInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::CArithmetic { ref cmd } => write!(f, "{}", cmd),
            Self::CPush { ref segment, ref value } => write!(f, "push {} {}", segment, value),
            Self::CPop { ref segment, ref value } => write!(f, "pop {} {}", segment, value),
            Self::CLabel { ref label } => write!(f, "({})", label),
            Self::CGoto { ref label }  => write!(f, "goto {}", label),                     
            Self::CIf { ref label} => write!(f, "if-goto {}", label),
            Self::CFunction { ref symbol, ref n_vars} => write!(f, "function {} {}", symbol, n_vars),
            Self::CReturn => write!(f, "return"),
            Self::CCall { ref symbol,ref n_args } => write!(f, "call {} {}", symbol, n_args)
        }
    }
}

pub struct VMInstructionParser {
    filename : String
}

impl VMInstructionParser {

    pub fn new(filestem : &str) -> VMInstructionParser {
        VMInstructionParser { filename: format!("{}.vm", filestem) }
    }

    pub fn parse(&self) -> Result<Vec<VMInstruction>,std::io::Error> {
        let mut f = File::open(& self.filename).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let lines = contents.to_string()
                            .split('\n')
                            .filter_map(|s| VMInstructionParser::read_instruction(s))
                            .collect();
        Ok(lines)
    }

    fn read_instruction(line : &str) -> Option<VMInstruction>{
        let mut cs = line.trim().chars();
        let mut done = false;
        let mut output : Vec<char> = Vec::new();
        let mut comment = false;
        while !done {
            match cs.next() {
                Some('/') => {
                    if comment { break };
                    comment = true;
                },
                Some(c) => {
                    comment = false;
                    output.push(c);
                },
                None => {
                    done = true;
                }
            }
        }
        if output.len() == 0 {
            None
        } else {
            Some(VMInstructionParser::parse_instruction(String::from_iter(output).trim().to_string()))
        }
    }

    fn parse_instruction(ins: String) -> VMInstruction {
        lazy_static! {
            // instruction reg exs
            static ref FUN0_REGEX : Regex = Regex::new(r"^(return|add|sub|neg|eq|lt|gt|and|or|not)$").unwrap();
            static ref FUN1_REGEX : Regex = Regex::new(r"^(if-goto|goto|label) (\w+)$").unwrap();
            static ref FUN2_REGEX : Regex = Regex::new(r"^(push|pop|function|call) ([\w.]+) (\d+)$").unwrap();
        }
        if FUN2_REGEX.is_match(&ins) {
            let captures = FUN2_REGEX.captures(&ins).unwrap();
            let arg1 = captures.get(2).unwrap().as_str().to_string();
            let arg2 = captures.get(3).unwrap().as_str().parse::<u16>().unwrap();
            match captures.get(1).unwrap().as_str() {
                "push" => VMInstruction::CPush{ segment: arg1, value: arg2 },
                "pop" => VMInstruction::CPop{ segment: arg1, value: arg2 },
                "function" => VMInstruction::CFunction { symbol: arg1, n_vars: arg2 },
                "call" => VMInstruction::CCall { symbol:  arg1, n_args: arg2 },
                _ => panic!("Unexpected 2 arg instruction {}", ins)
            }
        } else if FUN1_REGEX.is_match(&ins){
            let captures = FUN1_REGEX.captures(&ins).unwrap();
            let label = captures.get(2).unwrap().as_str().to_string();
            match captures.get(1).unwrap().as_str() {
                "if-goto" => VMInstruction::CIf { label },
                "goto" => VMInstruction::CGoto { label },
                "label" => VMInstruction::CLabel { label },
                _ => panic!("Unexpected 2 arg instruction {}", ins)
            }
        } else if FUN0_REGEX.is_match(&ins){
            if ins == "return" {
                VMInstruction::CReturn
            } else {
                VMInstruction::CArithmetic{ cmd: ins.clone() }
            }
        } else {
            panic!("Unexpected instruction {}", ins)
        }
    }
}

pub struct Compiler {
    bool_symbol_counter : i16,
    ret_symbol_count : i16
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler{ 
            bool_symbol_counter: 0,
            ret_symbol_count: 0
        }
    }

    pub fn compile(&mut self, vm_instructions: Vec<VMInstruction>) -> Vec<assembler::Instruction> {
        self.bool_symbol_counter = 0;
        vm_instructions.iter().map(|ins| self.compile_instruction(ins)).flatten().collect()
    
    }
    fn compile_instruction(&mut self, ins : &VMInstruction) -> Vec<Instruction> {
        let mut output = Vec::new();
        output.push(Instruction::Comment { contents: format!("{}", ins) });
        match ins {
            VMInstruction::CPush{ segment, value } => {
                match segment.as_str() {
                    "constant" => {
                        Compiler::push_value(*value, &mut output);
                    },
                    _ => {
                        let target = match segment.as_str() {
                            "local" => "LCL",
                            "argument" => "ARG",
                            "this" => "THIS",
                            "that" => "THAT",
                            "temp" => "5",
                            "static" => "16",
                            "pointer" => "3",
                            _ => panic!("Unsupported PUSH segment {}", segment)
                        };
                        output.push(Instruction::AInstruction { symbol: Some(target.to_string()), value: None });                    
                        match target {
                            "LCL" | "ARG" | "THIS" | "THAT" => {
                                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
                            },
                            _ => {}
                        }
                        if *value > 0 {
                            output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });
                            output.push(Instruction::AInstruction { symbol: None, value: Some(*value) });
                            output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"D+A".to_string(), jump: None });
                        }
                        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });                    
                        Compiler::push_d(&mut output);            
                    }
                }
            }, 
            VMInstruction::CPop{ segment, value } => {
                let target= match segment.as_str() {
                    "local" => "LCL",
                    "argument" => "ARG",
                    "this" => "THIS",
                    "that" => "THAT",
                    "temp" => "5",
                    "static" => "16",
                    "pointer" => "3",
                    _ => panic!("Unsupported POP segment {}", segment)
                };
                output.push(Instruction::AInstruction { symbol: Some(target.to_string()), value: None });                    
                match target {
                    "LCL" | "ARG" | "THIS" | "THAT" => {
                        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
                    },
                    _ => {}
                }
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });
                if *value > 0 {
                    output.push(Instruction::AInstruction { symbol: None, value: Some(*value) });
                    output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"D+A".to_string(), jump: None });
                }

                output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });

                //Pop the current stack value into the address at R13
                Compiler::pop_d(&mut output);
                output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });            
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });            
            },
            VMInstruction::CArithmetic { cmd } => {
                // grab top value off the stack
                output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
                    
                if cmd == "not" {
                    output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"!M".to_string(), jump: None });                    
                } else if cmd == "neg" {
                    output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"-M".to_string(), jump: None });                      
                } else {                         
                    // get prior value from stack
                    output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });
                    output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
                    match cmd.as_str() {
                        "add" => output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D+M".to_string(), jump: None }),
                        "sub" => output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-D".to_string(), jump: None }),
                        "and" => output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D&M".to_string(), jump: None }),
                        "or" => output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D|M".to_string(), jump: None }),
                        _ => {
                            let jump = match cmd.as_str() {
                                "lt" => Some("JLT".to_string()),
                                "gt" => Some("JGT".to_string()),
                                "eq" => Some("JEQ".to_string()),
                                _ => panic!("Unexpected cmd {}", cmd)
                            };
                            output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M-D".to_string(), jump: None });
                            output.push(Instruction::AInstruction { symbol: Some(format!("BOOL_{}", self.bool_symbol_counter)), value: None });
                            output.push(Instruction::CInstruction { dest: None, comp:"D".to_string(), jump });
                            output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });                
                            output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
                            output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
                            output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
                            output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"0".to_string(), jump: None });
                            output.push(Instruction::AInstruction { symbol: Some(format!("END_BOOL_{}", self.bool_symbol_counter)), value: None });
                            output.push(Instruction::CInstruction { dest: None, comp:"0".to_string(), jump: Some("JMP".to_string()) });
                            output.push(Instruction::LInstruction { symbol: format!("BOOL_{}", self.bool_symbol_counter) });
                            output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });                
                            output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
                            output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
                            output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
                            output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"-1".to_string(), jump: None });
                            output.push(Instruction::LInstruction { symbol: format!("END_BOOL_{}", self.bool_symbol_counter) });
                            self.bool_symbol_counter += 1;                        
                        }                
                    }
                    // decrement SP
                    output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
                    output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-1".to_string(), jump: None });                       
                }
            },
            VMInstruction::CLabel { label } => {
                output.push(Instruction::LInstruction { symbol: label.to_string() });
            },
            VMInstruction::CGoto { label } => {
                output.push(Instruction::AInstruction { symbol: Some(label.to_string()), value: None });
                output.push(Instruction::CInstruction { dest: None, comp: "0".to_string(), jump: Some("JMP".to_string()) })                            
            },
            VMInstruction::CIf { label } => {
                Compiler::pop_d(&mut output);
                output.push(Instruction::AInstruction { symbol: Some(label.to_string()), value: None });
                output.push(Instruction::CInstruction { dest: None, comp:"D".to_string(), jump: Some("JNE".to_string()) });
            },
            VMInstruction::CCall { symbol, n_args } => {
                Compiler::push_symbol(format!("ret_{}", self.ret_symbol_count).as_str(), &mut output);
                Compiler::push_symbol("LCL", &mut output);
                Compiler::push_symbol("ARG", &mut output);
                Compiler::push_symbol("THIS", &mut output);
                Compiler::push_symbol("THAT", &mut output);
                // // update ARG to SP-5-nargs
                Compiler::push_symbol("SP", &mut output);
                Compiler::push_value(5, &mut output);
                Compiler::push_value(*n_args, &mut output);
                Compiler::sub(&mut output);
                Compiler::sub(&mut output);
                Compiler::pop_symbol("ARG", &mut output);
                // // set LCL to current SP
                output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });   
                output.push(Instruction::AInstruction { symbol: Some("LCL".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });   
                // goto function
                output.push(Instruction::AInstruction { symbol: Some(symbol.to_string()), value: None });
                output.push(Instruction::CInstruction { dest: None, comp:"0".to_string(), jump: Some("JMP".to_string()) });   
                output.push(Instruction::LInstruction { symbol: format!("ret_{}", self.ret_symbol_count) });
                self.ret_symbol_count += 1;                
            },
            VMInstruction::CFunction { symbol, n_vars } => {
                for i in 0..*n_vars {
                    Compiler::push_value(0, &mut output);
                }
            },
            VMInstruction::CReturn => {
                // Save current LCL location in R13
                // @R13 = LCL
                output.push(Instruction::AInstruction { symbol: Some("LCL".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None }); 
                output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None }); 
                
                // pop stack value onto current value of ARG
                Compiler::pop_symbol("ARG", &mut output);
                
                // set SP = *ARG + 1
                output.push(Instruction::AInstruction { symbol: Some("ARG".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None }); 
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A+1".to_string(), jump: None }); 
                output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });                 
                
                // Restore THAT
                // THAT = @R13 - 1
                output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-1".to_string(), jump: None });                 
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });                 
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });                 
                output.push(Instruction::AInstruction { symbol: Some("THAT".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None }); 
                  
                // Restore THIS
                // THIS = @R13 - 2
                output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-1".to_string(), jump: None });                 
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });                 
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });                 
                output.push(Instruction::AInstruction { symbol: Some("THIS".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None }); 
                
                // Restore ARG
                // ARG = @R13 - 3
                output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-1".to_string(), jump: None });                 
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });                 
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });                 
                output.push(Instruction::AInstruction { symbol: Some("ARG".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None }); 
                
                // Restore LCL
                // LCL = @R13 - 4
                output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-1".to_string(), jump: None });                 
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });                 
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });                 
                output.push(Instruction::AInstruction { symbol: Some("LCL".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None }); 
                
                // goto retAddr = @R13 - 5
                output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-1".to_string(), jump: None });                 
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None }); 
                output.push(Instruction::CInstruction { dest: None, comp:"0".to_string(), jump: Some("JMP".to_string()) });                 
            }
        }
        output
    }

    fn push_symbol(symbol : &str, output : &mut Vec<Instruction>) {
        output.push(Instruction::AInstruction { symbol: Some(symbol.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None }); 
        Compiler::push_d(output);
    }

    fn push_value(value : u16, output : &mut Vec<Instruction>) {
        output.push(Instruction::AInstruction { symbol: None, value: Some(value) });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None }); 
        Compiler::push_d(output);
    }

    fn push_d(output : &mut Vec<Instruction>) {
        //Push the value in D
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });
        //Increment the stack pointer
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M+1".to_string(), jump: None }); 
    }

    fn sub(output : &mut Vec<Instruction>) {
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-D".to_string(), jump: None });
    }

    fn pop_symbol(symbol : &str, output : &mut Vec<Instruction>) {
        Compiler::pop_d(output);
        output.push(Instruction::AInstruction { symbol: Some(symbol.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });      
    }

    fn pop_d(output : &mut Vec<Instruction>) {
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        // dec stack pointer
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-1".to_string(), jump: None });            
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });
    }
}

pub struct ASMWriter {
    filename : String
}
impl ASMWriter {
    pub fn new(filestem : &str) -> ASMWriter {
        ASMWriter { 
            filename:  format!("{}.asm", filestem)
        }
    }

    pub fn write(&self, ins : &Vec<Instruction>) {
        let mut f = File::create(&self.filename).expect("unable to create file");
        let asm : Vec<String> = ins.iter().map(|i| i.to_string()).collect();
        write!(f, "{}", asm.join("\r")).expect("Failed to write ASM to file")
    }
}