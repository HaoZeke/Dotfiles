{
  # For Intel stuff
  allowUnfree = true;
  # NUR setup
  # https://github.com/nix-community/NUR
  packageOverrides = pkgs: {
    nur = import (builtins.fetchTarball "https://github.com/nix-community/NUR/archive/master.tar.gz") {
      inherit pkgs;
    };
  };
}
