use std::env;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    println!("Searching for typst files in: {}", path);

    let mut typst_files: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(path) {
        let entry = entry?;

        if let Some(extension) = entry.path().extension() {
            if extension == "typ" {
                typst_files.push(entry.path().to_path_buf());
            }
        }
    }

    println!("Found {} typst files:", typst_files.len());
    for path in &typst_files {
        println!("  {}", path.display());
    }

    Ok(())
}
