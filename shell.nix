with import <nixpkgs> {};

pkgs.mkShell {
  buildInputs = with pkgs; [
    cmake pkg-config llvmPackages_14.libclang  llvmPackages_14.clang openssl mbedtls rustup  rust-analyzer fontconfig  gdk-pixbuf cairo gtk3 webkitgtk wayland libxkbcommon python3
  ];
  nativeBuildInputs = with pkgs; [
    rustPlatform.bindgenHook
  ];

  LIBCLANG_PATH = "${llvmPackages_14.libclang.lib}/lib";

  LD_LIBRARY_PATH = with pkgs.xorg; "${pkgs.mesa}/lib:${libX11}/lib:${libXcursor}/lib:${libXxf86vm}/lib:${libXi}/lib:${libXrandr}/lib:${pkgs.libGL}/lib:${pkgs.gtk3}/lib:${pkgs.cairo}/lib:${pkgs.gdk-pixbuf}/lib:${pkgs.fontconfig}/lib:${wayland}/lib:${libxkbcommon}/lib";

}
