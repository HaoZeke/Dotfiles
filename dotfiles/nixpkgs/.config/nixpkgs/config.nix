{
  packageOverrides = super:
    let self = super.pkgs;
    in {

      rEnv = super.rWrapper.override {
        packages = with self.rPackages; [
          ggplot2
          knitr
          tidyverse
          tidybayes
          (buildRPackage {
            name = "rethinking";
            src = self.fetchFromGitHub {
              owner = "rmcelreath";
              repo = "rethinking";
              rev = "d0978c7f8b6329b94efa2014658d750ae12b1fa2";
              sha256 = "1qip6x3f6j9lmcmck6sjrj50a5azqfl6rfhp4fdj7ddabpb8n0z0";
            };
            propagatedBuildInputs =
              [ coda MASS mvtnorm loo shape rstan dagitty ];
          })
        ];
      };
    };
}
