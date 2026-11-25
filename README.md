# NZX Recomp

NZX Recomp is a standalone Windows x64 runtime project. The active source baseline comes from the earlier RTX 4080 VIctory integration at `47f372f07aef3a9d20f63eb846c6079a4521a426`. That build reached interactive prison gameplay at about 5.2 fps in its Series S profile. The first milestone is to load all visible textures along the same prison route while sustaining about 5 fps. Higher frame rates are the next step after that visual check passes.

The earlier DexNom-based NZX test releases stay available for comparison. This baseline includes two targeted DexNom changes. The texture queue alternates new uploads with retries, and the loader counts resident pages even when both ends of a texture are absent.

## Build and run

Use the pinned Rust toolchain, Visual Studio C++ build tools, and Windows SDK. Run the following in PowerShell.

```powershell
.\Build.ps1 -OutputDirectory 'D:\NZX Recomp'
```

Use `Launch-PerformanceProfile.ps1` from that directory. Pass your game executable, a local configuration TOML (see `victory.example.toml`), and the directory containing `dxc.exe` and `dxil.dll`. The earlier measured Series S profile used `-InflightFetch`.

```powershell
.\Launch-PerformanceProfile.ps1 -GameExe 'D:\My Game\game.exe' -Config 'D:\NZX Recomp\victory.toml' -DxcDirectory 'D:\DXC\bin\x64' -InflightFetch
```

The ZIP release needs a local table file. Run `.\Initialize-FormatTables.ps1 -Destination .\bcpack-tables.bin` from the extracted directory. Build.ps1 prepares that file when building locally. Game files and saves remain local.

The prior [RTX 4080 validation notes](docs/rtx4080-community-validation.md) describe tests of the original VIctory build. The [historical VIctory README](docs/README.victory.md) provides upstream documentation and credits. The project preserves contributor attribution and uses [GPL-3.0-or-later](LICENSE).
