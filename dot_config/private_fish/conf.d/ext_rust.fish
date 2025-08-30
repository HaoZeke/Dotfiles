if command -q atuin
    atuin init fish --disable-up-arrow | source
end

if command -q starship
    starship init fish | source
end

if command -q zoxide
    zoxide init fish | source
end
