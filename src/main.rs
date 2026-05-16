use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Points};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use walkdir::WalkDir;

const DEFAULT_PATH: &str = "/Users/arthurthiele/Documents/Schule/Mappen";

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

/// Which input mode we're in: regular navigation, or typing a search query.
#[derive(Copy, Clone)]
enum Mode {
    Normal,
    Search,
}

/// Which view is shown in the left pane: the list, or the graph canvas.
#[derive(Copy, Clone)]
enum View {
    List,
    Graph,
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

    // Mode state: are we navigating or typing a search query?
    let mut mode = Mode::Normal;
    let mut query = String::new();
    let mut view = View::List;

    loop {
        terminal.draw(|frame| {
            // ─── Outer layout: header (1 row) / body (rest) / footer (1 row)
            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(frame.area());

            // ─── Inner layout: split the body into left (40%) and right (60%)
            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(outer[1]);

            // ─── HEADER: app name + folder path, on a cyan band
            let header = Paragraph::new(format!(" synapse · {} ", path)).style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_widget(header, outer[0]);

            // ─── Compute which notes match the current query
            let filtered: Vec<usize> = if query.is_empty() {
                (0..notes.len()).collect()
            } else {
                let q = query.to_lowercase();
                notes
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| n.name().to_lowercase().contains(&q))
                    .map(|(i, _)| i)
                    .collect()
            };

            // ─── LEFT: either the list view or the graph view
            match view {
                View::List => {
                    let items: Vec<ListItem> = filtered
                        .iter()
                        .map(|&i| ListItem::new(notes[i].name()))
                        .collect();

                    let title = if query.is_empty() {
                        format!(" Files — {} ", notes.len())
                    } else {
                        format!(" Files — {}/{} ", filtered.len(), notes.len())
                    };
                    let list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title(title))
                        .highlight_style(
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol("> ");

                    frame.render_stateful_widget(list, body[0], &mut list_state);
                }
                View::Graph => {
                    let count = filtered.len();
                    let selected = list_state.selected();
                    let title = format!(" Graph — {} notes ", count);

                    // For each filtered note, find its top-level folder (e.g. "Mathe").
                    let root = std::path::Path::new(path);
                    let folder_of: Vec<String> = filtered
                        .iter()
                        .map(|&i| {
                            notes[i]
                                .path
                                .strip_prefix(root)
                                .ok()
                                .and_then(|p| p.parent())
                                .and_then(|p| p.iter().next())
                                .map(|c| c.to_string_lossy().to_string())
                                .unwrap_or_else(|| String::from("(root)"))
                        })
                        .collect();

                    // Collect unique folder names in the order they first appear.
                    let mut unique_folders: Vec<String> = Vec::new();
                    for f in &folder_of {
                        if !unique_folders.contains(f) {
                            unique_folders.push(f.clone());
                        }
                    }

                    // Color palette — each folder gets one, cycling if there are many.
                    let palette = [
                        Color::Red,
                        Color::Green,
                        Color::Yellow,
                        Color::Blue,
                        Color::Magenta,
                        Color::Cyan,
                        Color::LightRed,
                        Color::LightGreen,
                        Color::LightYellow,
                        Color::LightBlue,
                        Color::LightMagenta,
                        Color::LightCyan,
                    ];

                    // Offsets that, when added to a center (x, y), form a small filled
                    // disc made of braille dots. Used to make each node visibly larger.
                    let normal_blob: &[(f64, f64)] = &[
                        (0.0, 0.0),
                        (0.04, 0.0),
                        (-0.04, 0.0),
                        (0.0, 0.04),
                        (0.0, -0.04),
                        (0.03, 0.03),
                        (-0.03, 0.03),
                        (0.03, -0.03),
                        (-0.03, -0.03),
                    ];
                    // Larger disc for the selected node so it pops.
                    let selected_blob: &[(f64, f64)] = &[
                        (0.0, 0.0),
                        (0.05, 0.0),
                        (-0.05, 0.0),
                        (0.0, 0.05),
                        (0.0, -0.05),
                        (0.04, 0.04),
                        (-0.04, 0.04),
                        (0.04, -0.04),
                        (-0.04, -0.04),
                        (0.08, 0.0),
                        (-0.08, 0.0),
                        (0.0, 0.08),
                        (0.0, -0.08),
                        (0.06, 0.06),
                        (-0.06, 0.06),
                        (0.06, -0.06),
                        (-0.06, -0.06),
                    ];

                    let canvas = Canvas::default()
                        .block(Block::default().borders(Borders::ALL).title(title))
                        .x_bounds([-1.3, 1.3])
                        .y_bounds([-1.3, 1.3])
                        .marker(Marker::Braille)
                        .paint(|ctx| {
                            // Each note becomes a small blob on a unit circle, colored by folder.
                            for i in 0..count {
                                let angle =
                                    2.0 * std::f64::consts::PI * (i as f64) / (count as f64);
                                let x = angle.cos();
                                let y = angle.sin();
                                let folder_idx = unique_folders
                                    .iter()
                                    .position(|f| f == &folder_of[i])
                                    .unwrap_or(0);
                                let (offsets, color) = if Some(i) == selected {
                                    (selected_blob, Color::White)
                                } else {
                                    (normal_blob, palette[folder_idx % palette.len()])
                                };
                                let coords: Vec<(f64, f64)> = offsets
                                    .iter()
                                    .map(|&(dx, dy)| (x + dx, y + dy))
                                    .collect();
                                ctx.draw(&Points {
                                    coords: &coords,
                                    color,
                                });
                            }
                        });
                    frame.render_widget(canvas, body[0]);
                }
            }

            // ─── RIGHT: preview of selected note (look up via filtered index)
            let (preview_text, preview_title) = match list_state.selected() {
                Some(i) if i < filtered.len() => {
                    let note = &notes[filtered[i]];
                    (note.contents.as_str(), format!(" {} ", note.path.display()))
                }
                _ => ("No file selected", String::from(" Preview ")),
            };

            let preview = Paragraph::new(preview_text)
                .block(Block::default().borders(Borders::ALL).title(preview_title))
                .wrap(Wrap { trim: false });

            frame.render_widget(preview, body[1]);

            // ─── FOOTER: key hints in normal mode, search bar in search mode
            let view_hint = match view {
                View::List => "g: graph",
                View::Graph => "g: list",
            };
            let (footer_text, footer_style) = match mode {
                Mode::Normal => (
                    format!(" /: search · ↑↓/jk: navigate · {} · q: quit ", view_hint),
                    Style::default().fg(Color::DarkGray),
                ),
                Mode::Search => (
                    format!(" /{}_ ", query),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            };
            let footer = Paragraph::new(footer_text).style(footer_style);
            frame.render_widget(footer, outer[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('/') => {
                        mode = Mode::Search;
                        query.clear();
                        list_state.select(Some(0));
                    }
                    KeyCode::Char('g') => {
                        view = match view {
                            View::List => View::Graph,
                            View::Graph => View::List,
                        };
                    }
                    KeyCode::Down | KeyCode::Char('j') => list_state.select_next(),
                    KeyCode::Up | KeyCode::Char('k') => list_state.select_previous(),
                    _ => {}
                },
                Mode::Search => match key.code {
                    KeyCode::Esc => {
                        mode = Mode::Normal;
                        query.clear();
                        list_state.select(Some(0));
                    }
                    KeyCode::Enter => {
                        mode = Mode::Normal;
                    }
                    KeyCode::Backspace => {
                        query.pop();
                        list_state.select(Some(0));
                    }
                    KeyCode::Char(c) => {
                        query.push(c);
                        list_state.select(Some(0));
                    }
                    _ => {}
                },
            }
        }
    }

    ratatui::restore();
    Ok(())
}
