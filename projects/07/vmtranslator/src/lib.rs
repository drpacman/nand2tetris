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
    CArithmetic{ cmd : String },
    CPush{ segment : String , value: u16 },
    CPop{ segment : String, value: u16 },
    // CLabel{ symbol : String  },
    // CGoto{ symbol : String  },
    // C_IF{ symbol : String  },
    // C_FUNCTION{ symbol : String  },
    // CReturn,
    // C_CALL{ symbol : String },
}

impl fmt::Display for VMInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::CArithmetic { ref cmd } => write!(f, "{}", cmd),
            Self::CPush { ref segment, ref value } => write!(f, "push {} {}", segment, value),
            Self::CPop { ref segment, ref value } => write!(f, "pop {} {}", segment, value)                        
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
            Some(VMInstructionParser::parse_instruction(String::from_iter(output)))
        }
    }

    fn parse_instruction(ins: String) -> VMInstruction {
        lazy_static! {
            // instruction reg exs
            static ref PUSH_OR_POP_REGEX : Regex = Regex::new(r"^(push|pop) (\w+) (\d+)$").unwrap();
            static ref ARITHMETIC_REGEX : Regex = Regex::new(r"^(add|sub|neg|eq|lt|gt|and|or|not)$").unwrap();
        }
        if PUSH_OR_POP_REGEX.is_match(&ins) {
            let captures = PUSH_OR_POP_REGEX.captures(&ins).unwrap();
            let segment = captures.get(2).unwrap().as_str();
            let index = captures.get(3).unwrap().as_str().parse::<u16>().unwrap();
            if captures.get(1).unwrap().as_str() == "push" {
                VMInstruction::CPush{ segment: segment.to_string(), value: index }
            } else {
                VMInstruction::CPop{ segment: segment.to_string(), value: index }
            }
        } else if ARITHMETIC_REGEX.is_match(&ins){
            VMInstruction::CArithmetic{ cmd: ins.clone() }
        } else {
            panic!("Unexpected instruction {}", ins)
        }
    }
}

pub struct Compiler {
    bool_symbol_counter : i16
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler{ bool_symbol_counter: 0 }
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
                        output.push(Instruction::AInstruction { symbol: None, value: Some(*value) });
                        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });                
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
                    }
                }
                //Push the value in D
                output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });
                //Increment the stack pointer
                output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M+1".to_string(), jump: None });            
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

                //Decrement the stack pointer
                output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-1".to_string(), jump: None });            
                //Pop the current stack value into the address at R13
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });
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
            }             
        }
        output
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

    pub fn write(&self, ins : Vec<Instruction>) {
        let mut f = File::create(&self.filename).expect("unable to create file");
        let asm : Vec<String> = ins.iter().map(|i| i.to_string()).collect();
        write!(f, "{}", asm.join("\r")).expect("Failed to write ASM to file")
    }
}