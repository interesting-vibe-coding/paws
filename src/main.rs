mod lang;

use std::env;
use std::fs;
use std::io::{self, BufRead, Read, Write};
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
use serde::Deserialize;

const SESSIONS_DIR: &str = "/tmp/paws-sessions";
const POLL_MS: u64 = 33;
const STALE_SECS: u64 = 2 * 3600;
const DEFAULT_ROTATE_HOURS: u64 = 5;

const BUNDLED_REGISTRY: &str = include_str!("../registry.toml");

#[derive(Deserialize, Clone)]
struct Game {
    #[allow(dead_code)]
    id: String,
    name: String,
    icon: String,
    cmd: String,
    install: String,
    description: String,
}

#[derive(Deserialize)]
struct Registry {
    game: Vec<Game>,
}

fn load_registry() -> Vec<Game> {
    let user_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/paws/registry.toml");
    if user_path.exists() {
        if let Ok(content) = fs::read_to_string(&user_path) {
            if let Ok(reg) = toml::from_str::<Registry>(&content) {
                return reg.game;
            }
        }
    }
    toml::from_str::<Registry>(BUNDLED_REGISTRY)
        .map(|r| r.game)
        .unwrap_or_default()
}

fn is_installed(cmd: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
        .unwrap_or(false)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

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

fn resolve_random(installed: &[&Game]) -> String {
    let bucket = now_secs() / (rotate_hours() * 3600);
    installed[pick_index(bucket, installed.len().max(1))]
        .cmd
        .clone()
}

fn find_stable_ancestor() -> i32 {
    let mut pid = unsafe { libc::getppid() } as i32;
    for _ in 0..2 {
        let Ok(out) = std::process::Command::new("ps")
            .args(["-o", "ppid=", "-p", &pid.to_string()])
            .output()
        else {
            break;
        };
        let ppid: i32 = String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse()
            .unwrap_or(0);
        if ppid <= 1 {
            break;
        }
        pid = ppid;
    }
    pid
}

fn scan_pgrep(pattern: &str) -> Vec<i32> {
    std::process::Command::new("pgrep")
        .args(["-x", pattern])
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|l| l.trim().parse::<i32>().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn get_ppid_of(pid: i32) -> i32 {
    std::process::Command::new("ps")
        .args(["-o", "ppid=", "-p", &pid.to_string()])
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(0)
}

fn handle_scan() -> io::Result<()> {
    let dir = PathBuf::from(SESSIONS_DIR);
    fs::create_dir_all(&dir)?;
    let my_pid = std::process::id() as i32;
    let mut seen = std::collections::HashSet::new();
    let mut count = 0u32;

    let write = |dir: &PathBuf, pid: i32, tmp_id: i32| -> io::Result<()> {
        let tmp = dir.join(format!(".tmp.{}", tmp_id));
        let target = dir.join(format!("scan-{}", pid));
        fs::write(&tmp, format!("busy {pid}"))?;
        fs::rename(&tmp, &target)?;
        Ok(())
    };

    for pattern in &["cfuse", "kiro-cli"] {
        for pid in scan_pgrep(pattern) {
            if pid == my_pid || !pid_alive(pid) {
                continue;
            }
            seen.insert(pid);
            write(&dir, pid, my_pid)?;
            count += 1;
        }
    }

    for pid in scan_pgrep("claude") {
        if pid == my_pid || !pid_alive(pid) {
            continue;
        }
        let ppid = get_ppid_of(pid);
        if seen.contains(&ppid) || ppid <= 1 {
            continue;
        }
        write(&dir, pid, my_pid)?;
        count += 1;
    }

    println!("🐾 Found {count} agent session(s)");
    Ok(())
}

fn handle_signal() -> io::Result<()> {
    let state = env::args().nth(2).unwrap_or_else(|| "busy".to_string());
    if state != "busy" && state != "done" {
        eprintln!("Usage: paws signal busy|done");
        std::process::exit(1);
    }
    let dir = PathBuf::from(SESSIONS_DIR);
    fs::create_dir_all(&dir)?;
    let pid = find_stable_ancestor();
    let sid = format!("signal-{}", pid);
    let tmp = dir.join(format!(".tmp.{}", std::process::id()));
    let target = dir.join(&sid);
    fs::write(&tmp, format!("{state} {pid}"))?;
    fs::rename(&tmp, &target)?;
    Ok(())
}

fn main() -> io::Result<()> {
    match env::args().nth(1).as_deref() {
        Some("signal") => return handle_signal(),
        Some("scan") => return handle_scan(),
        _ => {}
    }

    let games = load_registry();

    let list_mode = env::args().any(|a| a == "--list");

    if !list_mode && !lang::is_set() {
        lang::pick_interactive()?;
    }

    if list_mode {
        println!("Paws game list:");
        for g in &games {
            let status = if is_installed(&g.cmd) { "✓" } else { "✗" };
            println!(
                "  [{status}] {:<14} cmd: {:<12} install: {}",
                g.name, g.cmd, g.install
            );
        }
        return Ok(());
    }

    let explicit = env::args().nth(1).filter(|a| !a.starts_with('-'));
    let game_cmd: String = if let Some(cmd) = explicit {
        cmd
    } else {
        // Always open the picker — uninstalled games can be installed from it.
        match pick_game_menu(&games)? {
            Some(cmd) => cmd,
            None => return Ok(()),
        }
    };

    host_game(&game_cmd)
}

fn pick_game_menu(games: &[Game]) -> io::Result<Option<String>> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    enum Screen {
        Menu,
        Settings,
        Install,
    }
    let mut screen = Screen::Menu;
    let mut hours = rotate_hours();
    let mut selected = 0usize; // main-menu cursor
    let mut install_sel = 0usize; // install-catalog cursor

    let mut installed: Vec<bool> = games.iter().map(|g| is_installed(&g.cmd)).collect();

    let result = loop {
        // Main menu = installed games (playable) + Random + Settings + Install.
        let playable: Vec<usize> = (0..games.len()).filter(|&i| installed[i]).collect();
        let mut menu_labels: Vec<String> = playable
            .iter()
            .map(|&i| format!("{}  {}", games[i].icon, games[i].name))
            .collect();
        let n_play = playable.len();
        menu_labels.push("🎲  Random".to_string());
        menu_labels.push("⚙   Settings".to_string());
        menu_labels.push("⤓  Install games".to_string());
        let menu_len = menu_labels.len();
        if selected >= menu_len {
            selected = menu_len - 1;
        }

        terminal.draw(|f| {
            match screen {
                Screen::Settings => draw_settings(f, hours),
                Screen::Install => draw_install(f, games, &installed, install_sel),
                Screen::Menu => draw_menu(f, &menu_labels, selected),
            }
            draw_hud(f);
        })?;

        if !event::poll(Duration::from_millis(120))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match screen {
            Screen::Settings => match key.code {
                KeyCode::Left | KeyCode::Char('-') | KeyCode::Char('h') => {
                    hours = (hours - 1).clamp(1, 24);
                    save_rotate_hours(hours);
                }
                KeyCode::Right | KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('l') => {
                    hours = (hours + 1).clamp(1, 24);
                    save_rotate_hours(hours);
                }
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => screen = Screen::Menu,
                _ => {}
            },
            Screen::Install => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    install_sel = if install_sel == 0 {
                        games.len() - 1
                    } else {
                        install_sel - 1
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    install_sel = (install_sel + 1) % games.len();
                }
                KeyCode::Esc | KeyCode::Char('q') => screen = Screen::Menu,
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if !installed[install_sel] {
                        let game = &games[install_sel];
                        disable_raw_mode()?;
                        io::stdout().execute(LeaveAlternateScreen)?;

                        println!("\n  Installing {}…", game.name);
                        println!("  $ {}\n", game.install);

                        let status = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(&game.install)
                            .status();

                        match status {
                            Ok(s) if s.success() => println!("\n  ✓ {} installed!", game.name),
                            Ok(s) => {
                                println!("\n  ✗ Install failed (exit {})", s.code().unwrap_or(-1))
                            }
                            Err(e) => println!("\n  ✗ Install error: {e}"),
                        }
                        print!("  Press Enter to continue…");
                        let _ = io::stdout().flush();
                        let _ = io::stdin().lock().read_line(&mut String::new());

                        enable_raw_mode()?;
                        io::stdout().execute(EnterAlternateScreen)?;
                        terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
                        installed = games.iter().map(|g| is_installed(&g.cmd)).collect();
                    }
                }
                _ => {}
            },
            Screen::Menu => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = if selected == 0 {
                        menu_len - 1
                    } else {
                        selected - 1
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    selected = (selected + 1) % menu_len;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if selected < n_play {
                        break Some(games[playable[selected]].cmd.clone());
                    } else if selected == n_play {
                        let inst: Vec<&Game> = playable.iter().map(|&i| &games[i]).collect();
                        if !inst.is_empty() {
                            break Some(resolve_random(&inst));
                        }
                    } else if selected == n_play + 1 {
                        screen = Screen::Settings;
                    } else {
                        screen = Screen::Install;
                        install_sel = 0;
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => break None,
                _ => {}
            },
        }
    };

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(result)
}

