{
  sprinkles ? (import ./npins).sprinkles,
  ...
}@overrides:
(import sprinkles).new {
  inherit overrides;
  sources = import ./npins;
  inputs =
    { sources, inputs }:
    {
      pkgs = import sources.nixpkgs {
        system = "x86_64-linux";
        overlays = [ (import sources.rust-overlay) ];
      };
      pkgsCross = inputs.pkgs.pkgsCross.x86_64-embedded;
      crane =
        let
          crane' = import sources.crane;
        in
        (if (builtins.isFunction crane') then crane' { inherit (inputs) pkgs; } else crane')
        .overrideToolchain
          (
            p:
            p.rust-bin.selectLatestNightlyWith (
              toolchain:
              toolchain.default.override {
                extensions = [
                  "rust-src"
                ];
                targets = [ "x86_64-unknown-none" ];
              }
            )
          );
    };
  outputs =
    {
      crane,
      pkgs,
      pkgsCross,
    }:

    {
      packages.x86_64-linux.dkos-x86_64 = crane.buildPackage {
        pname = "kernel";
        cargoExtraArgs = "-Zbuild-std=core,compiler_builtins";
        doCheck = false;
        strictDeps = true;
        src =
          let
            unfilteredRoot = ./kernel;
          in
          pkgs.lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = pkgs.lib.fileset.unions [
              (crane.fileset.commonCargoSources unfilteredRoot)
              (pkgs.lib.fileset.fileFilter (file: file.hasExt "s") unfilteredRoot)
              (pkgs.lib.fileset.fileFilter (file: file.hasExt "ld") unfilteredRoot)
            ];
          };
      };

      devShells.x86_64-linux.default = crane.devShell {
        packages = with pkgs; [
          rust-analyzer
        ];
      };
    };
}
