# 🐾 Paws

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

- **CMD+G** — first press spawns the game in its **own tab** and switches to it; after that it toggles between the agent tab and the game tab.
- **Agent finishes** — it emits a signal and Paws auto-switches back to the agent tab so you can respond.

The game lives in a separate tab, so it's naturally full-window and immersive — and your existing pane/split layout is never disturbed.

## Design philosophy

Everything runs inside the terminal's own native extension layer. **No external controller scripts, no temp files, no shelling out to `kaku cli`.**

```
Kiro stop hook ─ one line: emits an OSC 1337 user var to the tty (the only signal)
       │
       ▼
Kaku Lua ─ the brain. Reacts to the user var, switches tabs via wezterm.mux,
       │   state in wezterm.GLOBAL — all in-process
       ▼
Game tab ─ runs the game (later: a Rust wrapper hosting many games)
```

The terminal owns the tabs, so tab control lives in the terminal's Lua layer — not in a script reaching in from outside.

## Requirements

- [Kaku terminal](https://github.com/tw93/kaku) (WezTerm fork)
- [Kiro CLI](https://kiro.dev) (primary) or Claude Code (planned)
- A terminal game, e.g. `brew install c2048`

## Setup

1. Add [`lua/paws.lua`](lua/paws.lua) to your `~/.config/kaku/kaku.lua` (before `return config`).
2. Wire [`hooks/kiro/paws-signal.sh`](hooks/kiro/paws-signal.sh) as a `stop` hook in your Kiro agent config:
   ```json
   "hooks": { "stop": [{ "command": "/absolute/path/to/paws-signal.sh" }] }
   ```
3. Reload Kaku (CMD+Shift+R). Press **CMD+G** to start playing.

## Roadmap

- [x] Native CMD+G spawn + toggle (pure Lua, `wezterm.mux`, tab-based)
- [x] Auto-switch-back via OSC user var + `user-var-changed`
- [ ] Rust wrapper: pause overlay + countdown + auto-return mode
- [ ] Multi-game rotation (daily / hourly random pick)
- [ ] Pet mode (ambient companion reacting to agent state)
- [ ] Claude Code support
- [ ] `brew install paws`

## Design doc

Full rationale in [`docs/design.tex`](docs/design.tex).

## License

MIT
