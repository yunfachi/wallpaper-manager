{
  lib,
  rustPlatform,
  installShellFiles,
}:
rustPlatform.buildRustPackage rec {
  pname = "wallpaper_manager";
  version = "0.1.0";

  src = ./.;

  nativeBuildInputs = [installShellFiles];

  cargoLock.lockFile = ./Cargo.lock;

  postInstall = ''
    installShellCompletion --cmd wallpaper-manager \
      --bash completions/wallpaper-manager.bash \
      --fish completions/wallpaper-manager.fish \
      --zsh completions/_wallpaper-manager
  '';

  meta = {
    description = "Daemon for unified interaction with wallpaper daemons";
    homepage = "https://github.com/yunfachi/wallpaper-manager";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [yunfachi];
    mainProgram = "wallpaper-manager";
  };
}
