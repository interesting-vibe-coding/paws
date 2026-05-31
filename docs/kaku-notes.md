# Kaku integration notes

Hard-won facts about Kaku (a WezTerm fork) relevant to Paws. Read before touching
the Lua, to avoid re-discovering these.

## Keybindings
- **No auto-reload.** Kaku does not reload `kaku.lua` on save. Press **CMD+Shift+R**
  after every edit, or changes won't take effect.
- **SHIFT keys use the uppercase letter + `CMD|SHIFT`**, e.g. `key = 'P', mods = 'CMD|SHIFT'`.
  Lowercase `key = 'p'` with `SUPER|SHIFT` does NOT match (macOS sends the shifted char).
- **CMD+SHIFT+G is taken** by Kaku (launch lazygit). Other taken CMD|SHIFT letters:
  A, D, E, R, S, W, Y, plus `[ ] Enter`. Plain `CMD+G` is free.
- `wezterm.action_callback` works; prefer it + `wezterm.mux` over `os.execute('kaku cli …')`
  (shelling out to the mux from inside the mux can block/fail).
- `window:toast_notification(...)` — not confirmed to render in Kaku; don't rely on it
  for user feedback. Prefer visible actions (tab switch, `InputSelector`).

## Mux / panes / tabs (Lua API)
- `wezterm.mux.get_tab(id)` / `get_pane(id)` **raise** if the id is gone — wrap in `pcall`.
- Switch tab: `tab:activate()`. Spawn a tab: `window:mux_window():spawn_tab{ args = {...} }`.
- Spawned panes get a **minimal PATH** (`/usr/bin:/bin:...`). Launch games via a login
  shell so PATH resolves: `args = { os.getenv('SHELL'), '-l', '-c', cmd }`.
- State that must survive across the (manual) config reloads lives in `wezterm.GLOBAL`.
- Zooming a pane (`set_zoomed`) hides *all* sibling panes in the tab — bad when the user
  has other sessions split in the same tab. Use a separate **tab** for the game instead.

## CLI
- `kaku cli` exists (hidden/experimental) with WezTerm-style subcommands
  (`list`, `split-pane`, `spawn`, `activate-tab`, `zoom-pane`, `kill-pane`, …).
- `kaku cli list --format json` does **not** include `user_vars`, so you can't verify
  OSC user vars that way.

## Agent → terminal signal (UNVERIFIED)
- The plan: Kiro hooks emit OSC 1337 `SetUserVar` to the tty; Kaku fires the Lua
  `user-var-changed` event. **Not yet confirmed working on device** — the hook may lack a
  usable `/dev/tty`, or Kaku may handle the sequence differently. Next step: confirm via a
  `wezterm.log_info` probe in the handler + the Kaku GUI log
  (`~/.local/share/kaku/kaku-gui-log-*.txt`).
