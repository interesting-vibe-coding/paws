English | [中文](README.zh.md)

<div align="center">

# 🐾 Paws

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE) [![Built for Kaku](https://img.shields.io/badge/Built_for-Kaku-blue)](https://github.com/tw93/kaku) [![Made with Lua & Rust](https://img.shields.io/badge/Made_with-Lua_&_Rust-orange)]() [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/interesting-vibe-coding/paws/pulls) [![GitHub Stars](https://img.shields.io/github/stars/interesting-vibe-coding/paws?style=flat&color=yellow)](https://github.com/interesting-vibe-coding/paws/stargazers)

Play games while your AI agent works. A status HUD tells you when to come back.

</div>

<p align="center"><img src="docs/demo.gif" width="600" alt="Paws demo"></p>

A terminal companion for AI coding agents. Press CMD+G, pick a game, and play in a full-window tab while your agent thinks. A live HUD overlaid on the top row shows which sessions are running and flashes when one finishes — you switch back when you're ready.

## Use

| Key | Action |
|-----|--------|
| **CMD+G** | First press: open the game picker. After that: toggle agent ↔ game. |
| **CMD+SHIFT+P** | Re-open the picker to change game. |
| **CMD+H** | Open the Paws repo in your browser. |

The HUD shows session state (running / done) and flashes on completion. No auto-switching — you stay in control.

## Install

### 1. Let your agent do it (recommended)

> "Install Paws using the skill in `paws/skills/paws-install/SKILL.md`."

Supports **Kiro CLI**, **Claude Code**, and **Codex CLI** — see the [install skill](skills/paws-install/SKILL.md) for per-agent setup.

### 2. Homebrew

```bash
brew install --HEAD interesting-vibe-coding/paws/paws       # the paws binary
brew install --HEAD interesting-vibe-coding/paws/paws-games  # all three games
```

Full `brew tap interesting-vibe-coding/paws && brew install paws` is pending a tagged release — see [Formula/README.md](Formula/README.md) for details.

### 3. Manual

```bash
cargo install --path .                                       # build paws
cargo install --git https://github.com/interesting-vibe-coding/paws-games --bin jump-high
cargo install --git https://github.com/interesting-vibe-coding/paws-games --bin earth-online
cargo install --git https://github.com/interesting-vibe-coding/paws-games --bin tetris
```

Then add [`lua/paws.lua`](lua/paws.lua) to your `~/.config/kaku/kaku.lua` (before `return config`) and wire hooks for your agent (see [`hooks/`](hooks/) for reference configs). Reload Kaku (CMD+Shift+R).

## Games

| Game | Binary | Description |
|------|--------|-------------|
| 🐕 Dog Jump | `jump-high` | Jump King-style platformer — charge, aim, and pray |
| 🌍 Earth Online | `earth-online` | Real-life side quests to run while your agent works |
| 🧱 Tetris | `tetris` | Classic block-stacking with levels and scoring |

Don't see enough? Open **⤓ Install games** in the picker to browse the catalog and install more in place. The catalog is the [paws-games](https://github.com/interesting-vibe-coding/paws-games) plugin library — anyone can contribute a game.

## How it works

Agent hooks write session state to `/tmp/paws-sessions/` → Kaku Lua handles CMD+G (spawns/toggles a tab) → the `paws` host runs the chosen game in a PTY and renders the HUD on the top row. Games are standalone binaries discovered via a [registry](registry.toml).

For architecture details, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

More projects → [doabit.dev](https://doabit.dev) · License: MIT
