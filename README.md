<img src="assets/icon.svg" width="64" height="64" alt="">

# fastcraft

A [Horizon](https://playhorizon.dev) native mod that crafts straight from the
recipe book, in the spirit of
[OneClickCrafting](https://github.com/BreadMoirai/OneClickCrafting).

- **Ctrl+click a recipe** to place it and take the result into your
  inventory; without Ctrl a click is the vanilla placement.
- **Ctrl+Shift+click** to keep crafting until the materials run out or the
  inventory cannot hold another item.
- **+ the drop key (Q)** to drop the crafted items: with Ctrl alone the one
  result, with Ctrl+Shift whatever the inventory could not fit.
- **Enter/Space** repeats the last placed recipe — Ctrl+Space crafts each
  repeat as it lands, in the inventory's 2x2 grid and the crafting table
  alike, locally and on servers.

## Layout

`mixins/fastcraft/` is the mixin bundle (`horizon-mixin.toml` plus its
helpers). It pins Horizon `fdc87ae` (Nightly 2026-11-08) in
`[compatibility] horizon_revision`; retarget it at a newer Nightly's commit
when you rebuild on one.

## Build and play

Native mods run unsandboxed, so the launcher asks for an explicit trust flag:

```sh
horizon-launcher native-build /path/to/fastcraft/mixins/fastcraft \
  --instance NAME --trust-native-code
```

`horizon-launcher settings native-profile dev` makes rebuilds while
developing much faster (unoptimized game crate; don't play on it).

To validate the manifest and sources without compiling the game:

```sh
horizon-mixins plan \
  --workspace ~/.local/share/horizon-launcher/native/source/<revision> \
  --mods mixins --out /tmp/plan.json
```
