{
  lib,
  rustPlatform,
  installShellFiles,
}:
rustPlatform.buildRustPackage rec {
  pname = "wallpaper_manager";
  version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  postInstall = ''
    installShellCompletion --cmd wallpaper-manager \
      --bash completions/wallpaper-manager.bash \
      --fish completions/wallpaper-manager.fish \
      --zsh completions/_wallpaper-manager
  '';

  nativeBuildInputs = [installShellFiles];

  meta = {
    description = "Daemon for unified interaction with wallpaper daemons";
    homepage = "https://github.com/yunfachi/wallpaper-manager";
    license = lib.licenses.gpl3Plus;
    mainProgram = "wallpaper-manager";
    maintainers = with lib.maintainers; [yunfachi];
  };
}
