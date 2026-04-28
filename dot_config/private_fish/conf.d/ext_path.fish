# Generic stuff
fish_add_path "$HOME/.local/bin"
if command -q pixi
   fish_add_path "$HOME/.pixi/bin"
end
