---
name: paws-install
description: Install Paws 🐾 (terminal companion for AI coding agents) into the user's Kaku terminal and their agent (Kiro CLI or Claude Code). Use this when the user asks to install, set up, or wire up Paws. Performs the Kaku Lua merge, game install, and agent stop-hook wiring — all idempotently.
---

# Installing Paws 🐾

You are installing Paws for the user. Work from a local clone of this repo
(`paws/`). Do each step, verify, and report what you changed. All edits must be
**idempotent** — re-running must not duplicate anything.

## 0. Preconditions

- Confirm the terminal is **Kaku** (`which kaku`). If not, tell the user Paws
  currently requires Kaku and stop.
- Note the repo root (absolute path) — you'll need absolute paths later.

## 1. Install a game

The game tab runs `paws`, a tiny Rust launcher that rotates among installed
games. Build it and install a few games:

```bash
cargo install --path .                      # builds `paws` onto PATH
brew install c2048 nudoku vitetris          # 2048 / sudoku / tetris
paws --list                                 # confirm which games are detected
```

If `cargo` is missing, point the user to https://rustup.rs first.

## 2. Merge the Lua into the Kaku config

The Kaku config is `~/.config/kaku/kaku.lua` (it returns a `config` table at the
end). The snippet to insert is `lua/paws.lua` from this repo.

- If `kaku.lua` already contains `Paws 🐾` (the marker comment), skip — already installed.
- Otherwise insert the **body** of `lua/paws.lua` (everything except its
  `local wezterm = require 'wezterm'` line, which the config already has)
  **immediately before** the final `return config` line.
- Ensure `config.keys` exists before the insert: if the config never sets it,
  add `config.keys = config.keys or {}` at the top of the inserted block.
- Syntax-check afterward: `luac -p ~/.config/kaku/kaku.lua` (if `luac` exists).

## 3. Wire the agent's state signals

`hooks/kiro/paws-signal.sh busy|done` emits one OSC user var to the tty; Kaku's
Lua handler does the tab switch. Wire two hooks: `userPromptSubmit` → `busy`
(agent started) and `stop` → `done` (agent finished). **Use absolute paths** —
`~` is not expanded in hook commands.

### Kiro CLI
`kiro_default` is built-in and cannot be edited, so use a custom agent identical
to default except for the hooks:

1. Create/merge `~/.kiro/agents/default.json` with:
   ```json
   {
     "name": "default",
     "tools": ["*"],
     "allowedTools": ["@builtin", "@*"],
     "useLegacyMcpJson": true,
     "hooks": {
       "userPromptSubmit": [{ "command": "<REPO>/hooks/kiro/paws-signal.sh busy" }],
       "stop": [{ "command": "<REPO>/hooks/kiro/paws-signal.sh done" }]
     }
   }
   ```
   If the file exists, just add the hook entries (don't clobber other keys or
   existing hooks).
2. Tell the user to launch with `kiro-cli chat --agent default` (or update their
   shell alias) so the hooks are active.

### Claude Code (secondary / optional)
Add `Stop` / notification hooks in the user's Claude settings that run the same
`paws-signal.sh`. (Claude support is still being validated — flag it as such.)

## 4. Make the signal script executable

```bash
chmod +x <REPO>/hooks/kiro/paws-signal.sh
```

## 5. Finish

Tell the user to **reload Kaku (CMD+Shift+R)**, then:
- **CMD+G** — open the game in its own tab (first press) / toggle agent ↔ game.
- **CMD+SHIFT+G** — toggle auto-navigation: in auto mode the agent sends them to
  the game when it starts working and back when it finishes; in manual mode it
  only auto-returns on finish.

## Verify

- `luac -p ~/.config/kaku/kaku.lua` passes.
- `paws --list` shows at least one installed game.
- The hook paths in the agent config are absolute and the script is executable.

Report exactly which files you created or modified.
