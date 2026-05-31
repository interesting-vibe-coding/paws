English | [中文](README.zh.md)

# 🐾 Paws

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE) [![Built for Kaku](https://img.shields.io/badge/Built_for-Kaku-blue)](https://github.com/tw93/kaku) [![Made with Lua & Rust](https://img.shields.io/badge/Made_with-Lua_&_Rust-orange)]() [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/MisterBrookT/paws/pulls) [![GitHub stars](https://img.shields.io/github/stars/MisterBrookT/paws?style=social)](https://github.com/MisterBrookT/paws)

> *Pause* when your agent needs you. *Play* while it works.

A terminal companion for AI coding agents. Paws gives you an immersive full-screen game to play while your agent is working — and auto-switches back the moment it needs your input.

Built for the overlooked moment in vibe coding: you want to stay near the terminal, but the agent is thinking and you have nothing to do.

## How it works

```
       press CMD+G                    agent finishes a turn
  ┌──────────────────┐            ┌──────────────────────────┐
  │  🎮 Game tab      │  ───────>  │  🤖 Agent tab            │
  │  (full window)    │  <───────  │  (your split layout)     │
  └──────────────────┘   CMD+G     └──────────────────────────┘
```

### Key bindings

| Key | Action |
|-----|--------|
| **CMD+G** | First press: pick a game (🎲 Random / 2048 / Sudoku / Tetris). After that: toggle agent ↔ game. |
| **CMD+SHIFT+P** | Re-open the picker to change the game. |

> `CMD+SHIFT+G` is intentionally avoided — Kaku already binds it (lazygit).

### Modes

- **Manual** — press CMD+G to jump to the game. When any agent session finishes (`stop`), Paws brings you back to that session's tab.
- **Auto** — once a game is open, the agent sends you to it when it starts working (`userPromptSubmit`) and back when it finishes. Hands-free.

The game lives in a separate **tab**, so it's naturally full-window and immersive — your existing pane/split layout is never disturbed.

## Design philosophy

Everything runs inside the terminal's own native extension layer. **No external controller scripts, no temp files, no shelling out to `kaku cli`.**

```
Kiro hooks ─ one-line OSC 1337 user-var emitters (stop + userPromptSubmit)
       │
       ▼
Kaku Lua ─ the brain. Reacts via user-var-changed, switches tabs via wezterm.mux,
       │   state in wezterm.GLOBAL — all in-process
       ▼
Game tab ─ runs `paws`, a tiny Rust launcher that rotates among your
       │   installed games (a different one each day)
```

The terminal owns the tabs, so tab control lives in the terminal's Lua layer — not in a script reaching in from outside.

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
3. Wire [`hooks/kiro/paws-signal.sh`](hooks/kiro/paws-signal.sh) as `stop` and `userPromptSubmit` hooks in your Kiro agent config (use **absolute** paths, note the `done`/`busy` args):
   ```json
   "hooks": {
     "stop":             [{ "command": "/absolute/path/to/paws-signal.sh done" }],
     "userPromptSubmit": [{ "command": "/absolute/path/to/paws-signal.sh busy" }]
   }
   ```
4. `brew install c2048 nudoku vitetris`, then reload Kaku (CMD+Shift+R) and press **CMD+G**.

## Roadmap

### Done
- [x] Native tab-based switching (pure Lua, `wezterm.mux`, `wezterm.GLOBAL`)
- [x] Game picker via `InputSelector` (CMD+G first run; CMD+SHIFT+P to re-pick)
- [x] One-step install via agent skill
- [x] `paws` Rust launcher with daily game rotation (2048 / sudoku / tetris)

### Next (priority order)
1. **Verify auto-return on device** — confirm the OSC `user-var` signal from the
   Kiro hook reaches Kaku (`userPromptSubmit` → game, `stop` → agent).
2. **Pause overlay** — pause the game and show an overlay + countdown for auto-return.
3. **More & better games** — grow the curated set; drop 2048 once richer games land.
4. **Claude Code support** — notification / stop hooks.

## Design doc

Full rationale in [`docs/design.tex`](docs/design.tex).

## License

MIT
