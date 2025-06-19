{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = [ pkgs.pkg-config pkgs.openssl pkgs.delta ];

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
    export LD_LIBRARY_PATH="${pkgs.openssl.out}/lib:$LD_LIBRARY_PATH"
  '';
}
