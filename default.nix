{ pkgs ? import <nixpkgs> {}, lib ? pkgs.lib }:

{
  spwn = pkgs.rustPlatform.buildRustPackage {
    name = "spwn";
    src = ./.;
    nativeBuildInputs = with pkgs; [
      pkg-config
    ];
    buildInputs = with pkgs; [
      openssl
    ];
    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    meta = with lib; {
      description = "A language for Geometry Dash triggers";
      license = licenses.mit;
    };
  };
}
