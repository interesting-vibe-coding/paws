#!/bin/bash
# Paws signal hook for Kiro: report the agent's state.
#   1. /tmp/paws-signal        — latest event (busy|done), drives pause/resume
#   2. /tmp/paws-sessions/<id> — per-session state, drives the status HUD
#   3. OSC 1337 user-var       — Kaku Lua does the tab switch
# Usage: paws-pause.sh busy|done   (default: done)
state="${1:-done}"
echo "$state" > /tmp/paws-signal
mkdir -p /tmp/paws-sessions
echo "$state" > "/tmp/paws-sessions/${KIRO_SESSION_ID:-default}"
printf '\033]1337;SetUserVar=paws_agent_%s=%s\007' "$state" "$(printf 1 | base64)" > /dev/tty 2>/dev/null
exit 0
