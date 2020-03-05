use std::{fs, io::{self, prelude::*}};

fn encode(_input: &[u8]) {
    println!("encoding");
}

fn help() {
    println!("help");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(args.len() >= 2);
    let input = std::fs::read(&args[0]).unwrap();
    fs::write(&args[1], base32::decode(&input)).unwrap();
    //let mut buf = Vec::new();
    //io::stdin().read_to_end(&mut buf).unwrap();
    //let input = if *buf.last().unwrap() == b'\n' { &buf[..(buf.len() - 1)] } else { &buf[..] };
    //match (
    //    args.contains(&"e".to_string()),
    //    args.contains(&"d".to_string()),
    //) {
    //    (true, _) => encode(&buf),
    //    (_, true) => { io::stdout().write(&base32::decode(&input)).and_then(|_| io::stdout().flush()).unwrap(); }
    //    _ => help(),
    //}
}
