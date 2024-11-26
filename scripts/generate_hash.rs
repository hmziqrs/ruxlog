use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: cargo run --bin generate_hash <password>");
        std::process::exit(1);
    }

    let password = &args[1];
    let hash = password_auth::generate_hash(password);
    println!("Password: {}", password);
    println!("Hash: {}", hash);
}
