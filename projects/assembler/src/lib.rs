use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::io::Write;
use std::fmt;

#[derive(Debug)]
pub enum Instruction {
    Comment { contents : String },
    LInstruction{ symbol : String },
    AInstruction{ symbol : Option<String>, value: Option<u32> },
    CInstruction{ dest: Option<String>, comp: String, jump: Option<String> }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::Comment { ref contents } => write!(f, "// {}", contents),
            Instruction::LInstruction { ref symbol } => write!(f, "({})", symbol),
            Instruction::AInstruction { symbol: Some(ref s), value: None } => write!(f, "@{}", s),            
            Instruction::AInstruction { symbol: None, value: Some(ref v) } => write!(f, "@{}", v),            
            Instruction::CInstruction { ref dest, ref comp, ref jump } => {
                if let Some(ref d) = dest {
                    write!(f, "{}=", d).ok();
                }
                write!(f, "{}", &comp).ok();
                if let Some(ref j) = jump {
                    write!(f, ";{}", j).ok();
                }  
                Ok(())
            },
            _ => { Err(std::fmt::Error) }
        }
    }
}

pub struct Parser {
    filename : String
}

impl Parser {

    pub fn new(filestem: &str) -> Parser {
        Parser {
            filename : format!("{}.asm", filestem)
        }
    }

    pub fn parse(&self) -> Result<Vec<Instruction>,std::io::Error> {
        let mut f = File::open(&self.filename).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let lines = contents.to_string()
                            .split('\n')
                            .filter_map(|s| Parser::read_instruction(s))
                            .collect();
        Ok(lines)
    }

    fn read_instruction(line : &str) -> Option<Instruction>{
        let mut cs = line.chars();
        let mut done = false;
        let mut output : Vec<char> = Vec::new();
        let mut comment = false;
        while !done {
            match cs.next() {
                Some(' ') => continue,
                Some('\r') => continue,
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
            Some(Parser::parse_instruction(String::from_iter(output)))
        }
    }

    fn parse_instruction(ins: String) -> Instruction {
        match ins.chars().nth(0){
            Some('@')=> {
                let mut iter = ins.chars();
                iter.next();
                let symbol = iter.collect::<String>();
                match symbol.parse::<u32>() {
                    Ok(value) => Instruction::AInstruction { symbol: None, value: Some(value) },
                    Err(_) => Instruction::AInstruction { symbol: Some(symbol), value: None }
                }            
            },
            Some('(') => {
                let mut iter = ins.chars();
                iter.next();
                let label_len = ins.len() - 2;
                let symbol = String::from( iter.take(label_len).collect::<String>() );
                Instruction::LInstruction { symbol }
            },
            _ => {
                let (dest,comp,jump) = {
                    let mut iter = ins.split(";");
                    let mut comp_dest_iter = iter.next().unwrap().split("=");
                    let jump = iter.next();
                    match (comp_dest_iter.next(), comp_dest_iter.next(), jump) {
                        (Some(d), Some(c), j) => (Some(d.to_string()), c.to_string(), j.map(|s| s.to_string())),
                        (Some(c), None, j) => (None, c.to_string(), j.map(|s| s.to_string())),
                        _ => panic!("Unexpected input {}", ins)
                    }
                };
                Instruction::CInstruction{ dest, comp, jump }
            }
        }
    }
}

pub struct Assembler {
    symbol_table : HashMap<String, u32>
}

impl Assembler {
    pub fn new() -> Assembler {
        let mut symbol_table = HashMap::<String, u32>::new();
        for r in 0..15 {
            symbol_table.insert(format!("R{}", r), r);
        }  
        symbol_table.insert("SCREEN".to_string(), 16384);
        symbol_table.insert("KBD".to_string(), 24576);
        symbol_table.insert("SP".to_string(), 0);
        symbol_table.insert("LCL".to_string(), 1);
        symbol_table.insert("ARG".to_string(), 2);
        symbol_table.insert("THIS".to_string(), 3);
        symbol_table.insert("THAT".to_string(), 4);
        symbol_table.insert("R13".to_string(), 13);
        symbol_table.insert("R14".to_string(), 14);
        symbol_table.insert("R15".to_string(), 15);
        Assembler { symbol_table }
    }

    pub fn assemble(&mut self, instructions : &Vec<Instruction>) -> Vec<u32> {
        self.populate_symbol_table(instructions);
        self.generate_binary_code(instructions)
    }