fn draw_menu(f: &mut Frame, labels: &[String], selected: usize) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(56, 41, 28))),
        area,
    );

    let box_w = 44u16.min(area.width.saturating_sub(2));
    let box_h = (labels.len() as u16 + 9).min(area.height.saturating_sub(2));
    let content = centered_rect(box_w, box_h, area);

    let mut lines = vec![
        Line::raw(""),
        Line::from(Span::styled(
            "🐾  P A W S",
            Style::default()
                .fg(Color::Rgb(255, 200, 120))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "your agent's coffee break",
            Style::default().fg(Color::Rgb(165, 140, 115)),
        )),
        Line::raw(""),
    ];

    for (i, label) in labels.iter().enumerate() {
        let (style, prefix) = if i == selected {
            (
                Style::default()
                    .fg(Color::Rgb(255, 215, 140))
                    .add_modifier(Modifier::BOLD),
                "▸  ",
            )
        } else {
            (Style::default().fg(Color::Rgb(195, 175, 145)), "   ")
        };
        lines.push(Line::from(Span::styled(format!("{prefix}{label}"), style)));
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "↑↓ move · Enter/Space play · q quit",
        Style::default().fg(Color::Rgb(175, 150, 120)),
    )));
    lines.push(Line::from(Span::styled(
        "⌘G switch · ⌘⇧P re-pick · ⌘H help",
        Style::default().fg(Color::Rgb(155, 132, 105)),
    )));

    let para = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(150, 120, 70))),
    );
    f.render_widget(para, content);
}

