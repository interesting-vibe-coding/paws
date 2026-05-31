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

Check for a terminal game on PATH (`which 2048`). If none, install one:

```bash
brew install c2048   # provides the `2048` binary
```

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

## 3. Wire the agent's "I'm done" signal

The stop hook runs `hooks/kiro/paws-signal.sh`, which emits one OSC user var to
the tty. Kaku's Lua handler switches back to the agent tab. **Use the absolute
path** to the script — `~` is not expanded in hook commands.

### Kiro CLI
`kiro_default` is built-in and cannot be edited, so use a custom agent that is
identical to default except for the hook:

1. Create/merge `~/.kiro/agents/default.json` with:
   ```json
   {
     "name": "default",
     "tools": ["*"],
     "allowedTools": ["@builtin", "@*"],
     "useLegacyMcpJson": true,
     "hooks": { "stop": [{ "command": "<REPO>/hooks/kiro/paws-signal.sh" }] }
   }
   ```
   If the file exists, just add the `stop` hook entry (don't clobber other keys
   or other stop hooks).
2. Tell the user to launch with `kiro-cli chat --agent default` (or update their
   shell alias) so the hook is active.

### Claude Code (secondary / optional)
Add a `Stop` hook in the user's Claude settings that runs the same
`paws-signal.sh`. (Claude support is still being validated — flag it as such.)

## 4. Make the signal script executable

```bash
chmod +x <REPO>/hooks/kiro/paws-signal.sh
```

## 5. Finish

Tell the user to **reload Kaku (CMD+Shift+R)** and press **CMD+G**: the first
press opens the game in its own tab; press again to toggle back. When the agent
finishes a turn, Paws switches back to the agent tab automatically.

## Verify

- `luac -p ~/.config/kaku/kaku.lua` passes.
- The hook path in the agent config is absolute and the file is executable.
- A game binary is on PATH.

Report exactly which files you created or modified.
