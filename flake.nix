{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";

    # crane builds Rust packages with good Nix integration — its key feature
    # is building dependencies separately so incremental rebuilds only
    # recompile your code, not all dependencies.
    crane.url = "github:ipetkov/crane";

    # fenix provides the Rust toolchain (rustc, cargo, etc.) via Nix rather
    # than rustup, keeping everything reproducible and flake-managed.
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      fenix,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Use the stable Rust toolchain from fenix.
        toolchain = fenix.packages.${system}.stable.toolchain;

        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        # cleanCargoSource strips files irrelevant to the build (docs, scripts,
        # etc.) so changes to them don't invalidate the Nix cache.
        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;

          # libiconv is only needed on macOS; this keeps the flake cross-platform.
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        # Build dependencies in a separate derivation. crane caches this layer
        # independently, so subsequent builds only recompile hayal itself.
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        hayal = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in
      {
        # checks run via `nix flake check` — useful in CI or as a pre-push gate.
        checks = {
          inherit hayal;

          # clippy with --deny warnings turns all warnings into errors.
          hayal-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          # Enforces consistent formatting via `rustfmt`.
          hayal-fmt = craneLib.cargoFmt { inherit src; };

          # Runs the full test suite.
          hayal-test = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
        };

        packages.default = hayal;

        apps.default = flake-utils.lib.mkApp { drv = hayal; };

        # devShell inherits all checks above and adds developer tooling:
        #   cargo-watch  — reruns cargo commands on file changes
        #   cargo-expand — expands macros for debugging
        #   rust-analyzer — LSP for editor integration
        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            cargo-watch
            cargo-expand
            rust-analyzer
          ];
        };
      }
    );
}
