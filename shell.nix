{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = with pkgs; [
    expat
    fontconfig
    freetype
    freetype.dev
    libGL
    pkg-config
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    libxkbcommon
    libGL
    wayland
    vulkan-loader
    cmake
    nasm
    gtk3
  ];

  LD_LIBRARY_PATH =
    builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib:${pkgs.libxkbcommon}/lib:${pkgs.libGL}/lib:${pkgs.wayland}/lib" buildInputs;
}

# { pkgs ? import <nixpkgs> {} }:

# pkgs.mkShell {
#   nativeBuildInputs = with pkgs; [
#   ];
#   LD_LIBRARY_PATH = "";
#   RUST_BACKTRACE=1;
# }
