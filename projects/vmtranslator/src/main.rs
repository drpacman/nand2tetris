use std::env;
// use std::io::Write;
use assembler::{ Assembler, AssemblyWriter, Instruction };
use vmtranslator::{Compiler, VMInstructionParser, ASMWriter};

fn write_asm(filestem : &str, instructions : &Vec<Instruction>) {
    let writer = ASMWriter::new(filestem);
    writer.write(instructions);
}
fn write_hack(filestem : &str, instructions : &Vec<Instruction>) {
    let mut assembler = Assembler::new();
    let assembly = assembler.assemble(&instructions);
    let assembly_writer = AssemblyWriter::new(filestem);
    assembly_writer.write(assembly);
}

fn generate_asm(filestem : &str) -> Vec<Instruction> {
    let parser = VMInstructionParser::new(filestem);
    let vm_instructions = parser.parse().unwrap();
    let mut compiler = Compiler::new();
    compiler.compile(vm_instructions)    
}

fn main() {
    // let filestem = "/Users/caporp01/workspace/nand2tetris/nand2tetris/projects/07/StackArithmetic/StackTest/StackTest";
    
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let filestem = args[1].as_str();
    let instructions = generate_asm(filestem);
    write_asm(filestem, &instructions);
    write_hack(filestem, &instructions);
}