#!/usr/bin/env bash
# Paws 🐾 tmux toggle — bind to your prefix key:
#   bind-key g run-shell "~/.config/paws/tmux-toggle.sh"
#
# Press Prefix+g:
#   - No paws window exists → create one and switch to it
#   - Already on the paws window → jump back to previous window
#   - On another window → switch to the paws window

PAWS_WIN=$(tmux list-windows -F '#{window_index}:#{window_name}' \
           | awk -F: '$2=="paws"{print $1}')
CURRENT=$(tmux display-message -p '#{window_name}')

if [ -z "$PAWS_WIN" ]; then
    # Spawn a new login-shell window so ~/.cargo/bin is on PATH
    tmux new-window -n paws "${SHELL:-/bin/zsh} -l -c paws"
elif [ "$CURRENT" = "paws" ]; then
    tmux last-window
else
    tmux select-window -t "$PAWS_WIN"
fi
