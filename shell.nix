let
  rust_overlay = import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
in
pkgs.mkShell {
  buildInputs = [
    (pkgs.rust-bin.stable.latest.default.override {
      targets = [ "x86_64-unknown-linux-musl" ];
    })
    pkgs.cargo-zigbuild
    pkgs.zig
    pkgs.pkg-config
  ];

  shellHook = ''
    echo "Ready to zig-build!"
  '';
}
