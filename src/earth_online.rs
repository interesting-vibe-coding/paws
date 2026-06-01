use std::io;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

struct Quest {
    emoji: &'static str,
    title: &'static str,
    desc: &'static str,
    time: &'static str,
}

const QUESTS: &[Quest] = &[
    Quest { emoji: "🏃", title: "做5分钟运动", desc: "俯卧撑、平板支撑、或开合跳，选一个做5分钟", time: "5 min" },
    Quest { emoji: "📞", title: "给一个朋友打电话", desc: "随便聊几分钟，问问他们最近怎么样", time: "5 min" },
    Quest { emoji: "🚶", title: "下楼走走", desc: "出门散步10分钟，看看周围的世界", time: "10 min" },
    Quest { emoji: "💧", title: "喝杯水", desc: "站起来倒杯水，顺便伸个懒腰", time: "1 min" },
    Quest { emoji: "🧘", title: "冥想3分钟", desc: "闭眼，专注呼吸，什么都不想", time: "3 min" },
    Quest { emoji: "📖", title: "读几页书", desc: "拿起手边的书，读3-5页", time: "5 min" },
    Quest { emoji: "🎵", title: "听一首歌", desc: "完整地听一首喜欢的歌，不做别的", time: "4 min" },
    Quest { emoji: "🪟", title: "看看窗外", desc: "站到窗边，看看外面的天空和树", time: "2 min" },
    Quest { emoji: "✍️", title: "写三件感恩的事", desc: "拿张纸写下今天感恩的三件小事", time: "3 min" },
    Quest { emoji: "🧹", title: "整理桌面", desc: "花2分钟把桌上的东西归位", time: "2 min" },
    Quest { emoji: "🫁", title: "深呼吸10次", desc: "4秒吸气，7秒屏息，8秒呼气", time: "3 min" },
    Quest { emoji: "🤸", title: "拉伸一下", desc: "站起来做2分钟全身拉伸", time: "2 min" },
    Quest { emoji: "🌱", title: "给植物浇水", desc: "如果你有植物的话", time: "1 min" },
    Quest { emoji: "👀", title: "20-20-20护眼", desc: "看20英尺外的东西20秒，重复3次", time: "1 min" },
];

fn pseudo_random(seed: u64) -> usize {
    // Simple hash to get a pseudo-random index
    let h = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (h >> 33) as usize % QUESTS.len()
}

fn pick_quest() -> usize {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as u64;
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    pseudo_random(secs ^ nanos)
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut quest_idx = pick_quest();
    let mut start = Instant::now();
    let mut paused = false;
    let mut elapsed_when_paused = Duration::ZERO;

    loop {
        let elapsed = if paused {
            elapsed_when_paused
        } else {
            elapsed_when_paused + start.elapsed()
        };

        terminal.draw(|f| {
            let quest = &QUESTS[quest_idx];
            let area = f.area();

            // Background
            f.render_widget(
                Block::default().style(Style::default().bg(Color::Rgb(20, 20, 30))),
                area,
            );

            let content_height = 11u16;
            let content_width = 50u16.min(area.width.saturating_sub(4));
            let cx = area.x + area.width.saturating_sub(content_width) / 2;
            let cy = area.y + area.height.saturating_sub(content_height) / 2;
            let content_area = Rect::new(cx, cy, content_width, content_height);

            // Layout: title block, timer, controls
            let chunks = Layout::vertical([
                Constraint::Length(5), // quest box
                Constraint::Length(1), // spacer
                Constraint::Length(1), // timer
                Constraint::Length(1), // spacer
                Constraint::Length(1), // pause status
                Constraint::Length(1), // spacer
                Constraint::Length(1), // controls
            ])
            .split(content_area);

            // Quest box
            let title_line = Line::from(vec![
                Span::styled(
                    format!("  {} {} ", quest.emoji, quest.title),
                    Style::default()
                        .fg(Color::Rgb(255, 230, 180))
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            let desc_line = Line::from(Span::styled(
                quest.desc,
                Style::default().fg(Color::Rgb(180, 180, 200)),
            ));
            let time_line = Line::from(Span::styled(
                format!("⏱  {}", quest.time),
                Style::default().fg(Color::Rgb(130, 160, 180)),
            ));

            let quest_block = Paragraph::new(vec![
                Line::raw(""),
                title_line,
                Line::raw(""),
                desc_line,
                time_line,
            ])
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(80, 80, 120)))
                    .title(" 🌍 地球Online · Side Quest ")
                    .title_style(Style::default().fg(Color::Rgb(120, 180, 140))),
            );
            f.render_widget(quest_block, chunks[0]);

            // Timer
            let secs_total = elapsed.as_secs();
            let mins = secs_total / 60;
            let secs = secs_total % 60;
            let timer_text = format!("{}:{:02}", mins, secs);
            let timer = Paragraph::new(Line::from(Span::styled(
                timer_text,
                Style::default().fg(Color::Rgb(100, 140, 160)),
            )))
            .alignment(Alignment::Center);
            f.render_widget(timer, chunks[2]);

            // Pause indicator
            if paused {
                let pause_text = Paragraph::new(Line::from(Span::styled(
                    "⏸  已暂停",
                    Style::default().fg(Color::Rgb(200, 160, 80)),
                )))
                .alignment(Alignment::Center);
                f.render_widget(pause_text, chunks[4]);
            }

            // Controls
            let controls = Paragraph::new(Line::from(vec![
                Span::styled("r", Style::default().fg(Color::Rgb(180, 140, 200)).add_modifier(Modifier::BOLD)),
                Span::styled(" 换一个  ", Style::default().fg(Color::Rgb(100, 100, 120))),
                Span::styled("p", Style::default().fg(Color::Rgb(180, 140, 200)).add_modifier(Modifier::BOLD)),
                Span::styled(" 暂停  ", Style::default().fg(Color::Rgb(100, 100, 120))),
                Span::styled("q", Style::default().fg(Color::Rgb(180, 140, 200)).add_modifier(Modifier::BOLD)),
                Span::styled(" 退出", Style::default().fg(Color::Rgb(100, 100, 120))),
            ]))
            .alignment(Alignment::Center);
            f.render_widget(controls, chunks[6]);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => {
                        // Re-roll: pick a different quest
                        let old = quest_idx;
                        quest_idx = pick_quest();
                        if quest_idx == old {
                            quest_idx = (quest_idx + 1) % QUESTS.len();
                        }
                        // Reset timer
                        elapsed_when_paused = Duration::ZERO;
                        start = Instant::now();
                        paused = false;
                    }
                    KeyCode::Char('p') => {
                        if paused {
                            start = Instant::now();
                            paused = false;
                        } else {
                            elapsed_when_paused += start.elapsed();
                            paused = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
