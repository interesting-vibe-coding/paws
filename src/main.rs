mod lang;

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crossterm::{
    event::{
        self, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    terminal::{
        disable_raw_mode, enable_raw_mode, size as term_size, supports_keyboard_enhancement,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

const SESSIONS_DIR: &str = "/tmp/paws-sessions";
const POLL_MS: u64 = 33;
const STALE_SECS: u64 = 2 * 3600;
const DEFAULT_ROTATE_HOURS: u64 = 5;

struct Game {
    name: &'static str,
    cmd: &'static str,
    icon: &'static str,
    brew_hint: &'static str,
}

const GAMES: &[Game] = &[
    Game { name: "Tetris", cmd: "tetris", icon: "🎮", brew_hint: "brew install vitetris" },
    Game { name: "Dog Jump", cmd: "jump-high", icon: "🐕", brew_hint: "cargo install --git https://github.com/MisterBrookT/paws-games" },
    Game { name: "Pinball", cmd: "pinball", icon: "🕹️", brew_hint: "cargo install --git https://github.com/MisterBrookT/paws-games" },
    Game { name: "Earth Online", cmd: "earth-online", icon: "🌍", brew_hint: "cargo install --git https://github.com/MisterBrookT/paws-games" },
    Game { name: "Poetry", cmd: "poetry", icon: "🪶", brew_hint: "cargo install --git https://github.com/MisterBrookT/paws-games" },
];

fn is_installed(cmd: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
        .unwrap_or(false)
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

/// True if the process is still running (used to count only live agent sessions).
fn pid_alive(pid: i32) -> bool {
    pid > 0 && unsafe { libc::kill(pid, 0) } == 0
}

fn pick_index(bucket: u64, count: usize) -> usize {
    (bucket as usize) % count
}

fn rotate_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/paws/rotate_hours")
}

fn rotate_hours() -> u64 {
    fs::read_to_string(rotate_path())
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|h| h.clamp(1, 24))
        .unwrap_or(DEFAULT_ROTATE_HOURS)
}

fn save_rotate_hours(h: u64) {
    let path = rotate_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, h.clamp(1, 24).to_string());
}

/// Pick a game for "Random", rotating by the user-configured interval.
fn resolve_random(installed: &[&Game]) -> String {
    let bucket = now_secs() / (rotate_hours() * 3600);
    installed[pick_index(bucket, installed.len().max(1))].cmd.to_string()
}

fn main() -> io::Result<()> {
    let list_mode = env::args().any(|a| a == "--list");

    if !list_mode && !lang::is_set() {
        lang::pick_interactive()?;
    }

    if list_mode {
        println!("Paws game list:");
        for g in GAMES {
            let status = if is_installed(g.cmd) { "✓" } else { "✗" };
            println!("  [{status}] {:<14} cmd: {:<12} install: {}", g.name, g.cmd, g.brew_hint);
        }
        return Ok(());
    }

    let installed: Vec<&Game> = GAMES.iter().filter(|g| is_installed(g.cmd)).collect();

    // Game choice: explicit arg (e.g. `paws jump-high`), else the centered menu.
    let explicit = env::args().nth(1).filter(|a| !a.starts_with('-'));
    let game_cmd: String = if let Some(cmd) = explicit {
        cmd
    } else {
        if installed.is_empty() {
            println!("🐾 No games installed yet. Install one to play:");
            for g in GAMES {
                println!("  {} → {}", g.name, g.brew_hint);
            }
            return Ok(());
        }
        match pick_game_menu(&installed)? {
            Some(cmd) => cmd,
            None => return Ok(()),
        }
    };

    host_game(&game_cmd)
}

/// Centered, keyboard-driven menu: games + Random + Settings. Returns the chosen
/// game command, or None if the user backed out.
fn pick_game_menu(installed: &[&Game]) -> io::Result<Option<String>> {
    enum Item {
        Game(&'static str, String, String),
        Random,
        Settings,
    }
    let mut items: Vec<Item> = installed
        .iter()
        .map(|g| Item::Game(g.icon, g.name.to_string(), g.cmd.to_string()))
        .collect();
    items.push(Item::Random);
    items.push(Item::Settings);

    let labels: Vec<String> = items
        .iter()
        .map(|it| match it {
            Item::Game(icon, name, _) => format!("{icon}  {name}"),
            Item::Random => "🎲  Random".to_string(),
            Item::Settings => "⚙   Settings".to_string(),
        })
        .collect();

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut selected = 0usize;
    let mut in_settings = false;
    let mut hours = rotate_hours();

    let result = loop {
        terminal.draw(|f| {
            if in_settings {
                draw_settings(f, hours);
            } else {
                draw_menu(f, &labels, selected);
            }
        })?;

        if !event::poll(Duration::from_millis(120))? {
            continue;
        }
        let Event::Key(key) = event::read()? else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if in_settings {
            match key.code {
                KeyCode::Left | KeyCode::Char('-') | KeyCode::Char('h') => {
                    hours = (hours - 1).clamp(1, 24);
                    save_rotate_hours(hours);
                }
                KeyCode::Right | KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('l') => {
                    hours = (hours + 1).clamp(1, 24);
                    save_rotate_hours(hours);
                }
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => in_settings = false,
                _ => {}
            }
            continue;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                selected = if selected == 0 { labels.len() - 1 } else { selected - 1 };
            }
            KeyCode::Down | KeyCode::Char('j') => {
                selected = (selected + 1) % labels.len();
            }
            KeyCode::Enter => match &items[selected] {
                Item::Game(_, _, cmd) => break Some(cmd.clone()),
                Item::Random => break Some(resolve_random(installed)),
                Item::Settings => in_settings = true,
            },
            KeyCode::Esc | KeyCode::Char('q') => break None,
            _ => {}
        }
    };

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(result)
}

fn draw_menu(f: &mut Frame, labels: &[String], selected: usize) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(Color::Rgb(18, 18, 26))), area);

    let box_w = 44u16.min(area.width.saturating_sub(2));
    let box_h = (labels.len() as u16 + 8).min(area.height.saturating_sub(2));
    let content = centered_rect(box_w, box_h, area);

    let mut lines = vec![
        Line::raw(""),
        Line::from(Span::styled(
            "🐾  P A W S",
            Style::default().fg(Color::Rgb(255, 200, 120)).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "your agent's coffee break",
            Style::default().fg(Color::Rgb(120, 120, 140)),
        )),
        Line::raw(""),
    ];

    for (i, label) in labels.iter().enumerate() {
        let (style, prefix) = if i == selected {
            (Style::default().fg(Color::Rgb(120, 220, 160)).add_modifier(Modifier::BOLD), "▸  ")
        } else {
            (Style::default().fg(Color::Rgb(180, 180, 195)), "   ")
        };
        lines.push(Line::from(Span::styled(format!("{prefix}{label}"), style)));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "↑↓ move · Enter play · q quit",
        Style::default().fg(Color::Rgb(100, 100, 120)),
    )));

    let para = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(90, 80, 120))),
    );
    f.render_widget(para, content);
}

