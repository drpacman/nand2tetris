use std::fs::File;
use std::io::prelude::*;
use regex::Regex;
use lazy_static::lazy_static;
use assembler::Instruction;
use std::fmt;


#[derive(Debug, Clone )]
pub enum VMInstruction {
    CReturn,
    CArithmetic{ cmd : String },
    CLabel{ label : String  },
    CGoto{ label : String  },
    CIf{ label : String  },
    CPush{ segment : String , value: u32 },
    CPop{ segment : String, value: u32 },
    CFunction{ symbol : String, n_vars : u32 },
    CCall{ symbol : String, n_args : u32 },
    CPushPop{ src_segment: String, src_value: u32, dst_segment: String, dst_value: u32 }
}

impl fmt::Display for VMInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::CArithmetic { ref cmd } => write!(f, "{}", cmd),
            Self::CPush { ref segment, ref value } => write!(f, "push {} {}", segment, value),
            Self::CPop { ref segment, ref value } => write!(f, "pop {} {}", segment, value),
            Self::CLabel { ref label } => write!(f, "label {}", label),
            Self::CGoto { ref label }  => write!(f, "goto {}", label),                     
            Self::CIf { ref label} => write!(f, "if-goto {}", label),
            Self::CFunction { ref symbol, ref n_vars} => write!(f, "function {} {}", symbol, n_vars),
            Self::CReturn => write!(f, "return"),
            Self::CCall { ref symbol,ref n_args } => write!(f, "call {} {}", symbol, n_args),
            Self::CPushPop{ ref src_segment, src_value, ref dst_segment, dst_value } => {
                write!(f, "pushpop {} {} {} {}", src_segment, src_value, dst_segment, dst_value)
            }
        }
    }
}

pub struct VMInstructionParser {
    filename : String
}

impl VMInstructionParser {

    pub fn new(filename : &str) -> VMInstructionParser {
        VMInstructionParser { filename: filename.to_string() }
    }

    pub fn parse(&self) -> Result<Vec<VMInstruction>,std::io::Error> {
        let mut f = File::open(& self.filename).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let lines = contents.to_string()
                            .split('\r')
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
            let arg2 = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();
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
    bool_symbol_counter : u32,
    ret_symbol_count : u32,
    static_base : u32,
    scope_label_prefix : Option<String>
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler{ 
            bool_symbol_counter: 0,
            ret_symbol_count: 0,
            static_base: 16,
            scope_label_prefix: None
        }
    }

    pub fn generate_bootstrap(&mut self) -> Vec<assembler::Instruction> {
        let mut init = Vec::new();
        // SP = 256
        init.push(Instruction::AInstruction { symbol: None, value: Some(256) });
        init.push(Instruction::CInstruction { dest: Some("D".to_string()), comp: ("A".to_string()), jump: None });
        init.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        init.push(Instruction::CInstruction { dest: Some("M".to_string()), comp: ("D".to_string()), jump: None });
        // call Sys.init
        self.call("Sys.init", 0, &mut init);
        init
    }

    pub fn generate_global_helper_functions(&mut self) -> Vec<assembler::Instruction> {
        let mut global_fns = Vec::new();
        // prefix helper functions with an infinite loop (so it doesn't break simpler examples)
        global_fns.append(&mut self.generate_global_halt());
        // add in global helper functions which reduce duplicated code
        global_fns.append(&mut self.generate_restore_stack_and_return_global_helper());
        global_fns.append(&mut self.generate_call_global_helper());
        global_fns.append(&mut self.generate_boolean_cmd_global_helper("JLT"));
        global_fns.append(&mut self.generate_boolean_cmd_global_helper("JGT"));
        global_fns.append(&mut self.generate_boolean_cmd_global_helper("JEQ"));  
        global_fns
    }

    fn generate_global_halt(&mut self) -> Vec<assembler::Instruction> {
        let mut output = Vec::new();
        output.push(Instruction::LInstruction { symbol: "HALT_LOOP".to_string() });
        output.push(Instruction::AInstruction { symbol: Some("HALT_LOOP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: None, comp: ("0".to_string()), jump: Some("JMP".to_string()) });        
        output
    }

    fn generate_restore_stack_and_return_global_helper(&mut self) -> Vec<assembler::Instruction> {
        let mut output = Vec::new();
        output.push(Instruction::LInstruction { symbol: "restore_stack_and_return".to_string() });
                
        // // @R13 = LCL - 5
        Compiler::assign("R13", "LCL", &mut output);    
        output.push(Instruction::AInstruction { symbol: None, value: Some(5) });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });                 
        output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-D".to_string(), jump: None });                 
        // save ret address in R14 
        // retAddr = *(LCL - 5)
        Compiler::assign_deref("R14", "R13", &mut output);    
                                                        
        // // pop stack value onto current location of ARG
        Compiler::pop_d(&mut output);
        output.push(Instruction::AInstruction { symbol: Some("ARG".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None }); 
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });                 
        
        // set SP = *ARG + 1
        output.push(Instruction::AInstruction { symbol: Some("ARG".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None }); 
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A+1".to_string(), jump: None }); 
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });                 
        
