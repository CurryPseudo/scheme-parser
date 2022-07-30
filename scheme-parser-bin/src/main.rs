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
        match scheme_parser::tokenize(&file_content, &args.file_name) {
            Ok(tokens) => {
                println!("{:#?}", tokens);
            }
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
