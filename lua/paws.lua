-- Paws 🐾 — native Kaku/WezTerm integration.
-- Add this to ~/.config/kaku/kaku.lua (before `return config`).
-- Everything runs in-process: no external scripts, no temp files, no `kaku cli`.
--
-- The game lives in its OWN TAB, so your existing pane/split layout is never
-- disturbed. A tab is naturally full-window & immersive.
--   CMD+G       : spawn / toggle between the agent tab and the game tab
--   CMD+SHIFT+G : toggle auto-navigation mode on/off
-- The agent emits OSC user-vars as it works:
--   paws_agent_busy  (it started)  → in auto mode, jump to the game tab
--   paws_agent_done  (it finished) → jump back to the agent tab

local wezterm = require 'wezterm'

local PAWS_GAME = 'paws'  -- the paws launcher (rotates among installed games)
local PAWS_SHELL = os.getenv('SHELL') or '/bin/sh'  -- login shell, so PATH resolves

-- wezterm.mux.get_tab raises if the tab is gone; make it return nil instead
local function paws_tab(tab_id)
  if not tab_id then return nil end
  local ok, t = pcall(wezterm.mux.get_tab, tab_id)
  return ok and t or nil
end

-- Ensure a game tab exists (remembering the agent tab); return its MuxTab
local function paws_ensure_game(window, pane)
  local gt = paws_tab(wezterm.GLOBAL.paws_game_tab)
  if gt then return gt end
  wezterm.GLOBAL.paws_agent_tab = pane:tab():tab_id()
  local tab = window:mux_window():spawn_tab { args = { PAWS_SHELL, '-l', '-c', PAWS_GAME } }
  wezterm.GLOBAL.paws_game_tab = tab:tab_id()
  return tab
end

config.keys = config.keys or {}
table.insert(config.keys, {
  key = 'g',
  mods = 'SUPER',
  action = wezterm.action_callback(function(window, pane)
    local game = paws_tab(wezterm.GLOBAL.paws_game_tab)
    if game and pane:tab():tab_id() == wezterm.GLOBAL.paws_game_tab then
      local at = paws_tab(wezterm.GLOBAL.paws_agent_tab)
      if at then at:activate() end
    else
      paws_ensure_game(window, pane):activate()
    end
  end),
})
table.insert(config.keys, {
  key = 'g',
  mods = 'SUPER|SHIFT',
  action = wezterm.action_callback(function(window, pane)
    wezterm.GLOBAL.paws_auto = not wezterm.GLOBAL.paws_auto
    window:toast_notification('Paws 🐾',
      wezterm.GLOBAL.paws_auto and 'Auto-navigation ON' or 'Auto-navigation OFF', nil, 2000)
  end),
})

wezterm.on('user-var-changed', function(window, pane, name, value)
  if name == 'paws_agent_done' then
    local at = paws_tab(wezterm.GLOBAL.paws_agent_tab)
    if at then at:activate() end
  elseif name == 'paws_agent_busy' and wezterm.GLOBAL.paws_auto then
    paws_ensure_game(window, pane):activate()
  end
end)
