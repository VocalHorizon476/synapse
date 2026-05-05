use std::env;
use std::io;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::{Block, Borders, List, ListItem};
use walkdir::WalkDir;

const DEFAULT_PATH: &str = "/Users/trthurthiele/Documents/Schule/Mappen";

fn main() -> io::Result<()> {
    // 1. Collect the .typ files (same logic as before)
    let args: Vec<String> = env::args().collect();
    let path: &str = if args.len() > 1 {
        &args[1]
    } else {
        DEFAULT_PATH
    };

    let mut typst_files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if let Some(extension) = entry.path().extension() {
            if extension == "typ" {
                typst_files.push(entry.path().to_path_buf());
            }
        }
    }

    // 2. Take over the terminal
    let mut terminal = ratatui::init();

    // 3. Main loop: draw a frame, handle keys, repeat
    loop {
        terminal.draw(|frame| {
            let items: Vec<ListItem> = typst_files
                .iter()
                .map(|path| ListItem::new(path.display().to_string()))
                .collect();

            let title = format!(" Synapse — {} files ", typst_files.len());
            let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

            frame.render_widget(list, frame.area());
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    // 4. Give the terminal back
    ratatui::restore();

    Ok(())
}
