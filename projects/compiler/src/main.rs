mod compiler;
mod xmlcompiler;
mod vmcompiler;

// use std::process::Output;

use compiler::{ CompilationEngine, JackTokenizer };
use xmlcompiler::{ XMLCompilationEngine, XMLWriter };
use vmcompiler::{ VMCompiler, VMWriter };

fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let files = std::fs::read_dir(dir).unwrap();
    for f in files {
        let path = f.unwrap().path();
        if path.extension().unwrap().to_str() == Some("jack") {
            println!("{:?}", &path);
            let filestem = format!("{}/{}", path.parent().unwrap().to_str().unwrap(), path.file_stem().unwrap().to_str().unwrap());
            let mut tokenizer = JackTokenizer::new(path.to_str().unwrap());
            
            let mut xmlcompiler = XMLCompilationEngine::new(tokenizer.parse().unwrap().into_iter().peekable());
            let xml = xmlcompiler.compile_class();
            let xml_writer = XMLWriter::new(filestem.as_str());
            xml_writer.write(&xml);
            
            let mut vmcompiler = VMCompiler::new(tokenizer.parse().unwrap().into_iter().peekable());
            let vm_instructions = vmcompiler.compile_class();
            let writer = VMWriter::new( filestem.as_str() );
            writer.write(&vm_instructions);            
        }
    }
}
