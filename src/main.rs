use std::env;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    println!("Reading folder: {}", path);

    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        println!("{}", entry.path().display());
    }

    Ok(())
}
