# Uses the channel form of the installation
# https://github.com/nix-community/NUR/issues/150
{
  # For Intel stuff
  allowUnfree = true;
  # NUR setup
  # https://github.com/nix-community/NUR
  pkgs = import <unstable>; # nixos-unstable channel
  packageOverrides = pkgs: {
    nur = import <nur> { # Also defined as a channel
      inherit pkgs;
    };
  };
}
