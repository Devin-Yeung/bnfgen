{
  pkgs,
  crane,
  target,
}:
(crane.mkLib pkgs).overrideToolchain (
  p:
  p.rust-bin.stable.latest.default.override {
    targets = [
      target
    ];
  }
)
