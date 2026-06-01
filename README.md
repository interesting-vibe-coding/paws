English | [中文](README.zh.md)

<div align="center">

# 🐾 Paws

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE) [![Built for Kaku](https://img.shields.io/badge/Built_for-Kaku-blue)](https://github.com/tw93/kaku) [![Made with Lua & Rust](https://img.shields.io/badge/Made_with-Lua_&_Rust-orange)]() [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/MisterBrookT/paws/pulls) [![GitHub Stars](https://img.shields.io/github/stars/MisterBrookT/paws?style=flat&color=yellow)](https://github.com/MisterBrookT/paws/stargazers)

Play games while your AI agent works. A status HUD tells you when to come back.

</div>

<p align="center"><img src="docs/demo.gif" width="600" alt="Paws demo"></p>

A terminal companion for AI coding agents. You press CMD+G, pick a game, and play in a full-window tab while your agent thinks. A live HUD inside the game shows which sessions are running and flashes when one finishes — you switch back when you're ready.

## Use

| Key | Action |
|-----|--------|
| **CMD+G** | First press: pick a game. After that: toggle agent ↔ game. |
| **CMD+SHIFT+P** | Re-open the picker to change game. |

The HUD shows session state (running / done) and flashes on completion. No auto-switching.

## Install

**Let your agent do it:**

> "Install Paws using the skill in `paws/skills/paws-install/SKILL.md`."

**Manual fallback:**

1. `cargo install --path .`
2. Add [`lua/paws.lua`](lua/paws.lua) to your `~/.config/kaku/kaku.lua` (before `return config`).
3. Wire [`hooks/kiro/paws-signal.sh`](hooks/kiro/paws-signal.sh) as `stop` and `userPromptSubmit` hooks:
   ```json
   "hooks": {
     "stop":             [{ "command": "/absolute/path/to/paws-signal.sh done" }],
     "userPromptSubmit": [{ "command": "/absolute/path/to/paws-signal.sh busy" }]
   }
   ```
4. Install a game (`brew install vitetris` or `cargo install --git https://github.com/MisterBrookT/paws-games`), reload Kaku (CMD+Shift+R), press CMD+G.

## Games

Tetris · [Dog Jump](https://github.com/MisterBrookT/paws-games) · Pinball · Earth Online (real-life side quests) · Poetry · 🎲 Random rotation

## How it works

Hook writes session state to `/tmp/paws-sessions/` → Kaku Lua handles CMD+G (spawns/toggles a tab via `wezterm.mux`) → the `paws` wrapper hosts the game centered in a PTY and renders the live HUD.

Everything runs natively in the terminal's Lua layer. No external scripts, no auto-switching. You stay in control.

---

More projects → [doabit.dev](https://doabit.dev) · License: MIT
