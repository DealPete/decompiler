pub mod analyse;
pub mod defs;
pub mod graph;
pub mod chip8;
mod c8analyzer;

use chip8::arch::*;
use graph::flow_graph::{FlowGraph, CallGraph};

static PROGRAM: &str =
"#include \"api.h\"
#include <stdint.h>
#include <stdlib.h>

unsigned char memory[4096] = {
    // numerals
	0xf0, 0x90, 0x90, 0x90, 0xf0,	// 0
	0x20, 0x60, 0x20, 0x20, 0x70,	// 1
	0xf0, 0x10, 0xf0, 0x80, 0xf0,	// 2
	0xf0, 0x10, 0xf0, 0x10, 0xf0,	// 3
	0x90, 0x90, 0xf0, 0x10, 0x10,	// 4
	0xf0, 0x80, 0xf0, 0x10, 0xf0,	// 5
	0xf0, 0x80, 0xf0, 0x90, 0xf0,	// 6
	0xf0, 0x10, 0x20, 0x40, 0x40,	// 7
	0xf0, 0x90, 0xf0, 0x90, 0xf0,	// 8
	0xf0, 0x90, 0xf0, 0x10, 0xf0,	// 9
	0xf0, 0x90, 0xf0, 0x90, 0x90,	// A
	0xe0, 0x90, 0xe0, 0x90, 0xe0,	// B
	0xf0, 0x80, 0x80, 0x80, 0xf0,	// C
	0xe0, 0x90, 0x90, 0x90, 0xe0,	// D
	0xf0, 0x80, 0xf0, 0x80, 0xf0,	// E
	0xf0, 0x80, 0xf0, 0x80, 0x80,	// F

    // big numerals (for SuperChip8)
    0xff, 0xff, 0xc3, 0xc3, 0xc3,
    0xc3, 0xc3, 0xc3, 0xff, 0xff,   // 0
    0x18, 0x78, 0x78, 0x18, 0x18,
    0x18, 0x18, 0x18, 0xff, 0xff,   // 1
    0xff, 0xff, 0x03, 0x03, 0xff,
    0xff, 0xc0, 0xc0, 0xff, 0xff,   // 2
    0xff, 0xff, 0x03, 0x03, 0xff,
    0xff, 0x03, 0x03, 0xff, 0xff,   // 3
    0xc3, 0xc3, 0xc3, 0xc3, 0xff,
    0xff, 0x03, 0x03, 0x03, 0x03,   // 4
    0xff, 0xff, 0xc0, 0xc0, 0xff,
    0xff, 0x03, 0x03, 0xff, 0xff,   // 5
    0xff, 0xff, 0xc0, 0xc0, 0xff,
    0xff, 0xc3, 0xc3, 0xff, 0xff,   // 6 
    0xff, 0xff, 0x03, 0x03, 0x06,
    0x0c, 0x18, 0x18, 0x18, 0x18,   // 7
    0xff, 0xff, 0xc3, 0xc3, 0xff,
    0xff, 0xc3, 0xc3, 0xff, 0xff,   // 8
    0xff, 0xff, 0xc3, 0xc3, 0xff,
    0xff, 0x03, 0x03, 0xff, 0xff,   // 9
    0x7e, 0xff, 0xc3, 0xc3, 0xc3,
    0xff, 0xff, 0xc3, 0xc3, 0xc3,   // a
    0xfc, 0xfc, 0xc3, 0xc3, 0xfc,
    0xfc, 0xc3, 0xc3, 0xfc, 0xfc,   // b
    0x3c, 0xff, 0xc3, 0xc0, 0xc0,
    0xc0, 0xc0, 0xc3, 0xff, 0x3c,   // c
    0xfc, 0xfe, 0xc3, 0xc3, 0xc3,
    0xc3, 0xc3, 0xc3, 0xfe, 0xfc,   // d
    0xff, 0xff, 0xc0, 0xc0, 0xff,
    0xff, 0xc0, 0xc0, 0xff, 0xff,   // e
    0xff, 0xff, 0xc0, 0xc0, 0xff,
    0xff, 0xc0, 0xc0, 0xc0, 0xc0,   // f

    // unused 272 bytes
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

    // program
    {program}
};

int8_t V[16] = {
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
};

int16_t I = 0;

char* get_filename() {
    return \"{filename}\";
}

{functions}int run_game(void* data) {
{main}
\treturn 0;
}
";

static MAKEFILE: &str =
"TARGETS = code.c chip8/src/*.c
INCLUDES = `sdl2-config --cflags` -Ichip8/src
LIBS = `sdl2-config --libs` -lsodium

all: $(TARGETS)
\tcc -o game $(INCLUDES) $(LIBS) $(TARGETS)

clean:
\t$(RM) game

