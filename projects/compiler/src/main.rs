mod compiler;
mod xmlcompiler;
mod vmcompiler;

// use std::process::Output;

use compiler::{ CompilationEngine };

fn main() {
    let filestem = "nand2tetris/projects/11/ComplexArrays/Main";
    let mut tokenizer = compiler::JackTokenizer::new(format!("{}.jack", filestem).as_str());
    let mut compiler = xmlcompiler::XMLCompilationEngine::new(tokenizer.parse().unwrap().into_iter().peekable());
    let xml = compiler.compile_class();
    let xml_writer = xmlcompiler::XMLWriter::new(filestem);
    xml_writer.write(&xml);
    let mut vmcompiler = vmcompiler::VMCompiler::new(tokenizer.parse().unwrap().into_iter().peekable());
    let vm_instructions = vmcompiler.compile_class();
    let writer = vmcompiler::VMWriter::new( filestem );
    writer.write(&vm_instructions);    
}
