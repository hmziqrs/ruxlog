use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: cargo run --bin generate_csrf <input>");
        std::process::exit(1);
    }

    use base64::prelude::*;
    let input = &args[1];
    let output = BASE64_STANDARD.encode(input).replace("=", "");
    println!("input: {}", input);
    println!("output: {}", output);
}
