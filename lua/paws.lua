-- Paws 🐾 — native Kaku/WezTerm integration.
-- Add this to ~/.config/kaku/kaku.lua (before `return config`).
-- Everything runs in-process: no external scripts, no temp files, no `kaku cli`.
--
-- The game lives in its OWN TAB, so your existing pane/split layout is never
-- disturbed. A tab is naturally full-window & immersive.
-- CMD+G : first press spawns the game tab; then toggles agent tab <-> game tab.
-- The agent emits OSC user-var "paws_agent_done" when it finishes a turn
--         → auto-switch back to the agent tab.

local wezterm = require 'wezterm'

local PAWS_GAME = '2048'  -- game command (later: the paws Rust wrapper)
local PAWS_SHELL = os.getenv('SHELL') or '/bin/sh'  -- login shell, so PATH resolves

-- wezterm.mux.get_tab raises if the tab is gone; make it return nil instead
local function paws_tab(tab_id)
  if not tab_id then return nil end
  local ok, t = pcall(wezterm.mux.get_tab, tab_id)
  return ok and t or nil
end

-- Bind CMD+G (assumes `config.keys` already exists)
table.insert(config.keys, {
  key = 'g',
  mods = 'SUPER',
  action = wezterm.action_callback(function(window, pane)
    if not paws_tab(wezterm.GLOBAL.paws_game_tab) then
      wezterm.GLOBAL.paws_agent_tab = pane:tab():tab_id()
      local tab = window:mux_window():spawn_tab { args = { PAWS_SHELL, '-l', '-c', PAWS_GAME } }
      wezterm.GLOBAL.paws_game_tab = tab:tab_id()
      tab:activate()
      return
    end
    if pane:tab():tab_id() == wezterm.GLOBAL.paws_game_tab then
      local at = paws_tab(wezterm.GLOBAL.paws_agent_tab)
      if at then at:activate() end
    else
      paws_tab(wezterm.GLOBAL.paws_game_tab):activate()
    end
  end),
})

wezterm.on('user-var-changed', function(window, pane, name, value)
  if name == 'paws_agent_done' then
    local at = paws_tab(wezterm.GLOBAL.paws_agent_tab)
    if at then at:activate() end
  end
end)
