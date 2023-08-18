mod bytecode;
mod virtual_machine;
mod scanner;
mod parser;
mod precidence;
mod value;
mod object;
mod native_functions;
mod helper;

use bytecode::*;
use virtual_machine::*;
use scanner::*;
use parser::*;
use value::*;
use object::*;
use std::time::{Duration,Instant};

fn main() {

    let mut scanner = Scanner::from_file("test.dpp").unwrap();
    let token_list = scanner.scan();
    println!("{:?}", token_list);
    let mut prev_line = -1;
    for x in token_list
                        .iter()
                        .filter(|x| if let Token::Empty = x.token {false} else {true}) {
        let token = &x.token;
        let line = x.line;
        if line as i32 != prev_line {
            println!();
            prev_line = line as i32;
        }
        println!("{}\t{:>?}", line, token);
    }

    let mut parser = Parser::from_tokens(token_list);
    parser.compile();
    
    // parser.get_chunk().write_file("test_out.asm");

    println!();
    println!("{}", parser.get_chunk().disassemble());
    let mut vm = VirtualMachine::from_parser(&parser);
    vm.debug = false;
    vm.constants = parser.constants.clone();
    vm.write_file_detail("test_out.asm");
    let now = Instant::now();
    _ = vm.interpret();
    println!("{} ms", now.elapsed().as_nanos() as f64 / 1000. / 1000.);
    println!("{} s", now.elapsed().as_secs() as f64 / 1000. / 1000.);
    println!("\nConstants:");
    for x in &vm.constants {
        if let Value::Obj(c) = x {
            println!("{}", vm.obj_list[*c].to_str());
        } else {
            println!("{:?}", x);
        }
    }

    println!("\nGlobal:");
    for x in &vm.global {
        println!("{:?}", x);
    }


    println!("\nStack:");
    for x in &vm.stack {
        println!("{:?}", x);
    }
    println!();

    for func in vm.functions {
        println!("{:>?} {:?} {:?}", func.name, func.arity, func.chunk.len());
    }

}
