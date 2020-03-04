use std::io::{self, prelude::*};

fn encode(_input: &[u8]) {
    println!("encoding");
}

fn help() {
    println!("help");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf).unwrap();
    let input = if *buf.last().unwrap() == b'\n' { &buf[..(buf.len() - 1)] } else { &buf[..] };
    match (
        args.contains(&"e".to_string()),
        args.contains(&"d".to_string()),
    ) {
        (true, _) => encode(&buf),
        (_, true) => { io::stdout().write(&base32::decode(&input)).and_then(|_| io::stdout().flush()).unwrap(); }
        _ => help(),
    }
}
