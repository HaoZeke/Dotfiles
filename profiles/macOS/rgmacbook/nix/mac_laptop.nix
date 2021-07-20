# A config.nix
{
  # For Intel stuff
  allowUnfree = true;
  # NUR setup
  # https://github.com/nix-community/NUR
  packageOverrides = pkgs: {
    nur = import (builtins.fetchTarball {
      # Get the revision by choosing a version from https://github.com/nix-community/NUR/commits/master
      url = "https://github.com/nix-community/NUR/archive/3a6a6f4da737da41e27922ce2cfacf68a109ebce.tar.gz";
      # Get the hash by running `nix-prefetch-url --unpack <url>` on the above url
      sha256 = "04387gzgl8y555b3lkz9aiw9xsldfg4zmzp930m62qw8zbrvrshd";
    }
    ) {
      inherit pkgs;
    };
  };
}
