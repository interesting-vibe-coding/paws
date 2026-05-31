#!/bin/bash
# Paws stop hook for Kiro: tell Kaku the agent finished, via an OSC 1337 user var.
# Kaku's Lua user-var-changed handler does the actual pane switch.
# No pane control here — just a signal written to the controlling terminal.
#
# Wire it up in ~/.kiro/agents/<your-agent>.json:
#   "hooks": { "stop": [{ "command": "/absolute/path/to/paws-signal.sh" }] }
printf '\033]1337;SetUserVar=paws_agent_done=%s\007' "$(printf 1 | base64)" > /dev/tty 2>/dev/null
exit 0
