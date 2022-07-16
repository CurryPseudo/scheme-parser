use std::fs::read_to_string;

use clap::Parser;

#[derive(Parser)]
struct Args {
    file_name: String,
    #[clap(short, long)]
    non_colorful: bool,
    #[clap(short, long)]
    token: bool,
}
fn main() {
    let args = Args::parse();
    let file_content = read_to_string(&args.file_name).unwrap();
    if args.token {
        let (tokens, error) = scheme_parser::tokenize(&file_content, &args.file_name);
        if let Some(tokens) = tokens {
            println!("{:#?}", tokens);
        }
        if let Some(error) = error {
            print!("{}", error.with_color(!args.non_colorful));
        }
    } else {
        let (program, error) = scheme_parser::parse(&file_content, &args.file_name);
        if let Some(program) = program {
            println!("{:#?}", program);
        }
        if let Some(error) = error {
            print!("{}", error.with_color(!args.non_colorful));
        }
    }
}
