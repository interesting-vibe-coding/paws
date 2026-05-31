#!/bin/bash
# Paws stop hook: when agent finishes a turn, pause the game and switch focus back.
# Only acts if a paws session is active (state file exists).

STATE_FILE="/tmp/paws-state.json"

if [ ! -f "$STATE_FILE" ]; then
  exit 0
fi

GAME_PANE_ID=$(cat "$STATE_FILE" | grep -o '"game_pane_id":[0-9]*' | grep -o '[0-9]*')
AGENT_PANE_ID=$(cat "$STATE_FILE" | grep -o '"agent_pane_id":[0-9]*' | grep -o '[0-9]*')

if [ -z "$GAME_PANE_ID" ] || [ -z "$AGENT_PANE_ID" ]; then
  exit 0
fi

# Pause the game (send 'p' key) and activate agent pane
kaku cli send-text --pane-id "$GAME_PANE_ID" --no-paste "p" >/dev/null 2>&1 &
kaku cli activate-pane --pane-id "$AGENT_PANE_ID" >/dev/null 2>&1 &

exit 0