    fn populate_symbol_table(&mut self, instructions : &Vec<Instruction>){
        let mut line_num : u32 = 0;
        // pass 1 - handle label symbols
        let mut ins_iter = instructions.iter();
        while let Some(instruction) = ins_iter.next() {
            match instruction {
                Instruction::LInstruction{ symbol } => {
                    self.symbol_table.insert(symbol.clone(), line_num);
                    println!("{} - {}", symbol, line_num);
                },
                Instruction::Comment { contents: _ } => {},
                _ => {
                    line_num = line_num + 1;
                }
            }
        }

        // pass 2 - handle variable symbols
        ins_iter = instructions.iter();
        let mut token_num = 15;
        while let Some(instruction) = ins_iter.next() {
            match instruction {
                Instruction::AInstruction{ symbol: Some(symbol), value: _ } => {
                    if !self.symbol_table.contains_key(symbol){
                        token_num=token_num+1;
                        self.symbol_table.insert(symbol.clone(), token_num);
                    }
                },
                _ => {}
            }            
        }
    }

    fn generate_binary_code(&self, instructions : &Vec<Instruction>) -> Vec<u32> {
        fn dest_code(c : &Option<String>) -> u32 {
            match c.as_deref() {
                None => 0,
                Some("M") => 1,
                Some("D") => 2,
                Some("MD") => 3,
                Some("A") => 4,
                Some("AM") => 5,
                Some("AD") => 6,
                Some("ADM") => 7,
                _ => panic!("Unexpected dest value {:?}", c)
            }
        }
        
        fn jump_code(c : &Option<String>) -> u32 {
            match c.as_deref() {
                None => 0,
                Some("JGT") => 1,
                Some("JEQ") => 2,
                Some("JGE") => 3,
                Some("JLT") => 4,
                Some("JNE") => 5,
                Some("JLE") => 6,
                Some("JMP") => 7,
                _ => panic!("Unexpected jump value {:?}", c)
            }
        }
        
        fn comp_code(c : &String) -> u32 {
            match c.as_str() {
                "0" =>  0b0101010,
                "1" =>  0b0111111,
                "-1" => 0b0111010,
                "D" =>  0b0001100,
                "A" =>  0b0110000,
                "!D" => 0b0001101,
                "!A" => 0b0110001,
                "-D" => 0b0001111,
                "-A" => 0b0110011,
                "D+1"=> 0b0011111,
                "A+1"=> 0b0110111,
                "D-1"=> 0b0001110,
                "A-1"=> 0b0110010,
                "D+A"=> 0b0000010,
                "D-A"=> 0b0010011,
                "A-D"=> 0b0000111,
                "D&A"=> 0b0000000,
                "D|A"=> 0b0010101,
                "M" =>  0b1110000,
                "!M" => 0b1110001,
                "-M" => 0b1110011,
                "M+1"=> 0b1110111,
                "M-1"=> 0b1110010,
                "D+M"=> 0b1000010,
                "D-M"=> 0b1010011,
                "M-D"=> 0b1000111,
                "D&M"=> 0b1000000,
                "D|M"=> 0b1010101,
                _ => panic!("Unexpected comp code value {:?}", c)
            }
        }
        
        let mut output = Vec::<u32>::new();
        let mut iter = instructions.iter();
        while let Some(ins) = iter.next() {
            match ins {
                Instruction::AInstruction{ symbol : Some(symbol), value: _ } => output.push( *self.symbol_table.get(symbol).unwrap() ),
                Instruction::AInstruction{ symbol : None, value: v } => output.push( v.unwrap() ),
                Instruction::CInstruction{ dest, comp, jump } => {
                    let value : u32 = (0b111 << 13) + (comp_code(comp) << 6) + (dest_code(dest) << 3) + jump_code(jump);
                    output.push(value);
                }
                _ => {}            
            }
        }
        output
    }

}

pub struct AssemblyWriter {
    filename : String
}

impl AssemblyWriter {
    pub fn new(filestem : &str) -> AssemblyWriter {
        AssemblyWriter {
            filename : format!("{}.hack", filestem)
        }
    }

    pub fn write(&self, compiled : Vec<u32>) {
        fn u32_to_string(i : u32) -> String {
            let mut out = Vec::<char>::new();
            for n in 0..16 {
                if i & (1<<n) > 0 {
                    out.push('1');
                } else {
                    out.push('0');
                }
            }
            return out.into_iter().rev().collect();
        }
        
        let mut f = File::create(&self.filename).expect("unable to create file");
        let hack : Vec<String> = compiled.iter().map(|i| u32_to_string(*i)).collect();
        write!(f, "{}", hack.join("\r")).expect("Failed to write to file");    
    }   
}

