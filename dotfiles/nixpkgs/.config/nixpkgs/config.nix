{
  packageOverrides = super:
    let
      self = super.pkgs;
      rethinking = with self.rPackages;
        buildRPackage {
          name = "rethinking";
          src = self.fetchFromGitHub {
            owner = "rmcelreath";
            repo = "rethinking";
            rev = "d0978c7f8b6329b94efa2014658d750ae12b1fa2";
            sha256 = "1qip6x3f6j9lmcmck6sjrj50a5azqfl6rfhp4fdj7ddabpb8n0z0";
          };
          propagatedBuildInputs = [ coda MASS mvtnorm loo shape rstan dagitty ];
        };
      tidybayes_rethinking = with self.rPackages;
        buildRPackage {
          name = "tidybayes.rethinking";
          src = self.fetchFromGitHub {
            owner = "mjskay";
            repo = "tidybayes.rethinking";
            rev = "df903c88f4f4320795a47c616eef24a690b433a4";
            sha256 = "1jl3189zdddmwm07z1mk58hcahirqrwx211ms0i1rzbx5y4zak0c";
          };
          propagatedBuildInputs =
            [ dplyr tibble rlang MASS tidybayes rethinking rstan ];
        };
    in {
      rEnv = super.rWrapper.override {
        packages = with self.rPackages; [
          tidyverse
          devtools
          modelr
          purrr
          forcats
          ####################
          # Machine Learning #
          ####################
          # MLR3
          mlr3
          mlr3viz
          mlr3learners
          mlr3pipelines
          # Plotting tools
          ggplot2
          cowplot
          ggrepel
          RColorBrewer
          # Stan Stuff
          rstan
          tidybayes
          # Text Utilities
          orgutils
          latex2exp
          kableExtra
          knitr
          data_table
          printr
          # Devtools Stuff
          rethinking
          tidybayes_rethinking
        ];
      };
    };
}

# Local Variables:
# firestarter: "nix-env -f '<nixpkgs>' -iA rEnv"
# firestarter-default-type: (quote failure)
# End:
