# Kanged from https://github.com/adelbertc/dotfiles/blob/master/nixpkgs/.config/nixpkgs/overlays/emacs-native.nix
# Adapted from https://github.com/twlz0ne/nix-gccemacs-darwin/tree/aaacc6dd84dc3e585b4ad653dd3bbbe2cc7e070c
let
  sources = import ./nix/sources.nix;
  nixpkgs = sources.nixpkgs;
  pkgs = import nixpkgs {};
  emacs-nativecomp = sources."emacs-nativecomp";
  libPath = with pkgs; lib.concatStringsSep ":" [
    "${lib.getLib libgccjit}/lib/gcc/${stdenv.targetPlatform.config}/${libgccjit.version}"
    "${lib.getLib stdenv.cc.cc}/lib"
    "${lib.getLib stdenv.glibc}/lib"
  ];
  emacsGccDarwin = builtins.foldl' (drv: fn: fn drv)
    pkgs.emacs
    [

      (drv: drv.override { srcRepo = true; })

      (
        drv: drv.overrideAttrs (
          old: {
            name = "emacsGccDarwin";
            version = "28.0.50";
            src = pkgs.fetchFromGitHub {
              inherit (emacs-nativecomp) owner repo rev sha256;
            };

            configureFlags = old.configureFlags
            ++ [ "--with-ns" ];

            patches = [
              (
                pkgs.fetchpatch {
                  name = "tramp-detect-wrapped-gvfsd.patch";
                  url = "https://raw.githubusercontent.com/nix-community/emacs-overlay/master/patches/tramp-detect-wrapped-gvfsd.patch";
                  sha256 = "1rvz725697md3ir618kkrccsxzd6n0p8yddq7rsh5jg8dbrvjvcx";
                }
              )
            ];

            postPatch = old.postPatch + ''
              substituteInPlace lisp/loadup.el \
              --replace '(emacs-repository-get-version)' '"${emacs-nativecomp.rev}"' \
              --replace '(emacs-repository-get-branch)' '"master"'
            '';

            postInstall = old.postInstall or "" + ''
              ln -snf $out/lib/emacs/28.0.50/native-lisp $out/native-lisp
              ln -snf $out/lib/emacs/28.0.50/native-lisp $out/Applications/Emacs.app/Contents/native-lisp
              cat <<EOF> $out/bin/run-emacs.sh
              #!/usr/bin/env bash
              set -e
              exec $out/bin/emacs-28.0.50 "\$@"
              EOF
              chmod a+x $out/bin/run-emacs.sh
              ln -snf ./run-emacs.sh $out/bin/emacs
            '';
          }
        )
      )
      (
        drv: drv.override {
          nativeComp = true;
        }
      )
    ];
in
_: _:
  {
    inherit emacsGccDarwin;

    # ci = (import ./nix {}).ci;
    # emacsGccDarwinWrapped = pkgs.symlinkJoin {
    #   name = "emacsGccDarwinWrapped";
    #   paths = [ emacsGccDarwin ];
    #   buildInputs = [ pkgs.makeWrapper ];
    #   postBuild = ''
    #     wrapProgram $out/bin/emacs \
    #     --set LIBRARY_PATH ${libPath}
    #   '';
    #   meta.platforms = pkgs.stdenv.lib.platforms.linux;
    #   passthru.nativeComp = true;
    #   src = emacsGccDarwin.src;
    # };
  }
