# Contributing to Paws

Thanks for your interest! There are two main ways to contribute: **adding a game** (most wanted) or **improving the Paws host** itself.

## Adding a game

Games are standalone terminal binaries. You can publish one anywhere and submit it to the [paws-games](https://github.com/interesting-vibe-coding/paws-games) plugin library, or add it directly to the registry here.

### Game binary contract

Paws spawns your game in a PTY and overlays a one-row HUD at the top:

```
row 0:    🐾 HUD — session status (painted by Paws, not your game)
row 1..N: your game output
```

Your game **must**:

| Rule | Details |
|------|---------|
| Leave row 0 alone | Paws paints the HUD there; writing to it causes flicker |
| Handle `SIGWINCH` | The user may resize the window; update your layout |
| Exit cleanly on stdin EOF | Paws closes the PTY when the user quits |
| Be a named binary on `PATH` | Discovery uses `which <cmd>` |

Paws sizes your PTY as `(cols, rows-1)` — one row shorter than the terminal — so your game never needs to know about the HUD.

### Adding your game to the registry

Edit [`registry.toml`](registry.toml):

```toml
[[game]]
id          = "your-game-id"         # unique, kebab-case
name        = "Your Game Name"
icon        = "🎮"
cmd         = "your-binary-name"     # must match the binary on PATH
install     = "cargo install ..."    # shown in the in-app install catalog
description = "One-line description shown in the picker"
```

Then open a PR. If the game is in a separate repo, include a link and verify that the install command works from a clean machine.

## Improving the Paws host

### Dev setup

```bash
git clone https://github.com/interesting-vibe-coding/paws
cd paws
cargo build
cargo test
```

### Code structure

| Path | Responsibility |
|------|---------------|
| `src/main.rs` | Game picker, PTY hosting, HUD rendering, session state |
| `src/lang.rs` | Multilingual UI strings (EN, ZH, JA, KO) |
| `lua/paws.lua` | Kaku keybindings (CMD+G, CMD+SHIFT+P) |
| `hooks/` | Agent hooks (Claude Code, Kiro, Codex CLI) |
| `registry.toml` | Bundled game definitions |

### Running tests

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

CI runs on every push and PR — make sure all three pass before submitting.

### Adding a language

Languages live in `src/lang.rs`. Copy an existing match arm (e.g. `"en"`) and translate the strings. Then add the new language code to `pick_language()`.

## Pull request checklist

- [ ] `cargo test` passes locally
- [ ] `cargo clippy` passes (no warnings)
- [ ] `cargo fmt` applied
- [ ] If adding a game: tested the install command on a clean machine
- [ ] PR description explains what changed and why

## Reporting bugs

Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.yml) — it asks for the OS, Paws version, and agent so we can reproduce quickly.
