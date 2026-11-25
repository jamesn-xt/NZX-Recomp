# RTX 4080 community integration, 25 November 2026

This is an experimental integration branch. Correct textures and stable 30 fps
are the target; neither is established by the current results.

## Tested checkpoint

Commit `5452de4046bf0ad8af068d654554c0ea5a1cab52` combines the earlier performance
branch with fxceless main `a26859fc5f512e0db0d7522d3da821c038378ab7`:

- Texture invalidation and resident-texture refresh under VRAM pressure.
- Guards across physical-page aliases and kernel destination writes.
- Byte-copied descriptor synchronization and VRAM reservations.
- VIREX's recently changing texture uploads, adapted to the existing staging
  fence callback and optional levels-on-demand implementation (#510).
- sagouindupm's descriptor-page locking and fingerprint optimization (#774).

The earlier Series S compositor, staging lifetime protection, parallel detiling,
lean windows, wide hashes, grouped rehashing, parallel buffer refresh, and
composed-output capture fix remain integrated. Original author attribution is
retained in Git history. Levels on demand remains disabled by default.

Validation on Windows with an RTX 4080, Ryzen 5 5600 and 32 GB RAM:

- `cargo check --workspace --all-features --locked`: passed.
- `cargo test --workspace --all-features --locked -- --test-threads=1`:
  **570 passed, 0 failed, 10 ignored**. Some optional hardware tests can also
  return early; this is not complete coverage of every hardware path.
- Release build with `--features diagnostics`: passed.
- Executable SHA-256:
  `1E3D1BAB50878CD68BCDEB45A93146B512755C23B3B1DC723C4B001D2E9A0E18`.
- An initial WARP run failed `depth_bounds_pipeline_loads_and_filters_draws`;
  that test passed on the RTX 4080. The complete successful suite above used
  hardware selection rather than forcing WARP globally.

The earlier build rendered one prison view at approximately 5–6 fps, with
missing character surfaces and reported window flicker. The checkpoint has
launched and displayed startup/loading artwork. Its world rendering and
performance are still being evaluated. Menu/loading rates are not gameplay
performance. A screenshot alone does not establish whether temporal flicker is
fixed. Compilation and concurrent GPU tests confound live timing samples.

## Following changes

The working branch additionally carries:

- #606: mark staged textures dirty when their title memory is remapped, with
  staging and placed-resource regressions (laptopnotebook and Carlos Sant'Anna).
- #735: layouts for console texture formats 189–191 (davutsimsekk).
- #699: signature element order and interpolation-mode consistency, allowing
  additional shaders through validation (Mohammed Wasif Ahmed).

Revision `2294a5e` passed **420 tests, 0 failed, 10 ignored** across the complete
GPU, formats and shader crates. Its release build passed. Executable SHA-256:
`7ECC3FE4F217D76B47CBD575897FD6BF354B61CCB94984BC2A571D722471CE5F`.
The matching executable, source and checksums are published as the
[`rtx4080-material-test-20261125` prerelease](https://github.com/jamesn-xt/VIctory/releases/tag/rtx4080-material-test-20261125).
Its runtime evaluation remains separate from the original checkpoint.

## Same-submission texture uploads

Branch `test/rtx4080-texture-prelude` additionally adapts Adrito's #80. A changed
title texture is copied from refreshed staging in the submission's prelude,
before that submission uses it. Copies no longer depend on a dirty flag sampled
earlier while recording the main command list.

Both GPU regressions failed on the preceding code: changing `0x11` to `0x22`
still read `0x11`, and changing `0x33` to `0x44` after a two-list submission
still read `0x33`. Both pass with the adaptation. The full GPU suite passes:
**175 passed, 0 failed, 3 ignored**, with `VICTORY_D3D12_DEBUG=1` requested.

Staging lifetime protection remains: only staging buffers actually copied by a
prelude are marked in flight; unchanged refreshes do not wait, while detiling
waits for the prior reader. The existing staging-fence regression was adapted
to check actual submission-time copies and passes. The adaptive hash polling
for recently changing view memory remains; its earlier inline-copy decision is
superseded by the prelude. Runtime flicker and performance are not yet proven
fixed by these tests.

Clippy initially found an upstream `unnecessary_sort_by` warning in `draw.rs`;
the equivalent reverse-key sort is used on this branch.

## Reproduce

Follow the repository's normal Windows build prerequisites, then:

```powershell
cargo build -p victory --release --features diagnostics --locked
$env:VICTORY_CARD_HASH_WIDE = '1'
$env:VICTORY_TEXTURE_DETILE_WORKERS = '3'
$env:VICTORY_LEAN_WINDOWS = '1'
$env:VICTORY_CARD_REHASH_RUNS = '1'
$env:VICTORY_WIDE_REFRESH = '1'
target/release/victory.exe run 'D:/YourGame/game_scarlett_master.exe' --execute --console series-s
```

Configure the normal local game, data, shader compiler and cache paths as in the
README. No game files, saves, screenshots, captures, translated title shaders,
or raw title logs are included in this branch. Series S uses the integrated
plain bilinear compositor; this is not a DLSS/FSR implementation.

The original builds remain available locally for comparison. Main is unchanged;
the published integration and checkpoint are separate test branches.
