English | [中文](README.zh.md)

# 🐾 Paws

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE) [![Built for Kaku](https://img.shields.io/badge/Built_for-Kaku-blue)](https://github.com/tw93/kaku) [![Made with Lua & Rust](https://img.shields.io/badge/Made_with-Lua_&_Rust-orange)]() [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/MisterBrookT/paws/pulls) [![GitHub stars](https://img.shields.io/github/stars/MisterBrookT/paws?style=social)](https://github.com/MisterBrookT/paws)

> *Pause* when your agent needs you. *Play* while it works.

A terminal companion for AI coding agents. Paws gives you an immersive full-screen game to play while your agent is working — with a live status bar showing which of your agent sessions are running and which are done, so you flip back exactly when you want to.

Built for the overlooked moment in vibe coding: you want to stay near the terminal, but the agent is thinking and you have nothing to do.

## How it works

```
        press CMD+G                       a session finishes
  ┌──────────────────────┐          ┌──────────────────────────┐
  │  🎮 Game tab          │          │   ● 1 running  ✓ 1 done!  │  ← live HUD
  │  (full window)        │  CMD+G   │   (flashes when done)     │
  │  ● 2 running          │ ───────> │                           │
  └──────────────────────┘          └──────────────────────────┘
```

You switch with **CMD+G** whenever you like. A status HUD inside the game shows
every agent session's state (running / done) and **flashes when one finishes**,
so you never miss it — no surprise auto-jumping while you're mid-game.

### Key bindings

| Key | Action |
|-----|--------|
| **CMD+G** | First press: pick a game. After that: toggle agent ↔ game. |
| **CMD+SHIFT+P** | Re-open the picker to change the game. |

> `CMD+SHIFT+G` is intentionally avoided — Kaku already binds it (lazygit).

The game lives in a separate **tab**, so it's naturally full-window and immersive — your existing pane/split layout is never disturbed. Switching is **manual by design**: the HUD (plus an optional completion sound) tells you when to come back, and you stay in control.

## Design philosophy

Everything runs inside the terminal's own native extension layer. **No external controller scripts, no auto-switching magic, no shelling out to `kaku cli`.**

```
Kiro hook ─ one line: writes this session's state to /tmp/paws-sessions/<id>
       │
       ▼
Kaku Lua ─ CMD+G / CMD+SHIFT+P only: spawn + toggle tabs via wezterm.mux,
       │   state in wezterm.GLOBAL — all in-process
       ▼
Game tab ─ the `paws` wrapper hosts the game centered in a PTY and renders a
           live session-status HUD reading /tmp/paws-sessions/
```

The terminal owns the tabs, so tab control lives in the terminal's Lua layer — not in a script reaching in from outside. The agent's only job is to write its state to a file; nothing reaches in to move you around.

## Requirements

- [Kaku terminal](https://github.com/tw93/kaku) (WezTerm fork)
- [Kiro CLI](https://kiro.dev) (primary) or Claude Code (planned)
- A Rust toolchain (`cargo`) to build the `paws` launcher
- One or more terminal games — e.g. `brew install c2048 nudoku vitetris`

## Setup

### The easy way — let your agent install it

Paws ships an install skill. Clone the repo and just ask your AI coding agent:

> "Install Paws using the skill in `paws/skills/paws-install/SKILL.md`."

The agent merges the Lua into your Kaku config, wires the hooks, installs a game,
and tells you to reload. No manual editing. (Kiro reads `SKILL.md` natively;
Claude Code can read it too.)

### The manual way

1. Build the launcher: `cargo install --path .` (gives you `paws` on your PATH).
2. Add [`lua/paws.lua`](lua/paws.lua) to your `~/.config/kaku/kaku.lua` (before `return config`).
3. Wire [`hooks/kiro/paws-signal.sh`](hooks/kiro/paws-signal.sh) as `stop` and `userPromptSubmit` hooks in your Kiro agent config (use **absolute** paths, note the `done`/`busy` args). This just records each session's state for the HUD:
   ```json
   "hooks": {
     "stop":             [{ "command": "/absolute/path/to/paws-signal.sh done" }],
     "userPromptSubmit": [{ "command": "/absolute/path/to/paws-signal.sh busy" }]
   }
   ```
   (Optional: add a sound on `stop`, e.g. `afplay /System/Library/Sounds/Glass.aiff`, as a completion chime.)
4. `brew install vitetris` (Tetris) and/or `cargo install --git https://github.com/MisterBrookT/jump-high`, then reload Kaku (CMD+Shift+R) and press **CMD+G**.

## Roadmap

### Done
- [x] Tab-based switching (Lua `wezterm.mux`)
- [x] Game picker (CMD+G / CMD+SHIFT+P)
- [x] Rust wrapper: centered PTY + live session HUD
- [x] [Jump High](https://github.com/MisterBrookT/jump-high) — Jump King-style charge-jump game
- [x] 🌍 地球Online — real-life side quests (exercise, call a friend, go outside)
- [x] 🎲 Random rotation (switches game every 5 hours)
- [x] Install via agent skill

### Next
1. More games.
2. Claude Code support.
3. `brew install paws`.

## Design doc

Full rationale in [`docs/design.tex`](docs/design.tex).

## License

MIT
