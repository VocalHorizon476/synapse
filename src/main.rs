use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use walkdir::WalkDir;

const DEFAULT_PATH: &str = "/Users/trthurthiele/Documents/Schule/Mappen";

// ─── A Note: one typst file, with its path and pre-loaded contents ────────────
struct Note {
    path: PathBuf,
    contents: String,
}

impl Note {
    /// Read a typst file from disk and build a Note from it.
    fn from_path(path: PathBuf) -> io::Result<Note> {
        let contents = fs::read_to_string(&path)?;
        Ok(Note { path, contents })
    }

    /// Just the filename (e.g. "Vektoren.typ"), not the full path.
    fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.display().to_string())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let path: &str = if args.len() > 1 {
        &args[1]
    } else {
        DEFAULT_PATH
    };

    // Collect all .typ files into Notes (read each file ONCE, up front)
    let mut notes: Vec<Note> = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if let Some(extension) = entry.path().extension() {
            if extension == "typ" {
                notes.push(Note::from_path(entry.path().to_path_buf())?);
            }
        }
    }

    let mut terminal = ratatui::init();
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(frame.area());

            // LEFT: list of file names
            let items: Vec<ListItem> = notes
                .iter()
                .map(|note| ListItem::new(note.name()))
                .collect();

            let title = format!(" Files — {} ", notes.len());
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, chunks[0], &mut list_state);

            // RIGHT: preview of selected note (no more file reads in the loop!)
            let (preview_text, preview_title) = match list_state.selected() {
                Some(i) => {
                    let note = &notes[i];
                    (note.contents.as_str(), format!(" {} ", note.path.display()))
                }
                None => ("No file selected", String::from(" Preview ")),
            };

            let preview = Paragraph::new(preview_text)
                .block(Block::default().borders(Borders::ALL).title(preview_title))
                .wrap(Wrap { trim: false });

            frame.render_widget(preview, chunks[1]);
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
