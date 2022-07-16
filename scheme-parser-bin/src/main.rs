use std::fs::read_to_string;

use clap::Parser;

#[derive(Parser)]
struct Args {
    file_name: String,
    #[clap(short, long)]
    non_colorful: bool,
}
fn main() {
    let args = Args::parse();
    let file_content = read_to_string(&args.file_name).unwrap();
    match scheme_parser::parse(&file_content, &args.file_name) {
        Ok(program) => println!("{:#?}", program),
        Err(e) => {
            print!("{}", e.with_color(!args.non_colorful));
        }
    }
}