/// The "Install games" catalog: every game in the registry, with its install
/// state. Installing happens here, inside Paws — the registry is the plugin index.
fn draw_install(f: &mut Frame, games: &[Game], installed: &[bool], selected: usize) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(56, 41, 28))),
        area,
    );

    let box_w = 52u16.min(area.width.saturating_sub(2));
    let box_h = (games.len() as u16 + 11).min(area.height.saturating_sub(2));
    let content = centered_rect(box_w, box_h, area);

    let mut lines = vec![
        Line::raw(""),
        Line::from(Span::styled(
            "⤓  Install games",
            Style::default()
                .fg(Color::Rgb(255, 200, 120))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "from the paws-games library",
            Style::default().fg(Color::Rgb(165, 140, 115)),
        )),
        Line::raw(""),
    ];

    for (i, g) in games.iter().enumerate() {
        let marker = if installed[i] {
            "✓ installed"
        } else {
            "⤓ install"
        };
        let label = format!("{}  {}   {}", g.icon, g.name, marker);
        let (style, prefix) = if i == selected {
            (
                Style::default()
                    .fg(Color::Rgb(255, 215, 140))
                    .add_modifier(Modifier::BOLD),
                "▸  ",
            )
        } else if installed[i] {
            (Style::default().fg(Color::Rgb(150, 170, 140)), "   ")
        } else {
            (Style::default().fg(Color::Rgb(195, 175, 145)), "   ")
        };
        lines.push(Line::from(Span::styled(format!("{prefix}{label}"), style)));
    }

    lines.push(Line::raw(""));
    if let Some(g) = games.get(selected) {
        lines.push(Line::from(Span::styled(
            g.description.clone(),
            Style::default().fg(Color::Rgb(150, 130, 105)),
        )));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "↑↓ move · Enter install · Esc back",
        Style::default().fg(Color::Rgb(175, 150, 120)),
    )));

    let para = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(150, 120, 70))),
    );
    f.render_widget(para, content);
}

fn draw_settings(f: &mut Frame, hours: u64) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(56, 41, 28))),
        area,
    );

    let content = centered_rect(44u16.min(area.width.saturating_sub(2)), 11, area);
    let plural = if hours == 1 { "hour" } else { "hours" };
    let lines = vec![
        Line::raw(""),
        Line::from(Span::styled(
            "⚙  Settings",
            Style::default()
                .fg(Color::Rgb(255, 200, 120))
                .add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "Random rotation",
            Style::default().fg(Color::Rgb(210, 190, 160)),
        )),
        Line::from(Span::styled(
            format!("every  {hours}  {plural}"),
            Style::default()
                .fg(Color::Rgb(255, 215, 140))
                .add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "←  −     +  →     Enter back",
            Style::default().fg(Color::Rgb(175, 150, 120)),
        )),
    ];
    let para = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(150, 120, 70))),
    );
    f.render_widget(para, content);
}

