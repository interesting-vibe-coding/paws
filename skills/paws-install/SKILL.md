---
name: paws-install
description: Install Paws 🐾 (terminal companion for AI coding agents) into the user's Kaku or WezTerm terminal and their agent (Kiro CLI, Claude Code, or Codex CLI). Use this when the user asks to install, set up, or wire up Paws. Performs the Lua merge, game install, and agent hook wiring — all idempotently.
---

# Installing Paws 🐾

You are installing Paws for the user. Work from a local clone of this repo
(`paws/`). Do each step, verify, and report what you changed. All edits must be
**idempotent** — re-running must not duplicate anything.

## 0. Preconditions

- Confirm the terminal is **Kaku** (`which kaku`), **WezTerm** (`which wezterm`),
  or **iTerm2** (`ls /Applications/iTerm.app`). All three are supported.
  If none is installed, ask the user to install one first and stop.
- Determine the terminal type — it affects Step 2:
  - Kaku: `~/.config/kaku/kaku.lua` (Lua config)
  - WezTerm: `~/.config/wezterm/wezterm.lua` (Lua config)
  - iTerm2: Python script + manual key bindings (see Step 2b)
- Note the repo root (absolute path) — you'll need it for hook paths.
  **All hook paths in config files must be absolute** — `~` is not expanded
  by any of the three agents.

## 1. Build paws and install games

The game tab runs `paws`, a tiny Rust launcher that shows a centered menu of
games (installed and uninstalled, with one-click install from the picker). Build
it and install at least one game to start:

```bash
cargo install --path .                                          # builds `paws` onto PATH
cargo install --git https://github.com/interesting-vibe-coding/paws-games --bin jump-high    # Dog Jump
cargo install --git https://github.com/interesting-vibe-coding/paws-games --bin earth-online # Earth Online
cargo install --git https://github.com/interesting-vibe-coding/paws-games --bin tetris       # Tetris
paws --list                                                     # confirm which games are detected
```

**Alternatively**, install via Homebrew (builds from HEAD):

```bash
brew install --HEAD interesting-vibe-coding/paws/paws
brew install --HEAD interesting-vibe-coding/paws/paws-games
```

If `cargo` is missing, point the user to https://rustup.rs first.
The game install commands are independent — run them in parallel to save time.
Any games not installed now can be installed later directly from the in-app
game picker (uninstalled entries show "⤓ install" and run the install command
on Enter).

## 2a. Merge the Lua into the terminal config (Kaku / WezTerm)

The terminal config returns a `config` table at the end. The snippet to insert is
`lua/paws.lua` from this repo — it works identically in Kaku and WezTerm.

- **Kaku:** config is at `~/.config/kaku/kaku.lua`
- **WezTerm:** config is at `~/.config/wezterm/wezterm.lua`

Steps:
- If the config already contains `Paws 🐾` (the marker comment), skip — already installed.
- Otherwise insert the **body** of `lua/paws.lua` (everything except its
  `local wezterm = require 'wezterm'` line, which the config already has)
  **immediately before** the final `return config` line.
- Ensure `config.keys` exists before the insert: if the config never sets it,
  add `config.keys = config.keys or {}` at the top of the inserted block.
- Syntax-check afterward: `luac -p <config-path>` (if `luac` exists).

## 2b. Install the Python script (iTerm2 only)

Skip this step if the user is on Kaku or WezTerm.

```bash
mkdir -p ~/.config/iterm2/scripts/AutoLaunch
cp <REPO>/iterm2/paws.py ~/.config/iterm2/scripts/AutoLaunch/paws.py
```

Then tell the user to:
1. In iTerm2: **Scripts → AutoLaunch → paws.py** to load the script.
2. Open **Settings → Keys → Key Bindings** and add three bindings:
   - `Cmd+G` → Invoke Script Function → `paws_toggle()`
   - `Cmd+Shift+P` → Invoke Script Function → `paws_picker()`
   - `Cmd+H` → Invoke Script Function → `paws_help()`

Full details: [docs/iterm2-setup.md](../../docs/iterm2-setup.md)

## 3. Wire the agent's state signals (for the status HUD)

Paws ships two hook scripts that write session state to `/tmp/paws-sessions/<id>`
so the game's HUD can show which agents are running vs done:

| Script | Used by | Input |
|---|---|---|
| `hooks/kiro/paws-signal.sh` | Kiro CLI | CLI args: `busy` or `done`; env `KIRO_SESSION_ID` |
| `hooks/paws-hook.sh` | Claude Code, Codex CLI | JSON on stdin with `session_id` and `hook_event_name` |

Wire **one** of the three subsections below, matching the user's agent. If the
user uses multiple agents, wire all that apply — they share the same
`/tmp/paws-sessions/` directory and the HUD aggregates them.

