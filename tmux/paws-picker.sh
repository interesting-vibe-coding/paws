#!/usr/bin/env bash
# Paws 🐾 tmux picker — close the game window and reopen the picker.
# Bind to your prefix key:
#   bind-key P run-shell "~/.config/paws/tmux-picker.sh"

PAWS_WIN=$(tmux list-windows -F '#{window_index}:#{window_name}' \
           | awk -F: '$2=="paws"{print $1}')

if [ -n "$PAWS_WIN" ]; then
    tmux kill-window -t "$PAWS_WIN"
fi

tmux new-window -n paws "${SHELL:-/bin/zsh} -l -c paws"
