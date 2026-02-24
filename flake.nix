{
  description = "tuna";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    {
      self,
      nixpkgs,
      systems,
    }:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          tuna = pkgs.buildGoModule {
            pname = "tuna";
            version = "master";
            src = self;
            subPackages = [ "cmd/tuna" ];
            vendorHash = "sha256-gozHi23F/RdeChoF6WKc9a62b2qQpZd0fOV8+Laeyt4=";

            nativeBuildInputs = [ pkgs.makeWrapper ];

            postInstall = ''
              wrapProgram "$out/bin/tuna" \
                --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.chromium ]}
            '';

            meta = {
              mainProgram = "tuna";
            };
          };
        in
        {
          default = tuna;
          inherit tuna;
        }
      );

      devShells = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              chromium
              go
              gopls
              gofumpt
            ];
          };
        }
      );
    };
}
