mod parser;
use crate::parser::Parser;

fn main() {
    match do_everything() {
        Ok(()) => {},
        Err(e) => eprintln!("error: {}", e),
    }
}

fn do_everything() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    eprintln!("Args: {:?}", args);
    if args.len() != 2 {
        return Err("bad arguments.\nUsage: PROGNAME <FILE>".into());
    }

    let file_contents =
        std::fs::read_to_string(&args[1]).map_err(|e| format!("failed to open input file: {:?}", e))?;

    let mut parser = Parser::new("hello! world");
    eprintln!("{:?}", parser.get_word());

    Ok(())
}
