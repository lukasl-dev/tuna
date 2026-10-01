{
  description = "tuna";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs@{ crane, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        { pkgs, ... }:
        let
          craneLib = crane.mkLib pkgs;

          commonArgs = {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;
          };

          artifacts = craneLib.buildDepsOnly commonArgs;

          tuna = craneLib.buildPackage (
            commonArgs
            // {
              cargoArtifacts = artifacts;
              nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
                pkgs.makeWrapper
              ];
              postInstall = pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
                wrapProgram "$out/bin/tuna" \
                  --prefix PATH : ${pkgs.lib.makeBinPath [
                    pkgs.chromium
                    pkgs.chromedriver
                  ]}
              '';
              meta.mainProgram = "tuna";
            }
          );
        in
        {
          packages = {
            default = tuna;
            inherit tuna;
          };

          devShells.default = craneLib.devShell {
            inputsFrom = [ tuna ];
            packages = with pkgs; [
              rust-analyzer
            ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
              chromium
              chromedriver
            ];
          };

          checks.default = tuna;
        };
    };
}
