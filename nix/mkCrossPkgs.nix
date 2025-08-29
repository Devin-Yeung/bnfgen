{
  nixpkgs,
  rust-overlay,
  localSystem,
  target,
}:
import nixpkgs {
  overlays = [ (import rust-overlay) ];
  inherit localSystem;
  crossSystem = {
    config = target;
    # it seems that only when enable this, the aarch64 build will be truly static
    useLLVM = true;
  };
}
