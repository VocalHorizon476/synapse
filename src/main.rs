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

fn main() -> io::Result<()> {
    // 1. Collect the .typ files
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

    let mut list_state = ListState::default();
    list_state.select(Some(0));

    // 3. Main loop
    loop {
        terminal.draw(|frame| {
            // Split the screen horizontally: 40% left, 60% right
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(frame.area());

            // ─── LEFT PANE: file list ─────────────────────────────────
            let items: Vec<ListItem> = typst_files
                .iter()
                .map(|path| {
                    let label = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.display().to_string());
                    ListItem::new(label)
                })
                .collect();

            let title = format!(" Files — {} ", typst_files.len());
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

            // ─── RIGHT PANE: preview ──────────────────────────────────
            let (preview_text, preview_title) = match list_state.selected() {
                Some(i) => {
                    let contents = fs::read_to_string(&typst_files[i])
                        .unwrap_or_else(|e| format!("Error reading file: {}", e));
                    let title = format!(" {} ", typst_files[i].display());
                    (contents, title)
                }
                None => (String::from("No file selected"), String::from(" Preview ")),
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