### 3a. Kiro CLI

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
       "stop": [
         { "command": "<REPO>/hooks/kiro/paws-signal.sh done" },
         { "command": "afplay /System/Library/Sounds/Glass.aiff" }
       ]
     }
   }
   ```
   Replace `<REPO>` with the **absolute path** to this repo (e.g.
   `/Users/me/paws`). The second `stop` hook is an optional completion chime
   (macOS). If the file exists, merge the hook entries without clobbering
   other keys or existing hooks.
2. Tell the user to launch with `kiro-cli chat --agent default` (or update
   their shell alias) so the hooks are active.

### 3b. Claude Code

Claude Code reads hooks from `~/.claude/settings.json` (user-level) or
`.claude/settings.json` (project-level). Add two hook events: `UserPromptSubmit`
(fires once per turn when the user sends a prompt → marks session as "busy") and
`Stop` (fires when the agent finishes → marks session as "done").

1. Open `~/.claude/settings.json`. If it already has a `"hooks"` key, **merge**
   the two new events into it — do not overwrite existing hooks (e.g.
   `Notification`). If there is no `"hooks"` key, create it.
2. Add (or merge) these entries inside `"hooks"`:
   ```json
   "UserPromptSubmit": [
     {
       "hooks": [
         {
           "type": "command",
           "command": "<REPO>/hooks/paws-hook.sh",
           "timeout": 5
         }
       ]
     }
   ],
   "Stop": [
     {
       "hooks": [
         {
           "type": "command",
           "command": "<REPO>/hooks/paws-hook.sh",
           "timeout": 5
         }
       ]
     }
   ]
   ```
   Replace `<REPO>` with the **absolute path** to this repo.
3. **Hooks load at session start, not mid-session.** Since you (the installing
   agent) are modifying settings.json during the current session, the hooks
   won't fire until the user's **next** Claude Code session. To give the HUD
   something to show right now, bootstrap the current session:
   ```bash
   paws signal busy
   ```

**Why `UserPromptSubmit` instead of `PreToolUse`?** `PreToolUse` fires on every
tool call (dozens per turn), creating redundant writes. `UserPromptSubmit` fires
once per turn — cleaner and sufficient for the HUD's "busy" signal.

### 3c. Codex CLI

> **Status: experimental.** Codex CLI's hook system is still evolving. The
> wiring below works with codex-cli >= 0.130. If it doesn't work, the rest of
> Paws still functions — you just won't see Codex sessions in the HUD.

Codex CLI reads hooks from `~/.codex/config.toml` (TOML array-of-tables format).
The hook receives JSON on stdin with `session_id` and `hook_event_name`, same
as Claude Code.

1. Open `~/.codex/config.toml` and **append** the following (don't overwrite
   existing project trust settings etc.):
   ```toml
   [[hooks.UserPromptSubmit]]

   [[hooks.UserPromptSubmit.hooks]]
   type = "command"
   command = "<REPO>/hooks/paws-hook.sh"
   timeout = 5

   [[hooks.Stop]]

   [[hooks.Stop.hooks]]
   type = "command"
   command = "<REPO>/hooks/paws-hook.sh"
   timeout = 5
   ```
   Replace `<REPO>` with the **absolute path** to this repo.
2. Restart the Codex session for hooks to take effect.
3. On first run, Codex will prompt to trust the hook script — accept it.
4. To bootstrap the current session before hooks are active:
   ```bash
   paws signal busy
   ```

## 4. Make the hook scripts executable

```bash
chmod +x <REPO>/hooks/paws-hook.sh
chmod +x <REPO>/hooks/kiro/paws-signal.sh
chmod +x <REPO>/hooks/kiro/paws-pause.sh
```

## 5. Finish

- **Kaku users:** Press **CMD+Shift+R** to reload — Kaku does NOT auto-reload on save.
- **WezTerm users:** Config reloads automatically on save — no action needed.

Then:
- **CMD+G** — opens the game tab (a centered menu: games · 🎲 Random · ⚙ Settings); after that it toggles agent ↔ game.
- **CMD+SHIFT+P** — close the game tab and re-open the menu.
- **CMD+H** — open the Paws repo in your browser (to file an issue / say hi).

(Kaku users: don't use CMD+SHIFT+G — Kaku already binds it to lazygit.)

## Verify

- `luac -p <config-path>` passes (if luac is available).
- `paws --list` shows at least one installed game.
- The hook paths in the agent config are absolute and the scripts are executable.
- Quick smoke test (Claude Code / Codex): pipe mock JSON to the hook and check
  the state file:
  ```bash
  echo '{"session_id":"test","hook_event_name":"UserPromptSubmit"}' | <REPO>/hooks/paws-hook.sh
  cat /tmp/paws-sessions/test   # should show: busy <pid>
  ```

Report exactly which files you created or modified.
