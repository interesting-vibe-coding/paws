-- Paws: Kaku Lua config snippet
-- Add this to ~/.config/kaku/kaku.lua (or wezterm.lua)
-- Binds CMD+G to switch to the game pane

local wezterm = require 'wezterm'

wezterm.on('paws-switch-to-game', function(window, pane)
  local state_file = io.open('/tmp/paws-state.json', 'r')
  if not state_file then return end
  local content = state_file:read('*a')
  state_file:close()
  local game_pane_id = content:match('"game_pane_id":(%d+)')
  if game_pane_id then
    os.execute('kaku cli activate-pane --pane-id ' .. game_pane_id)
  end
end)

return {
  keys = {
    { key = 'g', mods = 'SUPER', action = wezterm.action.EmitEvent('paws-switch-to-game') },
  },
}
