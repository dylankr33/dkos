{
  sprinkles ? (import ./npins).sprinkles,
  ...
}@overrides:
(import sprinkles).new {
  inherit overrides;
  sources = import ./npins;
  inputs =
    { sources, inputs }:
    let
      rustToolchainFor = (
        p:
        p.rust-bin.selectLatestNightlyWith (
          toolchain:
          (toolchain.default.override {
            extensions = [
              "rust-src"
            ];
            targets = [
              "x86_64-unknown-none"
              "riscv64gc-unknown-none-elf"
            ];
          })
        )
      );
    in
    {
      pkgs = import sources.nixpkgs {
        system = "x86_64-linux";
        overlays = [ (import sources.rust-overlay) ];
      };
      getBuildInputs = target: f: f inputs.pkgs.pkgsCross."${target}-embedded".buildPackages;
      pkgsCross = inputs.pkgs.pkgsCross.x86_64-embedded;
      crane =
        let
          crane' = import sources.crane;
        in
        (if (builtins.isFunction crane') then crane' { inherit (inputs) pkgs; } else crane')
        .overrideToolchain
          rustToolchainFor;
      rustToolchain = rustToolchainFor inputs.pkgs;
      limine = sources.Limine;
    };
  outputs =
    {
      outputs,
      crane,
      pkgs,
      rustToolchain,
      getBuildInputs,
      limine,
    }:
    let
      buildKernel =
        arch:
        let
          src =
            let
              root = ./kernel;
            in
            pkgs.lib.fileset.toSource {
              inherit root;
              fileset = pkgs.lib.fileset.unions [
                (crane.fileset.commonCargoSources root)
                (pkgs.lib.fileset.fileFilter (file: file.hasExt "s") root)
                (pkgs.lib.fileset.fileFilter (file: file.hasExt "ld") root)
              ];
            };
        in
        crane.buildPackage {
          inherit src;
          nativeBuildInputs = getBuildInputs arch (p: with p; [ gcc ]);
          CARGO_BUILD_TARGET = (
            if arch == "riscv64" then "${arch}gc-unknown-none-elf" else "${arch}-unknown-none"
          );
          cargoVendorDir = crane.vendorMultipleCargoDeps {
            cargoLockList = [
              ./kernel/Cargo.lock
              "${rustToolchain.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/library/Cargo.lock"
            ];
          };
          doCheck = false;
          strictDeps = true;
        };
    in
    {
      packages.x86_64-linux = {
        dkos-x86_64 =
          let
            kernel = buildKernel "x86_64";
          in
          pkgs.stdenv.mkDerivation {
            name = "dkos-x86_64";
            src = ./.;
            buildCommand = ''
              mkdir -p $out/bin
              mkdir -p iso_root/boot
              cp -v ${kernel}/bin/kernel iso_root/boot/
              mkdir -p iso_root/boot/limine
              cp -v $src/limine.conf iso_root/boot/limine
              mkdir -p iso_root/EFI/BOOT
              cp -v ${limine}/limine-bios.sys ${limine}/limine-bios-cd.bin ${limine}/limine-uefi-cd.bin iso_root/boot/limine/
              cp -v ${limine}/BOOTX64.EFI  ${limine}/BOOTIA32.EFI iso_root/EFI/BOOT/
              xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
                -no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
                -apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
                -efi-boot-part --efi-boot-image --protective-msdos-label \
                iso_root -o $out/dkos-x86_64.iso
              ${outputs.packages.x86_64-linux.limine}/bin/limine bios-install $out/dkos-x86_64.iso
              chmod 0777 $out/dkos-x86_64.iso
            '';
            meta.mainProgram = "runner";
            buildInputs = [
              pkgs.xorriso
              outputs.packages.x86_64-linux.limine
              kernel
            ];
          };
        dkos-riscv64 =
          let
            kernel = buildKernel "riscv64";
          in
          pkgs.stdenv.mkDerivation {
            name = "dkos-riscv64";
            src = ./.;
            buildCommand = ''
              mkdir -p $out/bin
              mkdir -p iso_root/boot
              cp -v ${kernel}/bin/kernel iso_root/boot/
              mkdir -p iso_root/boot/limine
              cp -v $src/limine.conf iso_root/boot/limine
              mkdir -p iso_root/EFI/BOOT
              cp -v ${limine}/limine-uefi-cd.bin iso_root/boot/limine/
              cp -v ${limine}/BOOTRISCV64.EFI iso_root/EFI/BOOT/
              xorriso -as mkisofs\
                --efi-boot boot/limine/limine-uefi-cd.bin \
                -efi-boot-part --efi-boot-image --protective-msdos-label \
                iso_root -o $out/dkos-x86_64.iso
              ${outputs.packages.x86_64-linux.limine}/bin/limine bios-install $out/dkos-x86_64.iso
              chmod 0777 $out/dkos-x86_64.iso
            '';
            buildInputs = [
              pkgs.xorriso
              outputs.packages.x86_64-linux.limine
              kernel
            ];
          };

        limine = pkgs.stdenv.mkDerivation {
          name = "limine";
          src = limine;
          installPhase = ''
            mkdir -p $out/bin
            cp limine $out/bin
          '';
        };
      };

      apps.x86_64-linux = {
        run-x86_64 = {
          type = "app";
          program = "${pkgs.writeShellScriptBin "runner" ''
            cp ${outputs.packages.x86_64-linux.dkos-x86_64}/dkos-x86_64.iso .
            chmod 0777 dkos-x86_64.iso
            exec ${pkgs.qemu}/bin/qemu-system-x86_64 "dkos-x86_64.iso"
          ''}/bin/runner";
        };
        run-riscv64 = {
          type = "app";
          program = "${pkgs.writeShellScriptBin "runner" ''
            cp ${outputs.packages.x86_64-linux.dkos-riscv64}/dkos-riscv64.iso .
            chmod 0777 dkos-riscv64.iso
            exec ${pkgs.qemu}/bin/qemu-system-x86_64 "dkos-riscv64.iso"
          ''}/bin/runner";
        };
      };

      devShells.x86_64-linux.default = crane.devShell {
        packages = with pkgs; [
          pkgsCross.riscv64-embedded.buildPackages.gcc
          rust-analyzer
          qemu
        ];
      };
    };
}