fn host_game(game_cmd: &str) -> io::Result<()> {
    let (tcols, trows) = term_size().unwrap_or((80, 25));
    let gcols = tcols.max(20);
    let grows = trows.saturating_sub(1).max(10);

    let pty_system = NativePtySystem::default();
    let pair = pty_system
        .openpty(PtySize {
            rows: grows,
            cols: gcols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| io::Error::other(e.to_string()))?;

    let mut cmd = CommandBuilder::new(game_cmd);
    cmd.env("TERM", "xterm-256color");
    let _child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| io::Error::other(e.to_string()))?;
    drop(pair.slave);

    let mut pty_writer = pair
        .master
        .take_writer()
        .map_err(|e| io::Error::other(e.to_string()))?;
    let mut pty_reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| io::Error::other(e.to_string()))?;

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

    enable_raw_mode()?;
    let kitty = game_cmd == "jump-high" && supports_keyboard_enhancement().unwrap_or(false);
    if kitty {
        let _ = io::stdout().execute(PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
        ));
    }

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

#[allow(clippy::borrowed_box)]
fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    parser: &Arc<Mutex<vt100::Parser>>,
    running: &Arc<AtomicBool>,
    master: &Box<dyn MasterPty + Send>,
) -> io::Result<()> {
    let (mut pcols, mut prows) = (0u16, 0u16);
    while running.load(Ordering::SeqCst) {
        let sz = terminal
            .size()
            .unwrap_or(ratatui::layout::Size::new(80, 25));
        let gcols = sz.width.max(1);
        let grows = sz.height.saturating_sub(1).max(1);
        if gcols != pcols || grows != prows {
            let _ = master.resize(PtySize {
                rows: grows,
                cols: gcols,
                pixel_width: 0,
                pixel_height: 0,
            });
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
    let game_area = Rect::new(
        0,
        1,
        cols.min(area.width),
        rows.min(area.height.saturating_sub(1)),
    );
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        game_area,
    );

    let screen = parser.lock().unwrap();
    let mut lines: Vec<Line> = Vec::with_capacity(game_area.height as usize);
    for row in 0..game_area.height {
        let mut spans: Vec<Span> = Vec::new();
        let mut col = 0u16;
        while col < game_area.width {
            let Some(cell) = screen.screen().cell(row, col) else {
                break;
            };
            let ch = if cell.has_contents() {
                cell.contents()
            } else {
                " ".to_string()
            };
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
    let Ok(entries) = fs::read_dir(SESSIONS_DIR) else {
        return;
    };

    let (mut running, mut done) = (0u16, 0u16);
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let mut parts = content.split_whitespace();
        let state = parts.next().unwrap_or("");
        let pid: i32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);

        if pid > 0 {
            if !pid_alive(pid) {
                let _ = fs::remove_file(&path);
                continue;
            }
            if state == "done" {
                done += 1;
            } else {
                running += 1;
            }
        } else {
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
            match state {
                "busy" => running += 1,
                "done" => done += 1,
                _ => {}
            }
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

    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let mut spans = vec![Span::styled("🐾 ", Style::default())];

    if running > 0 {
        const SPIN: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = SPIN[((ms / 80) % 10) as usize];
        spans.push(Span::styled(
            format!("{frame} {running} {working_label}"),
            Style::default()
                .fg(Color::Rgb(120, 200, 230))
                .add_modifier(Modifier::BOLD),
        ));
    }
    if running > 0 && done > 0 {
        spans.push(Span::raw("   "));
    }
    if done > 0 {
        let fg = if (ms / 500).is_multiple_of(2) {
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
        assert_eq!(
            (0..3).map(|d| pick_index(d, 3)).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn centered_rect_works() {
        let r = centered_rect(80, 24, Rect::new(0, 0, 100, 40));
        assert_eq!((r.x, r.y, r.width, r.height), (10, 8, 80, 24));
    }

    #[test]
    fn load_registry_parses_bundled() {
        let games = load_registry();
        assert_eq!(games.len(), 3);
        assert_eq!(games[0].cmd, "jump-high");
        assert_eq!(games[1].cmd, "earth-online");
        assert_eq!(games[2].cmd, "tetris");
    }
}
