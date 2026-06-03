#!/usr/bin/env python3
"""
Paws 🐾 — iTerm2 integration.

Install
-------
  cp iterm2/paws.py ~/.config/iterm2/scripts/AutoLaunch/paws.py
  chmod +x ~/.config/iterm2/scripts/AutoLaunch/paws.py

Then bind three keys in iTerm2 → Settings → Keys → Key Bindings (+):
  Cmd+G       → Invoke Script Function → paws_toggle()
  Cmd+Shift+P → Invoke Script Function → paws_picker()
  Cmd+H       → Invoke Script Function → paws_help()

iTerm2 auto-starts this script at launch. State (current paws tab ID) is kept
in ~/.config/paws/iterm2-tab-id so it survives config reloads.
"""

import iterm2
import os
import subprocess

_STATE_FILE = os.path.expanduser("~/.config/paws/iterm2-tab-id")


def _read_tab_id() -> str | None:
    try:
        return open(_STATE_FILE).read().strip() or None
    except FileNotFoundError:
        return None


def _save_tab_id(tab_id: str) -> None:
    os.makedirs(os.path.dirname(_STATE_FILE), exist_ok=True)
    open(_STATE_FILE, "w").write(tab_id)


def _clear_tab_id() -> None:
    try:
        os.remove(_STATE_FILE)
    except FileNotFoundError:
        pass


async def _find_paws_tab(window, tab_id: str | None):
    if tab_id is None:
        return None
    for tab in window.tabs:
        if tab.tab_id == tab_id:
            return tab
    return None


async def _spawn_paws(window) -> None:
    shell = os.environ.get("SHELL", "/bin/zsh")
    tab = await window.async_create_tab()
    # Login shell so ~/.cargo/bin is on PATH
    await tab.current_session.async_send_text(f"{shell} -l -c paws\n")
    _save_tab_id(tab.tab_id)


async def main(connection):
    app = await iterm2.async_get_app(connection)

    @iterm2.RPC
    async def paws_toggle(window_id=iterm2.Reference("id")):
        """Cmd+G — open game picker, or toggle between agent and game."""
        window = app.get_window_by_id(window_id)
        if not window:
            return

        paws_tab = await _find_paws_tab(window, _read_tab_id())

        if paws_tab is None:
            await _spawn_paws(window)
        elif window.current_tab.tab_id == paws_tab.tab_id:
            # On paws tab → switch back to the previous tab
            tabs = window.tabs
            idx = next(i for i, t in enumerate(tabs) if t.tab_id == paws_tab.tab_id)
            prev_idx = idx - 1 if idx > 0 else (1 if len(tabs) > 1 else None)
            if prev_idx is not None:
                await tabs[prev_idx].async_select()
        else:
            await paws_tab.async_select()

    await paws_toggle.async_register(connection, paws_toggle)

    @iterm2.RPC
    async def paws_picker(window_id=iterm2.Reference("id")):
        """Cmd+Shift+P — close the game tab and reopen the picker."""
        window = app.get_window_by_id(window_id)
        if not window:
            return

        paws_tab = await _find_paws_tab(window, _read_tab_id())
        if paws_tab:
            await paws_tab.async_close(force=True)
        _clear_tab_id()
        await _spawn_paws(window)

    await paws_picker.async_register(connection, paws_picker)

    @iterm2.RPC
    async def paws_help(window_id=iterm2.Reference("id")):
        """Cmd+H — open the Paws repo in your browser."""
        subprocess.run(["open", "https://github.com/interesting-vibe-coding/paws"])

    await paws_help.async_register(connection, paws_help)


iterm2.run_forever(main)
