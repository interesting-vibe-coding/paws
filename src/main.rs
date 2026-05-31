// TODO: pause overlay + countdown (see README roadmap)

use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

struct Game {
    name: &'static str,
    cmd: &'static str,
    brew_hint: &'static str,
}

const GAMES: &[Game] = &[
    Game { name: "2048", cmd: "2048", brew_hint: "brew install c2048" },
    Game { name: "Nudoku (Sudoku)", cmd: "nudoku", brew_hint: "brew install nudoku" },
    Game { name: "Vitetris (Tetris)", cmd: "tetris", brew_hint: "brew install vitetris" },
];

fn is_installed(cmd: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| {
            env::split_paths(&paths).any(|dir| dir.join(cmd).is_file())
        })
        .unwrap_or(false)
}

fn epoch_day() -> u64 {
    // seconds since epoch / 86400
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 86400
}

fn pick_index(day: u64, count: usize) -> usize {
    (day as usize) % count
}

fn main() {
    if env::args().any(|a| a == "--list") {
        println!("Paws game list:");
        for g in GAMES {
            let status = if is_installed(g.cmd) { "✓" } else { "✗" };
            println!("  [{status}] {:<20} cmd: {:<10} install: {}", g.name, g.cmd, g.brew_hint);
        }
        return;
    }

    let installed: Vec<&Game> = GAMES.iter().filter(|g| is_installed(g.cmd)).collect();

    if installed.is_empty() {
        println!("🐾 No games installed! Install one to play:");
        for g in GAMES {
            println!("  {} → {}", g.name, g.brew_hint);
        }
        return;
    }

    let idx = pick_index(epoch_day(), installed.len());
    let game = installed[idx];

    let err = Command::new(game.cmd).exec();
    eprintln!("paws: failed to exec {}: {err}", game.cmd);
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_index_deterministic() {
        // Same day, same count → same result
        assert_eq!(pick_index(19874, 3), pick_index(19874, 3));
        // Different days can yield different indices
        let results: Vec<usize> = (0..3).map(|d| pick_index(d, 3)).collect();
        assert_eq!(results, vec![0, 1, 2]);
    }
}
