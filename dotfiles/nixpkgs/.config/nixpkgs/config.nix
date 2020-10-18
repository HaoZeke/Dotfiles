{
  packageOverrides = super:
    let
      self = super.pkgs;
      nix = {
        package = self.nixFlakes;
        extraOptions =
          "experimental-features = nix-command flakes recursive-nix";
      };
      ascii = with self.rPackages;
        buildRPackage {
          name = "ascii";
          src = self.fetchFromGitHub {
            owner = "mclements";
            repo = "ascii";
            rev = "c8598ee7963cc635373457120b5eecd5933bd6f3";
            sha256 = "0xqihbfyn7s9f32j2jamj7yjm3mzg5smfbrdpyiwdncbrigkr1zq";
          };
          propagatedBuildInputs = [ digest codetools survival ];
        };
      scholarnetwork = with self.rPackages;
        buildRPackage {
          name = "scholarnetwork";
          src = self.fetchFromGitHub {
            owner = "pablobarbera";
            repo = "scholarnetwork";
            rev = "11f37da8ab097c298f03eb9f1ee3233e9110c2d7";
            sha256 = "15kg0gjap4a910q33cyrpvixmn2wmzdcnjwv9qh4r5vfdlsnwxlp";
          };
          propagatedBuildInputs = [ igraph scholar stringr networkD3 ];
        };
      rethinking = with self.rPackages;
        buildRPackage {
          name = "rethinking";
          src = self.fetchFromGitHub {
            owner = "rmcelreath";
            repo = "rethinking";
            rev = "3b48ec8dfda4840b9dce096d0cb9406589ef7923";
            sha256 = "1cinz87q0z9vxpmz98r5y2vby3z8av9p8w283ihzvy4pc1fz0r9b";
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
          # Tidyverse and More
          tidyverse
          modelr
          purrr
          forcats
          gridExtra
          broom
          future
          # Plotting
          ggplot2
          gganimate
          GGally
          ggfortify
          cowplot
          ggrepel
          RColorBrewer
          corrplot
          rpart_plot
          ####################
          # Machine Learning #
          ####################
          caret
          # MLR3
          mlr3
          mlr3viz
          mlr3learners
          mlr3pipelines
          mlr3filters
          # Stan Stuff
          rstan
          tidybayes
          # Drivers
          e1071 # Misc collection, includes fourier
          pvclust # Better clustering
          # Time series
          forecast
          swdft
          # Validation
          mgcv
          # Forests and Trees
          xgboost
          ranger
          DALEX
          DALEXtra
          # PCA
          FactoMineR
          factoextra
          # Regression
          lindia
          rpart
          #############
          # Utilities #
          #############

          # Text
          orgutils
          latex2exp
          kableExtra
          knitr
          data_table
          printr

          # Other
          microbenchmark

          # Network
          webchem

          # Mixed Linear Models
          xtable
          emmeans
          car
          lmerTest
          skimr
          broom

          ############
          # Devtools #
          ############
          devtools
          # Built above
          rethinking
          tidybayes_rethinking
          ascii
          scholarnetwork
        ];
      };
    };
}

# Local Variables:
# firestarter: "nix-env -f '<nixpkgs>' -iA rEnv"
# firestarter-default-type: (quote failure)
# End:
