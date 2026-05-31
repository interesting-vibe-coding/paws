#!/bin/bash
# Paws signal hook for Kiro: report the agent's state to Kaku via an OSC 1337
# user var. Kaku's Lua user-var-changed handler does the actual tab switch.
# No pane/tab control here — just a signal written to the controlling terminal.
#
# Usage: paws-signal.sh busy|done   (default: done)
#
# Wire it up in ~/.kiro/agents/<your-agent>.json:
#   "hooks": {
#     "userPromptSubmit": [{ "command": "/abs/path/paws-signal.sh busy" }],
#     "stop":             [{ "command": "/abs/path/paws-signal.sh done" }]
#   }
state="${1:-done}"
printf '\033]1337;SetUserVar=paws_agent_%s=%s\007' "$state" "$(printf 1 | base64)" > /dev/tty 2>/dev/null
exit 0
