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
            sha256 = "sha256-vAYQq8XAEKdTvxQ7t3neRmwKDPu4UNsXHE/n8hSKFxo=";
          };

          cargoHash = "sha256-Jk+xGNhLQp7x3c13sDwrn8jz+9OgPhPXjaMAbHKLfM0=";

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