remove:
\t$(RM) game makefile code.c
";

fn main() {
    use std::env;
    use std::io::Read;
    use std::io::Write;
    use std::fs::File;
    use std::path::Path;

	if let Some(arg) = env::args().nth(1) {
        let input_file = arg.clone();
        let path = Path::new(&input_file);
        let stem = path.file_stem()
            .expect("argument has no stem").to_str().unwrap();

		let mut file = File::open(arg).expect(
			"Failed to open file.");
		let mut buffer = Vec::new();
		file.read_to_end(&mut buffer).expect(
			"Failed to read into buffer.");

        let result = analyse::analyse(
            &buffer, chip8::arch::Chip8 {},
            c8analyzer::Chip8Analyzer {}, 0);

        match result {
            Ok(graph) => {
                let mut file = File::create("code.c")
                    .expect("Couldn't create output file.");

                let call_graph = match graph.construct_call_graph() {
                    Err(err) => panic!(err),
                    Ok(call_graph) => call_graph
                };

                let source = match source_string(graph, call_graph, stem, buffer) {
                    Err(err) => panic!(err),
                    Ok(source) => source
                };
                
                file.write_all(source.as_bytes())
                    .expect("Couldn't write output file.");

                let mut makefile = File::create("makefile")
                    .expect("Couldn't create makefile.");

                makefile.write_all(MAKEFILE.as_bytes())
                    .expect("Couldn't write makefile.");
            },
            Err(error) => println!("{}", error)
        }
    } else {
		println!("usage: recompile <file-to-recompile>");
    }
}

fn source_string(graph: FlowGraph<Instruction>, mut call_graph: CallGraph, file_stem: &str, data: Vec<u8>) -> Result<String, String> {
    let mut data_string = String::new();

    for byte in data {
        data_string.push_str(format!("0x{:x}, ", byte).as_str());
    }

    /*
    let mut headers = String::new();

    for function in call_graph.iter().skip(1) {
        let first_offset = graph.initial_instruction(function[0])?.unwrap();
        headers.push_str(format!(
            "void f{:x}();\n", first_offset + 0x200).as_str());
    }*/

    let mut functions = String::new();
    let mut main = String::new();

    while let Some(function) = call_graph.pop() {
        if call_graph.len() > 0 {
            let address = graph.initial_instruction(function[0])?.unwrap() + 0x200;
            functions.push_str(format!("void f{:x}() {{\n", address).as_str());
            functions.push_str(compile_function(&graph, function)?.as_str());
            functions.push_str("}\n\n");
        } else {
            main = compile_function(&graph, function)?;
        }
    }

    Ok(PROGRAM.replace("{program}", &data_string)
              .replace("{filename}", file_stem)
              //.replace("{headers}", &headers)
              .replace("{functions}", &functions)
              .replace("{main}", &main))
}

fn compile_function(graph: &FlowGraph<Instruction>, function: Vec<usize>) -> Result<String, String> {
    let mut output = String::new();
    let mut node_outputs = Vec::new();

    for node in function {
        if let Some(offset) = graph.initial_instruction(node)? {
            node_outputs.push((offset, compile_node(&graph, node)));
        }
    }

    node_outputs.sort_by_key(|&(key, _)| key);

    for (_, node_output) in node_outputs {
        output.push_str(node_output.as_str());
    }

    Ok(output)
}

fn compile_node(graph: &FlowGraph<Instruction>, node: usize) -> String {
    let node_address = graph.initial_instruction(node).unwrap()
        .expect(format!("no instruction at node {}", node).as_str()) + 0x200;

    let mut output = format!("\nl{:x}:", node_address);
    
    for offset in graph.get_instructions_at(node) {
        let inst = graph.get_inst(*offset)
            .expect(format!("no instruction at offset {:x}", offset).as_str());

        output.push_str("\t");
        output.push_str( match inst.mnemonic {
            Mnemonic::LOW => "lores();\n".into(),
            Mnemonic::HIGH => "hires();\n".into(),
            Mnemonic::CLS => "clear_screen();\n".into(),
            Mnemonic::LD => load(inst.unpack_op1(), inst.unpack_op2()),
            Mnemonic::ADD => add(inst.unpack_op1(), inst.unpack_op2()),
            Mnemonic::SNE => skip(false, *offset, inst.unpack_op1(), inst.unpack_op2()),
            Mnemonic::SE => skip(true, *offset, inst.unpack_op1(), inst.unpack_op2()),
            Mnemonic::JP => jump(*offset, inst.unpack_op1(), inst.op2),
            Mnemonic::SKP => skip_key(true, *offset, inst.unpack_op1()),
            Mnemonic::SKNP => skip_key(false, *offset, inst.unpack_op1()),
            Mnemonic::DRW => draw(inst.unpack_op1(), inst.unpack_op2(), inst.unpack_op3()),
            Mnemonic::RND => random(inst.unpack_op1(), inst.unpack_op2()),
            Mnemonic::CALL => call(inst.unpack_op1()),
            Mnemonic::RET => "return;\n".into(),
            Mnemonic::EXIT => "exit(0);\n".into(),
            Mnemonic::SCL => "scroll_left();\n".into(),
            Mnemonic::SCR => "scroll_right();\n".into(),
            _ => panic!("unsupported mnemonic")
        }.as_str());
    }

    output
}

