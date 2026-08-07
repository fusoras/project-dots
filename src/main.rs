use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Hello, world!");

    if args.len() < 2 {
        println!("Uso: cargo run -- <comando>");
        return;
    }

    let comando_usuario = &args[1];

    let output = Command::new("sh")
        .arg("-c")
        .arg(comando_usuario)
        .output()
        .expect("Falló al ejecutar el comando");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}
