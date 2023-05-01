use std::env;
use std::path::{Path};
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

fn generate_asm(compiler: &mut Compiler, filename : &str) -> Vec<Instruction> {
    let parser = VMInstructionParser::new(filename);
    let vm_instructions = parser.parse().unwrap();
    compiler.compile(vm_instructions)    
}

fn compile_dir(mut compiler: &mut Compiler, path : &Path) -> Vec<Instruction> {
    // get all .vm files in that dir
    println!("Processing directory {:?}", path);
    let dir_entries = std::fs::read_dir(path).expect(format!("File {:?} not found", path).as_str());    
    dir_entries.into_iter().filter(|f| {
        f.as_ref().unwrap().path().as_path().extension().unwrap() == "vm"
    })
    .map(|dir_entry| {
        generate_asm(&mut compiler, dir_entry.unwrap().path().to_str().unwrap())
    })
    .flatten()
    .collect()
}

fn compile(path : &Path) {
    let mut compiler = Compiler::new();
    let mut instructions = compiler.generate_bootstrap();
    let filestem = path.file_stem().unwrap().to_str().unwrap();
    let mut compiled_instructions = compile_dir(&mut compiler, &path);
    instructions.append(&mut compiled_instructions);
    let mut target_file_stem = path.to_path_buf();
    target_file_stem.push(filestem);               
    write_asm(target_file_stem.to_str().unwrap(), &instructions);
    write_hack(target_file_stem.to_str().unwrap(), &instructions);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut path = Path::new(args[1].as_str());
    if path.is_file(){
        path = path.parent().unwrap();
    }
    compile(path);
}