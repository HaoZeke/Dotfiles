# A config.nix
{
  # For Intel stuff
  allowUnfree = true;
  # NUR setup
  # https://github.com/nix-community/NUR
  packageOverrides = pkgs: {
    nur = import
      (builtins.fetchTarball {
        # Get the revision by choosing a version from https://github.com/nix-community/NUR/commits/master
        url = "https://github.com/nix-community/NUR/archive/4a81d63c672ffdbbb999eb2549c0eb0934b3384b.tar.gz";
        # Get the hash by running `nix-prefetch-url --unpack <url>` on the above url
        sha256 = "09bg0xifdk6rw35w472rk0dj69w08c1ksb3a3qrzay0ww69495zk";
      }
      )
      {
        inherit pkgs;
      };
  };
}