fn load(op1: Operand, op2: Operand) -> String {
    match op1 {
        Operand::I => match op2 {
            Operand::Address(address) =>
                format!("I = 0x{:x};\n", address),
            Operand::Numeral(n) =>
                format!("I = 5 * V[{}];\n", n),
            Operand::LargeNumeral(n) =>
                format!("I = 10 * V[{}] + 80;\n", n),
            _ => panic!("invalid operand for LD I, ?.") 
        },
        Operand::V(x) => match op2 {
            Operand::Byte(byte) =>
                format!("V[{}] = {};\n", x, byte),
            Operand::V(y) =>
                format!("V[{}] = V[{}];\n", x, y),
            Operand::KeyPress =>
                format!("V[{}] = wait_for_keypress();\n", x),
            _ => panic!("invalid operand for LD Vx, ?.")
        },
        _ => panic!("invalid operand for LD.")
    }
}

fn add(op1: Operand, op2: Operand) -> String {
    match op1 {
        Operand::I => match op2 {
            Operand::V(x) =>
                format!("I += V[{}];\n", x),
            _ => panic!("invalid operand")
        },
        Operand::V(x) => match op2 {
            Operand::V(y) =>
                format!("V[{}] += V[{}];\n", x, y),
            Operand::Byte(byte) =>
                format!("V[{}] += {};\n", x, byte),
            _ => panic!("Invalid operand")
        }
        _ => panic!("Invalid operand")
    }
}

fn jump(offset: usize, op1: Operand, op2: Option<Operand>) -> String {
    match op1 {
        Operand::Address(address) => {
            if address as usize == offset + 0x200 {
                "return 0;\n".into()
            } else {
                format!("goto l{:x};\n", address)
            }
        },
        _ => panic!("Invalid operand")
    }
}

fn skip(equal: bool, offset: usize, op1: Operand, op2: Operand) -> String {
    let address = offset + 0x200;
    let comparison = if equal { "==" } else { "!=" };

    match op1 {
        Operand::V(x) => match op2 {
            Operand::Byte(byte) => format!(
                "if (V[{}] {} {}) goto l{:x}; else goto l{:x};\n",
                x, comparison, byte, address + 4, address + 2),
            Operand::V(y) => format!(
                "if (V[{}] {} V[{}]) goto l{:x}; else goto l{:x};\n",
                x, comparison, y, address + 4, address + 2),
            _ => panic!("invalid operand for SE")
        },
        _ => panic!("invalid operand for S(N)E")
    }
}

fn skip_key(equal: bool, offset: usize, op1: Operand) -> String {
    let address = offset + 0x200;
    let comparison = if equal { "" } else { "!" };

    match op1 {
        Operand::V(x) => format!(
            "if ({}key_pressed(V[{}])) goto l{:x}; else goto l{:x};\n",
            comparison, x, address + 4, address + 2),
        _ => panic!("invalid operand for SK(N)P")
    }
}

fn draw(op1: Operand, op2: Operand, op3: Operand) -> String {
    let xpos = match op1 {
        Operand::V(x) => x,
        _ => panic!("invalid operand for DRW ?.")
    };

    let ypos = match op2 {
        Operand::V(y) => y,
        _ => panic!("invalid operand for DRW Vx, ?.")
    };

    let lines = match op3 {
        Operand::Byte(byte) => byte,
        _ => panic!("invalid operand for DRW Vx, Vy, ?.")
    };

    format!("V[0xf] = draw_sprite(memory + I, V[{}], V[{}], {});\n",
        xpos, ypos, lines)
}

fn random(op1: Operand, op2: Operand) -> String {
    let target = match op1 {
        Operand::V(x) => x,
        _ => panic!("invalid operand for RND ?.")
    };

    let mask = match op2 {
        Operand::Byte(byte) => byte,
        _ => panic!("invalid operand for RND Vx, ?.")
    };

    format!("V[{}] = random_int8() & {:#b};\n", target, mask)
}

fn call(op: Operand) -> String {
    let target = match op {
        Operand::Address(address) => address,
        _ => panic!("Invalid operand for CALL ?.")
    };

    format!("f{:x}();\n", target)
}
