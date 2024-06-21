{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/24.05";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay"; # use master since newer Rust versions are continuously pushed
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
    in {
      defaultApp = utils.lib.mkApp {
        drv = self.defaultPackage."${system}";
      };

      packages = {
        cgvg-rs = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cgvg.rs";
          version = "0.0.1";

          src = pkgs.fetchFromGitHub {
            owner = "adfaure";
            repo = "${pname}";
            rev = "refs/heads/main";
            sha256 = "sha256-O2ZaBU7YWlQdmItv895U9B5/kQcgHFCBjPUQg5ilt6k=";
          };

          cargoHash = "sha256-qFCHngWcWmZCV7jAa8pvDuhd/CSsS7Q8EC+qyggosLk=";

          meta = with pkgs.lib; {
            description = "";
            longDescription = '''';
            homepage = "https://github.com/adfaure/cgvg.rs";
            license = licenses.mit;
            platforms = platforms.all;
            broken = false;
          };
        };
      };

      devShell = with pkgs;
        mkShell rec {
          buildInputs = [
            rust-bin.stable.latest.default
            rust-analyzer
          ];
        };

      formatter = nixpkgs.legacyPackages.x86_64-linux.alejandra;
    });
}