fn draw_settings(f: &mut Frame, hours: u64) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(Color::Rgb(18, 18, 26))), area);

    let content = centered_rect(44u16.min(area.width.saturating_sub(2)), 11, area);
    let plural = if hours == 1 { "hour" } else { "hours" };
    let lines = vec![
        Line::raw(""),
        Line::from(Span::styled(
            "⚙  Settings",
            Style::default().fg(Color::Rgb(255, 200, 120)).add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "Random rotation",
            Style::default().fg(Color::Rgb(180, 180, 195)),
        )),
        Line::from(Span::styled(
            format!("every  {hours}  {plural}"),
            Style::default().fg(Color::Rgb(120, 220, 160)).add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "←  −     +  →     Enter back",
            Style::default().fg(Color::Rgb(100, 100, 120)),
        )),
    ];
    let para = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(90, 80, 120))),
    );
    f.render_widget(para, content);
}

fn host_game(game_cmd: &str) -> io::Result<()> {
    let (tcols, trows) = term_size().unwrap_or((80, 25));
    let gcols = tcols.max(20);
    let grows = trows.saturating_sub(1).max(10); // reserve the top row for the HUD

    let pty_system = NativePtySystem::default();
    let pair = pty_system
        .openpty(PtySize { rows: grows, cols: gcols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut cmd = CommandBuilder::new(game_cmd);
    cmd.env("TERM", "xterm-256color");
    let _child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    drop(pair.slave);

    let mut pty_writer = pair
        .master
        .take_writer()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let mut pty_reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let parser = Arc::new(Mutex::new(vt100::Parser::new(grows, gcols, 0)));
    let parser_clone = Arc::clone(&parser);
    let running = Arc::new(AtomicBool::new(true));
    let running_reader = Arc::clone(&running);

    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match pty_reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => parser_clone.lock().unwrap().process(&buf[..n]),
            }
        }
        running_reader.store(false, Ordering::SeqCst);
    });

    // Enable the kitty protocol ONLY for jump-high (needs real key-release).
    enable_raw_mode()?;
    let kitty = game_cmd == "jump-high" && supports_keyboard_enhancement().unwrap_or(false);
    if kitty {
        let _ = io::stdout().execute(PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
        ));
    }

    // Raw stdin → PTY passthrough (preserves key-repeat / kitty sequences).
    std::thread::spawn(move || {
        let mut stdin = io::stdin();
        let mut buf = [0u8; 1024];
        loop {
            match stdin.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if pty_writer.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    let _ = pty_writer.flush();
                }
            }
        }
    });

    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let result = run_loop(&mut terminal, &parser, &running, &pair.master);

    if kitty {
        let _ = io::stdout().execute(PopKeyboardEnhancementFlags);
    }
    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    drop(pair.master);
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    parser: &Arc<Mutex<vt100::Parser>>,
    running: &Arc<AtomicBool>,
    master: &Box<dyn MasterPty + Send>,
) -> io::Result<()> {
    let (mut pcols, mut prows) = (0u16, 0u16);
    while running.load(Ordering::SeqCst) {
        let sz = terminal.size().unwrap_or(ratatui::layout::Size::new(80, 25));
        let gcols = sz.width.max(1);
        let grows = sz.height.saturating_sub(1).max(1);
        if gcols != pcols || grows != prows {
            let _ = master.resize(PtySize { rows: grows, cols: gcols, pixel_width: 0, pixel_height: 0 });
            if let Ok(mut p) = parser.lock() {
                p.set_size(grows, gcols);
            }
            pcols = gcols;
            prows = grows;
        }
        terminal.draw(|f| {
            draw_game(f, parser, grows, gcols);
            draw_hud(f);
        })?;
        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
    Ok(())
}

fn draw_game(f: &mut Frame, parser: &Arc<Mutex<vt100::Parser>>, rows: u16, cols: u16) {
    let area = f.area();
    // Game fills the screen below the HUD row; only this region gets a dark fill,
    // so the HUD row keeps the terminal's own background (no black block).
    let game_area = Rect::new(0, 1, cols.min(area.width), rows.min(area.height.saturating_sub(1)));
    f.render_widget(Block::default().style(Style::default().bg(Color::Black)), game_area);

    let screen = parser.lock().unwrap();
    let mut lines: Vec<Line> = Vec::with_capacity(game_area.height as usize);
    for row in 0..game_area.height {
        let mut spans: Vec<Span> = Vec::new();
        let mut col = 0u16;
        while col < game_area.width {
            let Some(cell) = screen.screen().cell(row, col) else { break };
            let ch = if cell.has_contents() { cell.contents() } else { " ".to_string() };
            let mut style = Style::default()
                .fg(vt_color_to_ratatui(cell.fgcolor()))
                .bg(vt_color_to_ratatui(cell.bgcolor()));
            if cell.bold() {
                style = style.add_modifier(Modifier::BOLD);
            }
            if cell.underline() {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            if cell.inverse() {
                style = style.add_modifier(Modifier::REVERSED);
            }
            let width = unicode_width(&ch).max(1) as u16;
            spans.push(Span::styled(ch, style));
            col += width;
        }
        lines.push(Line::from(spans));
    }
    f.render_widget(Paragraph::new(lines), game_area);
}

fn draw_hud(f: &mut Frame) {
    let Ok(entries) = fs::read_dir(SESSIONS_DIR) else { return };

    let (mut running, mut done) = (0u16, 0u16);
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(content) = fs::read_to_string(&path) else { continue };
        let mut parts = content.split_whitespace();
        let state = parts.next().unwrap_or("");
        let pid: i32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);

        if pid > 0 {
            // Closed session → process is gone: clean up and skip.
            if !pid_alive(pid) {
                let _ = fs::remove_file(&path);
                continue;
            }
        } else {
            // Legacy file without a PID: fall back to mtime staleness.
            let stale = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| SystemTime::now().duration_since(t).ok())
                .map(|d| d.as_secs() > STALE_SECS)
                .unwrap_or(false);
            if stale {
                continue;
            }
        }

        match state {
            "busy" => running += 1,
            "done" => done += 1,
            _ => {}
        }
    }

    if running == 0 && done == 0 {
        return;
    }

    let (working_label, waiting_label) = match lang::current() {
        lang::Lang::En => ("working", "waiting for you"),
        lang::Lang::Zh => ("运行中", "等你输入"),
        lang::Lang::Ja => ("実行中", "入力待ち"),
        lang::Lang::Ko => ("실행 중", "입력 대기"),
    };

    let ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut spans = vec![Span::styled("🐾 ", Style::default())];

    if running > 0 {
        // Animated spinner: motion = "still working".
        const SPIN: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = SPIN[((ms / 80) % 10) as usize];
        spans.push(Span::styled(
            format!("{frame} {running} {working_label}"),
            Style::default().fg(Color::Rgb(120, 200, 230)).add_modifier(Modifier::BOLD),
        ));
    }
    if running > 0 && done > 0 {
        spans.push(Span::raw("   "));
    }
    if done > 0 {
        // Settled (no motion) + gentle flash: "it's your turn".
        let fg = if (ms / 500) % 2 == 0 {
            Color::Rgb(245, 160, 50)
        } else {
            Color::Rgb(255, 245, 220)
        };
        spans.push(Span::styled(
            format!("✓ {done} {waiting_label}"),
            Style::default().fg(fg).add_modifier(Modifier::BOLD),
        ));
    }

    let line = Line::from(spans);
    let w = line.width() as u16 + 1;
    let area = f.area();
    let hud = Rect::new(area.width.saturating_sub(w), 0, w, 1);
    f.render_widget(Paragraph::new(vec![line]).alignment(Alignment::Right), hud);
}

fn centered_rect(w: u16, h: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w.min(area.width), h.min(area.height))
}

fn vt_color_to_ratatui(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn unicode_width(s: &str) -> usize {
    s.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_index_deterministic() {
        assert_eq!(pick_index(19874, 3), pick_index(19874, 3));
        assert_eq!((0..3).map(|d| pick_index(d, 3)).collect::<Vec<_>>(), vec![0, 1, 2]);
    }

    #[test]
    fn centered_rect_works() {
        let r = centered_rect(80, 24, Rect::new(0, 0, 100, 40));
        assert_eq!((r.x, r.y, r.width, r.height), (10, 8, 80, 24));
    }
}
