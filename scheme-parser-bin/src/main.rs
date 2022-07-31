use std::fs::read_to_string;

use clap::Parser;
use scheme_parser::datumize;

#[derive(Parser)]
struct Args {
    file_name: String,
    #[clap(short, long)]
    non_colorful: bool,
    #[clap(short, long)]
    token: bool,
    #[clap(short, long)]
    datum: bool,
}
fn main() {
    let args = Args::parse();
    let file_content = read_to_string(&args.file_name).unwrap();
    if args.token {
        match scheme_parser::tokenize(&file_content, &args.file_name) {
            Ok(tokens) => {
                println!("{:#?}", tokens);
            }
            Err(error) => {
                print!("{}", error.with_color(!args.non_colorful));
            }
        }
    } else if args.datum {
        match scheme_parser::tokenize(&file_content, &args.file_name) {
            Ok(tokens) => match datumize(&tokens, &file_content, &args.file_name) {
                Ok(datums) => {
                    println!("{:#?}", datums);
                }
                Err(error) => print!("{}", error.with_color(!args.non_colorful)),
            },
            Err(error) => {
                print!("{}", error.with_color(!args.non_colorful));
            }
        }
    } else {
        match scheme_parser::Parser::default().parse(&file_content, &args.file_name) {
            Ok(program) => {
                println!("{:#?}", program);
            }
            Err(error) => {
                print!("{}", error.with_color(!args.non_colorful));
            }
        }
    }
}
