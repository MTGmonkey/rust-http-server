{rustPlatform, ...}:
rustPlatform.buildRustPackage {
  name = "rust_http_server";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
  nativeBuildInputs = [];
  buildInputs = [];
  configurePhase = '''';
  buildPhase = '''';
  installPhase = '''';
  meta = {
    mainProgram = "rust_http_server";
    description = "bare minimum, serves the current directory";
    homepage = "https://mtgmonkey.net";
  };
}
