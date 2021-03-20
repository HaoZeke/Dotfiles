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
            rev = "0aba97c523f9b663c0d453a3bbe66558bb2355bf";
            sha256 = "0ygy4s9hcr97bbw6m232cn1fjxi61cbv9n0v39dcbgkszrx2am3g";
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
      mixedup = with self.rPackages;
        buildRPackagee {
          name = "mixedup";
          src = self.fetchFromGitHub {
            owner = "mjskay";
            repo = "tidybayes.rethinking";
            rev = "df903c88f4f4320795a47c616eef24a690b433a4";
            sha256 = "1jl3189zdddmwm07z1mk58hcahirqrwx211ms0i1rzbx5y4zak0c";
          };
          propagatedBuildInputs = [ lme4 ];
        };
    in {
      rEnv = super.rWrapper.override {
        packages = with self.rPackages; [
          # Tidyverse and More
          tidymodels
          tidyverse
          modelr
          purrr
          forcats
          gridExtra
          broom
          broom_mixed
          future
          pkgdown
          ICC
          # Symengine
          # symengine
          odeintr
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
          lattice
          dotwhisker
          # redres
          afex
          sjPlot
          sjmisc
          glmmTMB
          simr
          merTools
          predictmeans
          tufte
          prettydoc
          htmlTable
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
          stargazer

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
          multcomp
          nlme
          nlmeU
          lme4
          haven
          MuMIn
          rms
          tableone

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
