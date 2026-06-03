use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

#[derive(Clone, Copy, PartialEq)]
pub enum Lang {
    En,
    Zh,
    Ja,
    Ko,
}

const OPTIONS: &[(Lang, &str)] = &[
    (Lang::En, "English"),
    (Lang::Zh, "中文"),
    (Lang::Ja, "日本語"),
    (Lang::Ko, "한국어"),
];

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/paws/lang")
}

pub fn current() -> Lang {
    if let Ok(content) = fs::read_to_string(config_path()) {
        match content.trim() {
            "zh" => return Lang::Zh,
            "ja" => return Lang::Ja,
            "ko" => return Lang::Ko,
            "en" => return Lang::En,
            _ => {}
        }
    }
    // Fallback to $LANG env
    if let Ok(val) = std::env::var("LANG") {
        let v = val.to_lowercase();
        if v.starts_with("zh") {
            return Lang::Zh;
        } else if v.starts_with("ja") {
            return Lang::Ja;
        } else if v.starts_with("ko") {
            return Lang::Ko;
        }
    }
    Lang::En
}

pub fn save(lang: Lang) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let code = match lang {
        Lang::En => "en",
        Lang::Zh => "zh",
        Lang::Ja => "ja",
        Lang::Ko => "ko",
    };
    let _ = fs::write(path, code);
}

pub fn is_set() -> bool {
    config_path().exists()
}

pub fn pick_interactive() -> io::Result<Lang> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut selected: usize = 0;

    let result = loop {
        terminal.draw(|f| {
            let area = f.area();
            f.render_widget(
                Block::default().style(Style::default().bg(Color::Rgb(20, 20, 30))),
                area,
            );

            let box_w = 30u16.min(area.width.saturating_sub(4));
            let box_h = 10u16.min(area.height.saturating_sub(2));
            let cx = area.x + area.width.saturating_sub(box_w) / 2;
            let cy = area.y + area.height.saturating_sub(box_h) / 2;
            let content_area = Rect::new(cx, cy, box_w, box_h);

            let mut lines = vec![
                Line::raw(""),
                Line::from(Span::styled(
                    "🐾 Choose language",
                    Style::default()
                        .fg(Color::Rgb(255, 230, 180))
                        .add_modifier(Modifier::BOLD),
                )),
                Line::raw(""),
            ];

            for (i, (_, label)) in OPTIONS.iter().enumerate() {
                let style = if i == selected {
                    Style::default()
                        .fg(Color::Rgb(120, 220, 160))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(160, 160, 180))
                };
                let prefix = if i == selected { "▸ " } else { "  " };
                lines.push(Line::from(Span::styled(
                    format!("{}{}", prefix, label),
                    style,
                )));
            }

            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(
                "↑↓ select · Enter confirm",
                Style::default().fg(Color::Rgb(100, 100, 120)),
            )));

            let para = Paragraph::new(lines).alignment(Alignment::Center).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(80, 80, 120))),
            );
            f.render_widget(para, content_area);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Up => {
                        selected = if selected == 0 {
                            OPTIONS.len() - 1
                        } else {
                            selected - 1
                        };
                    }
                    KeyCode::Down => {
                        selected = (selected + 1) % OPTIONS.len();
                    }
                    KeyCode::Enter => {
                        break OPTIONS[selected].0;
                    }
                    _ => {}
                }
            }
        }
    };

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    save(result);
    Ok(result)
}
