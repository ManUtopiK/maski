{
  description = "maski — Interactive TUI for mask";

  nixConfig = {
    extra-substituters = [ "https://maski.cachix.org" ];
    extra-trusted-public-keys = [ "maski.cachix.org-1:D5Ok9Mln7WxD7vm5ADYL92kTlrN6tB1y1V5Yq2UGmUw=" ];
  };

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
            version = "0.1.2";
            src = ./.;

            postUnpack = ''
              rm -rf $sourceRoot/vendor/md4x
              mkdir -p $sourceRoot/vendor
              cp -r ${md4x} $sourceRoot/vendor/md4x
            '';

            # Vendor straight from the lockfile so the hash never goes stale on
            # a version bump (a hardcoded cargoHash must be regenerated every time
            # Cargo.lock changes).
            cargoLock.lockFile = ./Cargo.lock;

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
