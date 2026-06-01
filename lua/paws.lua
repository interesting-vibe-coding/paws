-- Paws 🐾 — native Kaku/WezTerm integration.
-- Add this to ~/.config/kaku/kaku.lua (before `return config`).
-- Everything runs in-process: no external scripts, no temp files, no `kaku cli`.
--
-- The game lives in its OWN TAB (full-window, never disturbs your panes).
--   CMD+G        : first time → pick a game; after that → toggle agent ↔ game
--   CMD+SHIFT+P  : re-open the picker to change the game
-- Manual switching only. The game shows a session-status HUD (running/done)
-- so you decide when to flip back — no auto-jumping.
--
-- NOTE: CMD+SHIFT+G is intentionally NOT used — Kaku already binds it (lazygit).
-- Kaku does not auto-reload config; press CMD+Shift+R after editing.

local wezterm = require 'wezterm'

local PAWS_SHELL = os.getenv('SHELL') or '/bin/sh'  -- login shell, so PATH resolves
local PAWS_CHOICES = {
  { label = '🎲 Random (rotates every few hours)', id = 'random' },
  { label = 'Tetris (俄罗斯方块)', id = 'tetris' },
  { label = 'Jump High (跳跳跳)', id = 'jump-high' },
  { label = '🌍 地球Online (real-life side quests)', id = '--earth-online' },
}

local function resolve_game(id)
  if id ~= 'random' then return id end
  local real_games = {}
  for _, c in ipairs(PAWS_CHOICES) do
    if c.id ~= 'random' then table.insert(real_games, c.id) end
  end
  local slot = math.floor(os.time() / (5 * 3600))
  return real_games[(slot % #real_games) + 1]
end

-- wezterm.mux.get_tab raises if the tab is gone; make it return nil instead
local function paws_tab(tab_id)
  if not tab_id then return nil end
  local ok, t = pcall(wezterm.mux.get_tab, tab_id)
  return ok and t or nil
end

-- spawn the game tab running `cmd`; remember the agent tab; activate the game
local function paws_spawn(window, agent_tab_id, cmd)
  if agent_tab_id then wezterm.GLOBAL.paws_agent_tab = agent_tab_id end
  local game = resolve_game(cmd)
  local tab = window:mux_window():spawn_tab { args = { PAWS_SHELL, '-l', '-c', 'paws ' .. game } }
  wezterm.GLOBAL.paws_game_tab = tab:tab_id()
  tab:activate()
end

-- show the native game picker; on choose, run on_pick(cmd)
local function paws_pick(window, pane, on_pick)
  window:perform_action(wezterm.action.InputSelector {
    title = 'Paws 🐾 — choose a game',
    choices = PAWS_CHOICES,
    action = wezterm.action_callback(function(_w, _p, id)
      if not id then return end
      wezterm.GLOBAL.paws_choice = id
      on_pick(id)
    end),
  }, pane)
end

config.keys = config.keys or {}
-- CMD+G: pick (first time) / toggle agent ↔ game
table.insert(config.keys, {
  key = 'g',
  mods = 'CMD',
  action = wezterm.action_callback(function(window, pane)
    local game = paws_tab(wezterm.GLOBAL.paws_game_tab)
    if game then
      if pane:tab():tab_id() == wezterm.GLOBAL.paws_game_tab then
        local at = paws_tab(wezterm.GLOBAL.paws_agent_tab)
        if at then at:activate() end
      else
        game:activate()
      end
      return
    end
    local agent_id = pane:tab():tab_id()
    if wezterm.GLOBAL.paws_choice then
      paws_spawn(window, agent_id, wezterm.GLOBAL.paws_choice)
    else
      paws_pick(window, pane, function(cmd) paws_spawn(window, agent_id, cmd) end)
    end
  end),
})
-- CMD+SHIFT+P: re-pick the game (close any open game tab, open the new one)
table.insert(config.keys, {
  key = 'P',
  mods = 'CMD|SHIFT',
  action = wezterm.action_callback(function(window, pane)
    local agent_id = wezterm.GLOBAL.paws_agent_tab or pane:tab():tab_id()
    paws_pick(window, pane, function(cmd)
      local old = paws_tab(wezterm.GLOBAL.paws_game_tab)
      if old then
        old:activate()
        window:perform_action(wezterm.action.CloseCurrentTab { confirm = false }, old:active_pane())
      end
      paws_spawn(window, agent_id, cmd)
    end)
  end),
})
