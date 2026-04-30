{
  description = "Bloodborne save editor — Tauri desktop + WASM web";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        linuxDeps = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          pkg-config
          openssl
          glib
          gtk3
          libsoup_3
          webkitgtk_4_1
          librsvg
          libayatana-appindicator
          dbus
        ]);
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            wasm-pack
            wasm-bindgen-cli
            binaryen
            nodejs_20
            cargo-watch
          ] ++ linuxDeps;

          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.lib.makeSearchPath "lib/pkgconfig" linuxDeps}:$PKG_CONFIG_PATH"
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath linuxDeps}:$LD_LIBRARY_PATH"
            echo "Bloodborne save editor dev shell"
            echo "  npm install              install JS deps"
            echo "  npm run wasm:build       build wasm crate"
            echo "  npm run web:dev          run web (browser) dev server"
            echo "  npm run web:build        build static web bundle"
            echo "  npm run dev              run Tauri desktop dev"
          '';
        };
      });
}
