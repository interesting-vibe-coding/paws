# Paws × tmux setup

Paws works in any terminal that runs tmux. The integration is two shell scripts
that handle the window-create/toggle behavior — no dependencies beyond tmux itself.

## What you get

| Key | Action |
|-----|--------|
| **Prefix+g** | First press: open the game picker in a new window. After that: toggle agent ↔ game. |
| **Prefix+G** | Close the game window and re-open the picker. |

> **Prefix** is `Ctrl+B` by default. If you've rebound it, use your custom prefix.

## Install

### 1. Copy the scripts

```bash
mkdir -p ~/.config/paws
cp tmux/paws-toggle.sh ~/.config/paws/tmux-toggle.sh
cp tmux/paws-picker.sh ~/.config/paws/tmux-picker.sh
cp tmux/paws.conf      ~/.config/paws/paws.conf
chmod +x ~/.config/paws/tmux-toggle.sh ~/.config/paws/tmux-picker.sh
```

### 2. Add keybindings to your tmux config

**Option A — source the provided config file:**

```bash
echo 'source-file ~/.config/paws/paws.conf' >> ~/.tmux.conf
```

**Option B — copy the two lines directly into `~/.tmux.conf`:**

```
bind-key g run-shell "$HOME/.config/paws/tmux-toggle.sh"
bind-key G run-shell "$HOME/.config/paws/tmux-picker.sh"
```

### 3. Reload tmux config

```bash
tmux source-file ~/.tmux.conf
```

Or press `Prefix+:` and type `source-file ~/.tmux.conf`.

### 4. Done

Press **Prefix+g** — the game picker opens in a new tmux window named `paws`.
The HUD on the top row shows your agent sessions. Press **Prefix+g** again to
toggle back to your previous window.

## How it works

`tmux-toggle.sh` queries the current window list with `tmux list-windows` to
find a window named `paws`. If one doesn't exist it spawns a new window running
`paws` via a login shell (so `~/.cargo/bin` is on PATH). If you're already on
the paws window it calls `tmux last-window` to jump back. Otherwise it switches
to the paws window.

No external state file — tmux's own window list is the source of truth.

## Troubleshooting

**`paws: command not found`**  
The scripts use a login shell (`$SHELL -l`) so `~/.cargo/bin` should be on
PATH. If it's still missing, run `cargo install --path .` from the paws repo
and try again.

**Prefix+g conflicts with an existing binding**  
Change `g` / `G` to any free key in `paws.conf` (or the lines in `~/.tmux.conf`).

**Game opens in a small pane instead of full window**  
Make sure you're not inside a split pane when pressing Prefix+g. The script
creates a new *window* (full screen), not a pane split.
