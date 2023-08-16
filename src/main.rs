mod bytecode;
mod virtual_machine;
mod scanner;
mod parser;
mod precidence;
mod value;
mod object;

use std::rc::Rc;

use bytecode::*;
use virtual_machine::*;
use scanner::*;
use parser::*;
use value::*;
use object::*;



fn main() {

    let a = ByteCode::Add;
    let b = ByteCode::Add;

    println!("{}", a==b);


    let s = String::from("AAAAA");
    print!("{}", s.len());

    let mut scanner = Scanner::from_file("test.dpp").unwrap();
    let token_list = scanner.scan();
    println!("{:?}", token_list);
    let mut prev_line = -1;
    for x in token_list.iter()
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
    
    parser.get_chunk().write_file("test_out.asm");

    println!();
    println!("{}", parser.get_chunk().disassemble());
    let mut vm = VirtualMachine::new(parser.get_chunk().clone());
    vm.debug = false;
    vm.constants = parser.constants.clone();
    
    let r = vm.interpret();

    println!("\nConstants:");
    for x in &vm.constants {
        println!("{:?}", x);
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

}



// fn main() {
//     let f: f32 = 1.;
//     f.to_string();

//     let mut chunk = Chunk::default();
//     chunk.add(ByteCode::Constant(Constant::Float(6.)), 1);
//     chunk.add(ByteCode::Constant(Constant::Float(5.)), 1);
//     chunk.add(ByteCode::Constant(Constant::Float(1.)), 1);
//     chunk.add(ByteCode::Sub, 1);
//     chunk.add(ByteCode::Div, 1);
//     chunk.add(ByteCode::Out, 1);
//     chunk.add(ByteCode::Constant(Constant::Bool(true)), 1);
//     chunk.add(ByteCode::Constant(Constant::Ptr(246743)), 1);
//     println!("{}", chunk.disassemble());
//     chunk.write_file("./test_out.asm");
//     chunk = Chunk::from_file("./test_out.asm");
//     println!("[{:?}]", &chunk);
//     // println!("{}", chunk.disassemble());

//     let mut vm = VirtualMachine::new(chunk);
//     vm.debug = false;
//     _ = vm.interpret();   


//     // println!("Hello, world!");
// }
