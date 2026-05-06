use std::env;
use std::io;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use walkdir::WalkDir;

const DEFAULT_PATH: &str = "/Users/trthurthiele/Documents/Schule/Mappen";

fn main() -> io::Result<()> {
    // 1. Collect the .typ files (unchanged)
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

    // 3. Track which list item is selected
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    // 4. Main loop
    loop {
        terminal.draw(|frame| {
            let items: Vec<ListItem> = typst_files
                .iter()
                .map(|path| ListItem::new(path.display().to_string()))
                .collect();

            let title = format!(" Synapse — {} files ", typst_files.len());
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, frame.area(), &mut list_state);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Down | KeyCode::Char('j') => list_state.select_next(),
                KeyCode::Up | KeyCode::Char('k') => list_state.select_previous(),
                _ => {}
            }
        }
    }

    ratatui::restore();
    Ok(())
}
