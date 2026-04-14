{
  description = "maski — Interactive TUI for mask";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    md4x = {
      url = "github:unjs/md4x";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, md4x }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "maski";
            version = "0.1.0";
            src = ./.;

            postUnpack = ''
              rm -rf $sourceRoot/vendor/md4x
              mkdir -p $sourceRoot/vendor
              cp -r ${md4x} $sourceRoot/vendor/md4x
            '';

            cargoHash = "sha256-CnoXPGn0n8SiAkEFZq6xrbiNDx/jOAIh2/w42l1Zfb0=";

            nativeBuildInputs = [ pkgs.pkg-config ];

            meta = {
              description = "Interactive TUI for mask — browse and run maskfile commands with fuzzy search";
              homepage = "https://github.com/ManUtopiK/maski";
              license = pkgs.lib.licenses.mit;
              mainProgram = "maski";
            };
          };
        }
      );
    };
}