        // Restore LCL
        // LCL = *(@R13 - 4)
        output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M+1".to_string(), jump: None });                 
        Compiler::assign_deref("LCL", "R13", &mut output);                
        
        // Restore ARG
        // ARG = *(@R13 - 3)
        output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M+1".to_string(), jump: None });                 
        Compiler::assign_deref("ARG", "R13", &mut output);                
        
        // Restore THIS
        // THIS = *(@R13 - 2)
        output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M+1".to_string(), jump: None });                 
        Compiler::assign_deref("THIS", "R13", &mut output);                
        
        // Restore THAT
        // THAT = *(@R13 - 1)
        output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M+1".to_string(), jump: None });
        Compiler::assign_deref("THAT", "R13", &mut output);                
          
        // goto retAddr = *(@R13 - 5)
        output.push(Instruction::AInstruction { symbol: Some("R14".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });                 
        output.push(Instruction::CInstruction { dest: None, comp:"0".to_string(), jump: Some("JMP".to_string()) });
        return output;           
    }

    fn generate_call_global_helper(&mut self) -> Vec<assembler::Instruction> {
        // Helper function expects the following registers to have been setup
        // D = return label
        // R14 = n args
        // R15 = function symbol        
        let mut output = Vec::new();
        output.push(Instruction::LInstruction { symbol: "call_helper".to_string() });
        // save return label
        Compiler::push_d(&mut output);
        
        Compiler::push_symbol("LCL", &mut output);
        Compiler::push_symbol("ARG", &mut output);
        Compiler::push_symbol("THIS", &mut output);
        Compiler::push_symbol("THAT", &mut  output);
        // // update ARG to SP-5-nargs
        Compiler::push_symbol("SP", &mut output);
        Compiler::push_value(5, &mut output);
        Compiler::sub(&mut output);
        
        // push args number held in R14
        output.push(Instruction::AInstruction { symbol: Some("R14".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None }); 
        Compiler::push_d(&mut output);

        Compiler::sub(&mut output);
        Compiler::pop_symbol("ARG", &mut output);
        
        // // set LCL to current SP
        Compiler::assign("LCL", "SP", &mut output);
        
        // goto function - destination saved in R15
        output.push(Instruction::AInstruction { symbol: Some("R15".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });           
        output.push(Instruction::CInstruction { dest: None, comp:"0".to_string(), jump: Some("JMP".to_string()) });
        output
    }


    fn generate_boolean_cmd_global_helper(&mut self, jmp_cmd : &str) -> Vec<assembler::Instruction> {
        // Requires continuation address is in R15
        let mut output = Vec::new();
        output.push(Instruction::LInstruction { symbol: format!("boolean_cmd_helper_{}", jmp_cmd) });
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
            
        // get prior value from stack
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M-D".to_string(), jump: None });
        output.push(Instruction::AInstruction { symbol: Some(format!("BOOL_{}", jmp_cmd)), value: None });
        output.push(Instruction::CInstruction { dest: None, comp:"D".to_string(), jump: Some(jmp_cmd.to_string()) });
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });                
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"0".to_string(), jump: None });
        output.push(Instruction::AInstruction { symbol: Some(format!("END_BOOL_{}", jmp_cmd)), value: None });
        output.push(Instruction::CInstruction { dest: None, comp:"0".to_string(), jump: Some("JMP".to_string()) });
        output.push(Instruction::LInstruction { symbol: format!("BOOL_{}", jmp_cmd) });
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });                
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"-1".to_string(), jump: None });
        output.push(Instruction::LInstruction { symbol: format!("END_BOOL_{}", jmp_cmd) });
        self.bool_symbol_counter += 1;                        
        // decrement SP
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-1".to_string(), jump: None });
        // jump back to the invoking function
        output.push(Instruction::AInstruction { symbol: Some("R15".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: None, comp:"0".to_string(), jump: Some("JMP".to_string()) });        
        output
    }

    fn pre_process(instructions : Vec<VMInstruction>) -> Vec<VMInstruction>{
        let mut pre_processed_instuctions = Vec::new();
        let mut skip = false;
        for i in 0..(instructions.len() - 1){
            if skip { 
                skip = false;
                continue; 
            }
            match (instructions.get(i).unwrap(), instructions.get(i+1).unwrap()) {
                ( VMInstruction::CPush{ segment: src_seg, value: src_value }, VMInstruction::CPop{ segment: dst_seg, value: dst_value }) => {
                    pre_processed_instuctions.push(VMInstruction::CPushPop { 
                        src_segment: src_seg.clone(), 
                        src_value: *src_value, 
                        dst_segment: dst_seg.clone(), 
                        dst_value: *dst_value });
                    skip = true;
                }, 
                _ => {
                    pre_processed_instuctions.push(instructions.get(i).unwrap().clone());
                }
            }
        }
        if !skip {
            pre_processed_instuctions.push(instructions.get(instructions.len() - 1).unwrap().clone());
        }
        pre_processed_instuctions
    }

    pub fn compile(&mut self, vm_instructions: Vec<VMInstruction>) -> Vec<assembler::Instruction> {
        // convert any push followed by a pop into a single push pop command (for more efficient asm generation)
        let pre_processed_instuctions = Compiler::pre_process(vm_instructions);
        let instructions = pre_processed_instuctions.iter().map(|ins| self.compile_instruction(ins)).flatten().collect();
        // update static base
        let static_count = pre_processed_instuctions.iter().filter_map(|ins| {
            match ins  {
                VMInstruction::CPush{ segment, value } if segment == "static" => Some(*value + 1),
                VMInstruction::CPop{ segment, value } if segment == "static" => Some(*value + 1),              
                _ => Some(0)
            }
        }).max().unwrap();
        self.static_base += static_count;
        instructions
    }

    fn lookup_segment_target(&mut self, segment: &String) -> String {
        match segment.as_str() {
            "local" => "LCL".to_string(),
            "argument" => "ARG".to_string(),
            "this" => "THIS".to_string(),
            "that" => "THAT".to_string(),
            "temp" => "5".to_string(),
            "static" => { self.static_base.to_string() },
            "pointer" => "3".to_string(),
            _ => panic!("Unsupported segment {}", segment)
        }
    }

    fn scoped_label(&self, label: &String) -> String {
        if let Some(scope_label_prefix) = self.scope_label_prefix.as_ref() {
            format!("{}${}", scope_label_prefix, label).to_lowercase()
        } else {
            label.to_lowercase()
        }
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
                        let target = self.lookup_segment_target(segment);
                        output.push(Instruction::AInstruction { symbol: Some(target.clone()), value: None });                    
                        match target.as_str() {
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
            // VMInstruction::CPop{ segment, value } if segment == "pointer" => {
            //     Compiler::pop_d(&mut output);
            //     if *value == 0 {
            //         output.push(Instruction::AInstruction { symbol: Some("THIS".to_string()), value: None });                                        
            //     } else {
            //         output.push(Instruction::AInstruction { symbol: Some("THAT".to_string()), value: None });                                        
            //     } 
            //     output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
            //     output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });
            // }, 
            VMInstruction::CPop{ segment, value } => {
                let target = self.lookup_segment_target(segment);                     
                output.push(Instruction::AInstruction { symbol: Some(target.to_string()), value: None });                    
                match target.as_str() {
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
            VMInstruction::CPushPop{ src_segment, src_value, dst_segment, dst_value } => {
                // store the intended pushed value in R13
                match src_segment.as_str() {
                    "constant" => {
                        output.push(Instruction::AInstruction { symbol: None, value: Some(*src_value) });
                        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });            
                        output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });            
                    },
                    _ => {
                        let target = self.lookup_segment_target(src_segment);
                        output.push(Instruction::AInstruction { symbol: Some(target.clone()), value: None });                    
                        match target.as_str() {
                            "LCL" | "ARG" | "THIS" | "THAT" => {
                                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
                            },
                            _ => {}
                        }
                        // adjust Address A if not first item in segment
                        if *src_value == 1 {
                            output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A+1".to_string(), jump: None });
                        } else if *src_value > 1 {
                            output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });
                            output.push(Instruction::AInstruction { symbol: None, value: Some(*src_value) });
                            output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"D+A".to_string(), jump: None });
                        }

                        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });                    
                        output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });            
                    }
                }

                // calculate the target memory location and store in R14
                let target = self.lookup_segment_target(dst_segment);                        
                output.push(Instruction::AInstruction { symbol: Some(target.to_string()), value: None });                    
                match target.as_str() {
                    "LCL" | "ARG" | "THIS" | "THAT" => {
                        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });
                    },
                    _ => {}
                }
                match *dst_value {
                    0 => { 
                        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });                
                    },
                    1 => {
                        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A+1".to_string(), jump: None });
                    },
                    _ => {
                        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });                
                        output.push(Instruction::AInstruction { symbol: None, value: Some(*dst_value) });
                        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"D+A".to_string(), jump: None });
                    }
                }
                output.push(Instruction::AInstruction { symbol: Some("R14".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });            
                
                // Put the target value held in R13 into the destination address held in R14
                output.push(Instruction::AInstruction { symbol: Some("R13".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });            
                output.push(Instruction::AInstruction { symbol: Some("R14".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });                
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });                
            },
            VMInstruction::CArithmetic { cmd } => {
                match cmd.as_str() {
                    "sub" => { Compiler::sub(&mut output); },
                    "add"=> { Compiler::add(&mut output); },
                    "and"=> { Compiler::and(&mut output); },
                    "or"=> { Compiler::or(&mut output); },
                    "not"=> { Compiler::not(&mut output); },
                    "neg"=> { Compiler::neg(&mut output); },
                    "lt" => { self.lt(&mut output) },
                    "gt" => { self.gt(&mut output) },
                    "eq" => { self.eq(&mut output) },
                    _ => panic!("Unexpected arithmetic cmd {}", cmd)
                }
            },
            VMInstruction::CLabel { label } => {
                output.push(Instruction::LInstruction { symbol: self.scoped_label(label) });
            },
            VMInstruction::CGoto { label } => {
                output.push(Instruction::AInstruction { symbol: Some(self.scoped_label(label)), value: None });
                output.push(Instruction::CInstruction { dest: None, comp: "0".to_string(), jump: Some("JMP".to_string()) })                            
            },
            VMInstruction::CIf { label } => {
                Compiler::pop_d(&mut output);
                output.push(Instruction::AInstruction { symbol: Some(self.scoped_label(label)), value: None });
                output.push(Instruction::CInstruction { dest: None, comp:"D".to_string(), jump: Some("JNE".to_string()) });
            },
            VMInstruction::CCall { symbol, n_args } => {
                self.call(symbol, *n_args, &mut output);                             
            },
            VMInstruction::CFunction { symbol, n_vars } => {
                output.push(Instruction::LInstruction { symbol: symbol.to_string() });
                self.scope_label_prefix = Some(symbol.clone());
                for _ in 0..*n_vars {
                    Compiler::push_value(0, &mut output);
                }
            },
            VMInstruction::CReturn => {
                // use shared definition
                Compiler::goto_label("restore_stack_and_return", &mut output);
            }
        }
        output
    }

    fn push_symbol(symbol : &str, output : &mut Vec<Instruction>) {
        output.push(Instruction::AInstruction { symbol: Some(symbol.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None }); 
        Compiler::push_d(output);
    }

    fn push_value(value : u32, output : &mut Vec<Instruction>) {
        match value {
            0 | 1 => {
                output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("AM".to_string()), comp:"M+1".to_string(), jump: None });
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:value.to_string(), jump: None });
            },
            _ => {
                output.push(Instruction::AInstruction { symbol: None, value: Some(value) });
                output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None }); 
                output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
                output.push(Instruction::CInstruction { dest: Some("AM".to_string()), comp:"M+1".to_string(), jump: None });
                output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
                output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });
            }
        };
    }

    fn push_d(output : &mut Vec<Instruction>) {
        //Push the value in D, incrementing the stack
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("AM".to_string()), comp:"M+1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });
    }

    fn sub(output : &mut Vec<Instruction>) {
        Compiler::arithmetic_cmd("M-D", output);
    }

    fn add(output : &mut Vec<Instruction>) {
        Compiler::arithmetic_cmd("D+M", output);
    }

    fn or(output : &mut Vec<Instruction>) {
        Compiler::arithmetic_cmd("D|M", output);
    }
    
    fn and(output : &mut Vec<Instruction>) {
        Compiler::arithmetic_cmd("D&M", output);
    }
    
    fn arithmetic_cmd(cmd : &str, output : &mut Vec<Instruction>){
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"A-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:cmd.to_string(), jump: None });
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"M-1".to_string(), jump: None });        
    }

    fn neg(output : &mut Vec<Instruction>){
        Compiler::unary_cmd("-M", output);
    }

    fn not(output : &mut Vec<Instruction>){
        Compiler::unary_cmd("!M", output);
    }
    
    fn unary_cmd(cmd: &str, output : &mut Vec<Instruction>){
        // grab top value off the stack
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M-1".to_string(), jump: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:cmd.to_string(), jump: None });                            
    }

    fn pop_symbol(symbol : &str, output : &mut Vec<Instruction>) {
        Compiler::pop_d(output);
        output.push(Instruction::AInstruction { symbol: Some(symbol.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });      
    }

    fn pop_d(output : &mut Vec<Instruction>) {
        output.push(Instruction::AInstruction { symbol: Some("SP".to_string()), value: None });
        // dec stack pointer
        output.push(Instruction::CInstruction { dest: Some("AM".to_string()), comp:"M-1".to_string(), jump: None });            
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });
    }

    fn goto_label(symbol: &str, output: &mut Vec<Instruction>) {
        output.push(Instruction::AInstruction { symbol: Some(symbol.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: None, comp:"0".to_string(), jump: Some("JMP".to_string()) });
    }

    fn assign(symbol: &str, other : &str, output: &mut Vec<Instruction>) {
        output.push(Instruction::AInstruction { symbol: Some(other.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });   
        output.push(Instruction::AInstruction { symbol: Some(symbol.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });           
    }

    fn assign_deref(symbol: &str, other : &str, output: &mut Vec<Instruction>) {
        output.push(Instruction::AInstruction { symbol: Some(other.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("A".to_string()), comp:"M".to_string(), jump: None });   
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"M".to_string(), jump: None });   
        output.push(Instruction::AInstruction { symbol: Some(symbol.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });           
    }

    pub fn call(&mut self, symbol: &str, n_args : u32, output : &mut Vec<Instruction>) {
        // Use an helper function to reduce ASM footprint
        // R14 = n args
        if n_args == 0 || n_args == 1 {
            output.push(Instruction::AInstruction { symbol: Some("R14".to_string()), value: None });
            output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp: n_args.to_string(), jump: None });                
        } else {
            output.push(Instruction::AInstruction { symbol: None, value: Some(n_args) });
            output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });   
            output.push(Instruction::AInstruction { symbol: Some("R14".to_string()), value: None });
            output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });                
        }
        // R15 = function symbol
        output.push(Instruction::AInstruction { symbol: Some(symbol.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });   
        output.push(Instruction::AInstruction { symbol: Some("R15".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });                
        // D = return label
        let return_label = format!("ret_{}", self.ret_symbol_count);
        output.push(Instruction::AInstruction { symbol: Some(return_label.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });           
        Compiler::goto_label("call_helper", output);
        
        // return label
        output.push(Instruction::LInstruction { symbol: return_label.clone() });
        self.ret_symbol_count += 1;  
    }

    fn lt(&mut self, output : &mut Vec<Instruction>){
        self.boolean_cmd("JLT", output);
    } 
    
    fn gt(&mut self, output : &mut Vec<Instruction>){
        self.boolean_cmd("JGT", output);
    } 
    
    fn eq(&mut self, output : &mut Vec<Instruction>){
        self.boolean_cmd("JEQ", output);
    } 
    
    fn boolean_cmd(&mut self, jmp_cmd: &str, output: &mut Vec<Instruction>){
        let return_label = format!("END_BOOL_{}", self.bool_symbol_counter);
        self.bool_symbol_counter += 1;                        
        // Use helper - stick the next PC for the next instruction to execute in R15
        // and call the helper
        output.push(Instruction::AInstruction { symbol: Some(return_label.to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("D".to_string()), comp:"A".to_string(), jump: None });
        output.push(Instruction::AInstruction { symbol: Some("R15".to_string()), value: None });
        output.push(Instruction::CInstruction { dest: Some("M".to_string()), comp:"D".to_string(), jump: None });
        
        output.push(Instruction::AInstruction { symbol: Some(format!("boolean_cmd_helper_{}", jmp_cmd)), value: None });
        output.push(Instruction::CInstruction { dest: None, comp:"0".to_string(), jump: Some("JMP".to_string()) });
        output.push(Instruction::LInstruction { symbol: return_label });
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