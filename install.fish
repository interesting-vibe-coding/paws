#!/usr/bin/env fish
# Paws installer
# Installs: launcher script, Kiro hook, prints Lua config instructions

set PAWS_DIR (cd (dirname (status filename)); and pwd)

echo "🐾 Installing Paws..."

# 1. Install launcher to ~/.local/bin (should be in PATH)
mkdir -p ~/.local/bin
cp "$PAWS_DIR/bin/paws.fish" ~/.local/bin/paws.fish
chmod +x ~/.local/bin/paws.fish

# Create a wrapper so `paws` works as a command
echo '#!/usr/bin/env fish
source ~/.local/bin/paws.fish' > ~/.local/bin/paws
chmod +x ~/.local/bin/paws

# 2. Install Kiro hook
mkdir -p ~/.kiro/hooks
cp "$PAWS_DIR/hooks/kiro/paws-pause.sh" ~/.kiro/hooks/paws-pause.sh
chmod +x ~/.kiro/hooks/paws-pause.sh

echo "✓ Launcher installed to ~/.local/bin/paws"
echo "✓ Kiro hook installed to ~/.kiro/hooks/paws-pause.sh"
echo ""
echo "⚠️  Manual steps:"
echo ""
echo "1. Add stop hook to your Kiro agent (~/.kiro/agents/default.json):"
echo '   "hooks": { "stop": [{ "command": "~/.kiro/hooks/paws-pause.sh" }] }'
echo ""
echo "2. Add Lua keybinding to ~/.config/kaku/kaku.lua:"
echo "   See lua/paws.lua in this repo for the snippet."
echo ""
echo "3. Make sure ~/.local/bin is in your PATH:"
echo "   fish_add_path ~/.local/bin"
echo ""
echo "4. Install a game: brew install 2048"
echo ""
echo "🐾 Done! Run 'paws start' to begin."
