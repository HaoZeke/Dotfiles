{ config, pkgs, ... }:

{
  # List packages installed in system profile. To search by name, run:
  # $ nix-env -qaP | grep wget
  environment.systemPackages = with pkgs; [
    neovim
    # GUI
    emacs
    alacritty
    # Base Tools
    gnupg
    ripgrep
    fd
    fzf
    exa
    zsh
    tmux
    htop
    cmake
    neofetch
    wakatime
    # Version control
    git
    subversion
    # Managers
    rustup
    # Languages
    # R # broken for now
    # Nix
    nixfmt
    nox
    lorri
    # xquartz # A mac requirement for x11 and cairo
  ];

  # Use a custom configuration.nix location.
  # $ darwin-rebuild switch -I darwin-config=$HOME/.config/nixpkgs/darwin/configuration.nix
  # environment.darwinConfig = "$HOME/.config/nixpkgs/darwin/configuration.nix";

  # Auto upgrade nix package and the daemon service.
  services.nix-daemon.enable = true;
  nix.package = pkgs.nix;

  # Create /etc/bashrc that loads the nix-darwin environment.
  programs.zsh.enable = true; # default shell on catalina
  # programs.fish.enable = true;

  # Used for backwards compatibility, please read the changelog before changing.
  # $ darwin-rebuild changelog
  system.stateVersion = 4;
  # Manage users better
  users.nix.configureBuildUsers = true;

  # Needed for X11
  nixpkgs.config.allowUnfree = true;
}
