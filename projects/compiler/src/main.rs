mod compiler;
mod xmlcompiler;
mod vmcompiler;

// use std::process::Output;

use compiler::{ CompilationEngine, JackTokenizer };
use xmlcompiler::{ XMLCompilationEngine, XMLWriter };
use vmcompiler::{ VMCompiler, VMWriter };

fn main() {
    let source_dir = std::env::args().nth(1).unwrap();
    let files = std::fs::read_dir(&source_dir).unwrap();
    
    let build_dir = std::env::args().nth(2).unwrap();
    let writer = VMWriter::new( &build_dir );
    let xml_writer = XMLWriter::new( &build_dir );
            
    for f in files {
        let path = f.unwrap().path();
        if path.extension().unwrap().to_str() == Some("jack") {
            println!("{:?}", &path);
            let filestem = path.file_stem().unwrap().to_str().unwrap();
            let mut tokenizer = JackTokenizer::new(path.to_str().unwrap());
            
            let mut xmlcompiler = XMLCompilationEngine::new(tokenizer.parse().unwrap().into_iter().peekable());
            let xml = xmlcompiler.compile_class();
            xml_writer.write(filestem, &xml);
            
            let mut vmcompiler = VMCompiler::new(tokenizer.parse().unwrap().into_iter().peekable());
            let vm_instructions = vmcompiler.compile_class();
            writer.write(filestem, &vm_instructions);            
        }
    }
}
