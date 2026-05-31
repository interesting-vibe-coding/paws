#!/usr/bin/env fish
# paws - Terminal companion for AI coding agents
# Usage: paws start | paws stop | paws status

set STATE_FILE /tmp/paws-state.json
set GAME_CMD 2048  # default game

function paws_start
    if test -f $STATE_FILE
        echo "🐾 Paws is already running. Use 'paws stop' to end."
        return 1
    end

    # Get current pane id (this is the agent pane)
    set agent_pane_id (kaku cli list --format json 2>/dev/null | python3 -c "
import json, sys
data = json.load(sys.stdin)
for w in data:
    for t in w.get('tabs', []):
        if t.get('is_active'):
            for p in t.get('panes', []):
                if p.get('is_active'):
                    print(p['pane_id'])
                    sys.exit()
" 2>/dev/null)

    if test -z "$agent_pane_id"
        echo "❌ Could not detect Kaku pane. Are you running inside Kaku?"
        return 1
    end

    # Split right, 25% width, run the game
    set game_pane_id (kaku cli split-pane --right --percent 25 -- $GAME_CMD 2>/dev/null | grep -o '[0-9]*')

    if test -z "$game_pane_id"
        echo "❌ Failed to split pane."
        return 1
    end

    # Write state
    echo "{\"game_pane_id\":$game_pane_id,\"agent_pane_id\":$agent_pane_id}" > $STATE_FILE

    # Switch focus back to agent pane
    kaku cli activate-pane --pane-id $agent_pane_id >/dev/null 2>&1

    echo "🐾 Paws started! Game running in side pane."
    echo "   Press CMD+G to switch to game. It auto-pauses when agent needs input."
end

function paws_stop
    if not test -f $STATE_FILE
        echo "🐾 Paws is not running."
        return 0
    end

    set game_pane_id (cat $STATE_FILE | python3 -c "import json,sys; print(json.load(sys.stdin)['game_pane_id'])" 2>/dev/null)

    if test -n "$game_pane_id"
        kaku cli kill-pane --pane-id $game_pane_id >/dev/null 2>&1
    end

    rm -f $STATE_FILE
    echo "🐾 Paws stopped."
end

function paws_status
    if test -f $STATE_FILE
        echo "🐾 Paws is running."
        cat $STATE_FILE | python3 -c "
import json, sys
d = json.load(sys.stdin)
print(f'   Game pane: {d[\"game_pane_id\"]}')
print(f'   Agent pane: {d[\"agent_pane_id\"]}')
"
    else
        echo "🐾 Paws is not running."
    end
end

# Main
switch (count $argv) > 0; and echo $argv[1]; or echo "help"
    case start
        paws_start
    case stop
        paws_stop
    case status
        paws_status
    case '*'
        echo "🐾 Paws - Terminal companion for AI coding agents"
        echo ""
        echo "Usage:"
        echo "  paws start   Start a game in a side pane"
        echo "  paws stop    Kill the game pane"
        echo "  paws status  Check if paws is running"
end
