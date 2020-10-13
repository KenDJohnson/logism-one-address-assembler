use clap::{App, Arg};

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

mod token;
use token::Token;

mod parser;
use parser::*;

mod instructions;
use instructions::*;

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("One-Address CPU Assembler")
        .version("1.0")
        .about("Assembles input for use with the One-Address CPU")
        .arg(
            Arg::with_name("input")
                .help("input file to assemble")
                .required(true)
                .takes_value(true)
                .value_name("INPUT")
                .index(1),
        )
        .arg(
            Arg::with_name("data")
                .help("data output file")
                .short("d")
                .takes_value(true)
                .value_name("DATA"),
        )
        .arg(
            Arg::with_name("text")
                .help("text output file")
                .short("t")
                .takes_value(true)
                .value_name("TEXT"),
        )
        .get_matches();

    let input_file = Path::new(matches.value_of("input").unwrap());

    let data_out = if let Some(data) = matches.value_of("data") {
        PathBuf::from(data)
    } else {
        let mut data = input_file.to_path_buf();
        data.set_extension("dat");
        data
    };

    let text_out = if let Some(text) = matches.value_of("text") {
        PathBuf::from(text)
    } else {
        let mut text = input_file.to_path_buf();
        text.set_extension("mc");
        text
    };

    let input = fs::read_to_string(input_file)?;

    let mut parser = Parser::parse(&input).unwrap();

    let addressed = parser.address_program().unwrap();

    {
        let mut data_outfile = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&data_out)?;

        writeln!(data_outfile, "v2.0 raw")?;
        for byte in addressed.data_bytes() {
            writeln!(data_outfile, "{:02x}", byte)?;
        }
    }

    {
        let mut text_outfile = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&text_out)?;

        writeln!(text_outfile, "v2.0 raw")?;
        for instr in &addressed.text {
            writeln!(text_outfile, "{}", instr.hex_string())?;
        }
    }

    Ok(())
}
