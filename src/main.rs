use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io};
use std::time::Instant;

use blythia::builder::Builder;
use blythia::compiler::Compiler;
use blythia::lexer::Lexer;
use blythia::parser::Parser;
use blythia::type_checker::TypeChecker;


fn test2(file: &Path) {
    let f = fs::read_to_string(file);
    let Ok(content) = f else {
        panic!("Error while reading file: {:#?}" , f);
    };

    let file_name = file.file_name().unwrap().to_str().unwrap().to_string();
    println!("Starting compilation process for {}", file_name);

    let lexer = Lexer::new(&content,file_name.clone() );
    let mut parser = Parser::new(lexer, file_name.clone());

    println!("Meassuring parser time");
    let now = Instant::now();
    let mut ast = parser.parse();
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    println!("Meassuring type checking time");
    let now = Instant::now();
    let mut type_checker = TypeChecker::new();
    type_checker.prepare_ast(&mut ast);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    println!("Meassuring build program time");
    let mut op = Builder::new(file_name.clone());
    let now = Instant::now();
    let program = op.build_program(&mut ast);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    println!("Meassuring compile time");
    let now = Instant::now();
    let output = Compiler::compile_program(program);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    let mut outfile = PathBuf::new().join(".").join("out").join(file.file_name().unwrap());
    outfile.set_extension("asm");
    match fs::write(&outfile, output) {
        Ok(()) => {}
        Err(x) => panic!("Could not save file: {:#?}\nError: {:#?}", outfile, x),
    }
    let nasm_out = Command::new("nasm")
        .arg("-felf64")
        .arg("-gdwarf")
        .arg(outfile.to_str().unwrap())
        .output()
        .expect("Nasm failed to compile");
    println!("{}",String::from_utf8(nasm_out.stderr).unwrap());

    let mut binary = outfile.clone();
    binary.set_extension("");
    let mut o_file = outfile.clone();
    o_file.set_extension("o");
    let _ = Command::new("ld")
        .arg("-o")
        .arg(binary.to_str().unwrap())
        .arg(o_file.to_str().unwrap())
        .output()
        .expect("Nasm failed to compile");

}

fn main() {
    let mut args= env::args().peekable();
    args.next(); // consume program path
    // get program options
    while let Some(s) = args.peek() {
        if !s.starts_with("-") {
            break
        }
        let _s = args.next().unwrap();
    }

    // get command
    let Some(cmd) = args.next() else {
        panic!("Did not provide a command.");
    };
    if cmd == "com" {
        let mut com_flags: Vec<String> = Vec::new();
        // parse compile options
        while let Some(s) = args.peek() {
            if !s.starts_with("-") {
                break;
            }
            let s = args.next().unwrap();
            if s == "-r" {
                com_flags.push(s.to_string());
            }
        }
        while let Some(s) = args.peek() {
            let path = Path::new(s);
            test2(path);
            if com_flags.contains(&"-r".to_string()) {
                let mut  outfile = PathBuf::new()
                    .join(".")
                    .join("out")
                    .join(path.file_name().unwrap());
                outfile.set_extension("");
                println!("Run program '{}'", outfile.to_str().unwrap());
                let out = Command::new(&outfile)
                    .output()
                    .expect("Failed to run compiled program");
                io::stdout().write_all(&out.stdout).unwrap();
                println!("Finished program '{}' with status: {}", outfile.to_str().unwrap(), out.status);
            }
            args.next();

        }
    }
    else {
        println!("{}", cmd);
    }
    // test1();
    // test2();
}
