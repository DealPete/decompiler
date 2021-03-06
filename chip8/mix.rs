pub mod analyse;
pub mod defs;
pub mod graph;
pub mod chip8;
mod c8analyzer;

use defs::main::Architecture;

fn main() {
    use std::env;
    use std::io::Read;
    use std::fs::File;

	if let Some(arg) = env::args().nth(1) {
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
                println!("{}", graph);
                chip8::arch::Chip8::print_listing(graph.listing());
            },
            Err(error) => println!("{}", error)
        }
    } else {
		println!("usage: dis <file-to-disassemble>");
    }
}
