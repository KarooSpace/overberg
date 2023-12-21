{
  description = "KarooSpace Mission Control";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      buildInputs = with pkgs; [
        vulkan-loader
        wayland
        wayland-protocols
        libxkbcommon
        expat
        fontconfig
        freetype
        freetype.dev
        libGL
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
        systemd
        pkg-config
        llvm
        openssl
        systemd
        cargo-espflash
      ];
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
            extensions = [ "rust-src" ];
            targets = [ "riscv32imc-unknown-none-elf"];
          }))
        ] ++ buildInputs;

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        # Extra inputs can be added here
      };
    });
}

