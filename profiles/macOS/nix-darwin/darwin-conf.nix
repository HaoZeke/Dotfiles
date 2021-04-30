{ config, pkgs, ... }:

# getEnv concept from here https://github.com/peel/dotfiles/blob/c2c24d50aa3d36185a4f557ed1db7a8f5dd1f02b/setup/darwin/config.nix
let home = builtins.getEnv "HOME";
in {
  # List packages installed in system profile. To search by name, run:
  # $ nix-env -qaP | grep wget
  environment.systemPackages = with pkgs; [
    neovim
    # GUI
    emacs
    alacritty
    zathura
    # Base Tools
    gnupg
    ripgrep
    fd
    fzf
    exa
    bat
    tree
    hub
    jq
    zsh
    tmux
    htop
    cmake
    neofetch
    wakatime
    wget
    clang-tools # clang-format and a whole lot more
    gibo
    hugo
    # Version control
    git
    subversion
    # Managers
    rustup
    # rbenv # better off with brew
    # Languages
    # R # broken for now
    # texlive.combined.scheme-full # pointless, use homebrew or texlive installer
    pandoc
    # Nix
    # nixfmt
    nox
    lorri
    niv
    mosh
    # bundix
    # nodePackages.node2nix
    # MacOS
    yabai
    skhd
    spacebar
    pinentry_mac gnupg
    # pngpaste # An image clipboard helper, use homebrew for now
    # xquartz # A mac requirement for x11 and cairo
  ];
  # Use a custom configuration.nix location.
  # $ darwin-rebuild switch -I darwin-config=$HOME/.config/nixpkgs/darwin/configuration.nix
  # environment.darwinConfig = "$HOME/.config/nixpkgs/darwin/configuration.nix";

  # Auto upgrade nix package and the daemon service.
  services.nix-daemon.enable = true;
  nix.package = pkgs.nixUnstable;
  nix.extraOptions = ''
    experimental-features = nix-command flakes
  '';

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

  ############
  # Services #
  ############
  # gpg #
  programs.gnupg.agent = {
        enable = true;
        enableSSHSupport = true;
      };

  # Strongly kanged from https://github.com/xanderle/config/blob/main/darwin-configuration.nix
  # SKHD #
  # services.skhd.skhdConfig =
  #   builtins.readFile ./service_confs/skhdrc; # Managed by bombadili
  services.skhd = {
    enable = true;
    package = pkgs.skhd;
    skhdConfig =
      builtins.readFile "${home}/.config/skhd/skhdrc"; # Managed by bombadili
  };
  # yabai #
  services.yabai = {
    enable = true;
    package = pkgs.yabai;
    enableScriptingAddition = true;
    config = {
           window_border              = "on";
           window_border_width        = 5;
      active_window_border_color = "0xff81a1c1";
      normal_window_border_color = "0xff3b4252";
      focus_follows_mouse = "autofocus";
      mouse_follows_focus = "off";
      window_placement = "second_child";
      window_opacity = "off";
      window_topmost = "on"; # Floating windows on top
      window_shadow = "float";
      active_window_opacity = "1.0";
      normal_window_opacity = "1.0";
      split_ratio = "0.50";
      auto_balance = "on";
      mouse_modifier = "fn";
      mouse_action1 = "move";
      mouse_action2 = "resize";
      layout = "bsp";
      top_padding = 10;
      bottom_padding = 10;
      left_padding = 10;
      right_padding = 10;
      window_gap = 10;
    };
    extraConfig =
      builtins.readFile "${home}/.config/yabai/yabairc"; # Managed by bombadili, has rules and signals
  };

  # Spacebar #
  services.spacebar = {
    enable = true;
    package = pkgs.spacebar;
    config = {
      debug_output = "on";
      position = "top";
      clock_format = "%R";
      space_icon_strip = "   ";
      text_font = ''"CaskaydiaCove Nerd Font:Regular:12.0"'';
      icon_font = ''"CaskaydiaCove Nerd Font:Regular:12.0"'';
      background_color = "0xff2e3440";
      foreground_color = "0xffd8dee9";
      space_icon_color = "0xff81a1c1";
      dnd_icon_color = "0xff81a1c1";
      clock_icon_color = "0xff81a1c1";
      power_icon_color = "0xff81a1c1";
      battery_icon_color = "0xff81a1c1";
      power_icon_strip = " ";
      space_icon = "";
      clock_icon = "";
      dnd_icon = "";
    };
  };
}
