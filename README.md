# NZX Recomp

A Windows x64 runtime project based on [DexNom/VIctory](https://github.com/DexNom/VIctory), with NVIDIA compatibility, texture correctness and frame-time improvements as the immediate development priorities.

The target is stable **30 fps with correct rendering**. That target has not been reached. A successful build, a menu running at 30 fps, and a shader test passing are not evidence of stable gameplay performance.

## Current baseline

This project starts from DexNom revision `921d8d3b43b44f3a6ca4b44095edd3c4baf10a87` and preserves its renderer, texture streaming, native temporal processing and GPU memory work. The initial NVIDIA test machine is an RTX 4080, Ryzen 5 5600 and 32 GB RAM. Runtime findings and measured changes are recorded in [development notes](docs/nzx-development.md).

Earlier RTX 4080 VIctory builds remain available in [jamesn-xt/VIctory](https://github.com/jamesn-xt/VIctory/releases). Useful fixes will be adapted individually and tested against this newer baseline.

## Build on Windows

Use the pinned Rust toolchain, Visual Studio C++ build tools and Windows SDK. Build dependencies and the format-table prerequisite are described in the [original upstream documentation](README.upstream.md). Keep game files, saves and shader caches outside this checkout.

```powershell
.\Initialize-FormatTables.ps1
cargo build --locked --release -p victory --features diagnostics --bins
```

The Cargo package and internal crate names retain `victory` for compatibility. The programs are `nzx-recomp.exe`, `nzx-play.exe` and `nzx-warm.exe`. Copy all three from `target\release` into a writable directory outside the checkout, or use `Build.ps1` to build and copy them:

```powershell
.\Build.ps1 -OutputDirectory 'D:\NZX Recomp' -GamePath 'D:\My Game\game.exe'
```

## Run and preload

From that output directory:

```powershell
.\nzx-play.exe 'D:\My Game\game.exe'
.\nzx-warm.exe 'D:\My Game\game.exe'
```

Play uses the Series X profile inherited from DexNom. Preloading fills shader and compute-pipeline caches; graphics pipelines also need states encountered during gameplay. Saves go to `state`, caches to `cache`, and diagnostics to `logs`, beside the launchers. An existing game installation is required; this repository supplies none of its files.

The launchers also accept a game path stored in `game-path.txt`. `--rt` enables the donor's experimental ray-tracing path, which still needs NVIDIA validation. It is not enabled by default. `--warp` selects software rendering for diagnostics. The advanced CLI and configuration retain upstream `VICTORY_*` settings.

## Credits and license

- [Marvisak/VIctory](https://github.com/Marvisak/VIctory) and its contributors created the original runtime.
- [DexNom/VIctory](https://github.com/DexNom/VIctory) supplies the rendering and streaming baseline. Its [testing report](BENCHMARK.md) records Radeon findings; those figures are not NZX Recomp NVIDIA benchmarks.
- ZSlayerHQ, triippywhale, asdaasdsaddf and the victory-world contributors provided the preceding rendering work, as recorded in the preserved Git history.
- [XCL](https://github.com/xcl-dev/xcl) informed the donor project's investigation of shader translation, memory synchronization and performance.

Source is [GPL-3.0-or-later](LICENSE). Contributor history is preserved. Separately supplied BCPack format tables are outside that license; see the [asset policy](docs/asset-policy.md). Game data, saves, captures and generated title shaders are not release assets.
