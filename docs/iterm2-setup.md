# Paws × iTerm2 setup

Paws works in iTerm2 via a small Python AutoLaunch script that registers three
keyboard shortcuts. The `paws` binary itself runs identically in any terminal —
this script handles the tab-create/toggle behavior that makes the workflow smooth.

## What you get

| Key | Action |
|-----|--------|
| **Cmd+G** | First press: open the game picker. After that: toggle agent ↔ game. |
| **Cmd+Shift+P** | Close the game tab and re-open the picker. |
| **Cmd+H** | Open the Paws repo in your browser. |

## Install

### 1. Copy the script

```bash
mkdir -p ~/.config/iterm2/scripts/AutoLaunch
cp iterm2/paws.py ~/.config/iterm2/scripts/AutoLaunch/paws.py
```

iTerm2 automatically runs scripts in `AutoLaunch/` at startup. The script
registers the three RPC functions above and keeps running in the background.

### 2. Reload scripts

In iTerm2: **Scripts → AutoLaunch → paws.py** (or restart iTerm2).

You only need to do this once — after that it auto-starts on every launch.

### 3. Bind the three keys

Open **iTerm2 → Settings → Keys → Key Bindings**, then click **+** three times:

| Keyboard Shortcut | Action | Parameter |
|-------------------|--------|-----------|
| `Cmd+G` | Invoke Script Function | `paws_toggle()` |
| `Cmd+Shift+P` | Invoke Script Function | `paws_picker()` |
| `Cmd+H` | Invoke Script Function | `paws_help()` |

For each binding: click **+** → set the key combo → choose **"Invoke Script Function"** from the Action drop-down → type the function name exactly as shown above.

> **Note:** "Invoke Script Function" only appears in the Action list once the
> script is loaded. Reload first (step 2) if you don't see it.

### 4. Done

Press **Cmd+G** — the game picker appears in a new tab. The HUD on the top row
shows your agent sessions. Press **Cmd+G** again to toggle back.

## How it works

The script (`iterm2/paws.py`) uses the [iTerm2 Python API](https://iterm2.com/python-api/):

- On `paws_toggle()`: looks for an existing paws tab (by ID, persisted to
  `~/.config/paws/iterm2-tab-id`). If none exists, spawns a new tab running
  `paws` via a login shell (so `~/.cargo/bin` is on PATH). If you're already on
  the paws tab, switches back to the previous tab.
- On `paws_picker()`: closes the existing paws tab and opens a fresh one.
- On `paws_help()`: calls `open` to launch the repo URL.

## Troubleshooting

**"Invoke Script Function" not in the Action list**  
The script isn't loaded yet. Go to **Scripts → AutoLaunch → paws.py** to
start it, then try the key binding dialog again.

**`paws: command not found`**  
Run `cargo install --path .` from the paws repo, then open a new terminal
tab and try again.

**State mismatch after closing the game tab manually**  
Delete `~/.config/paws/iterm2-tab-id` — Paws will treat the next Cmd+G as
a fresh start.

## Difference vs WezTerm / Kaku

iTerm2 setup requires one extra step: wiring keybindings in the GUI (3 clicks
per binding). WezTerm/Kaku uses a Lua config file where Paws ships the bindings
as code. The runtime behavior is identical once set up.
