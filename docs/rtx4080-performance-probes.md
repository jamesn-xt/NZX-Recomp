# RTX 4080 performance investigation

The target is correct world rendering at a sustained 30 fps. That target has
**not been demonstrated**. Menu frame rates do not count as world performance.
The first prison scene renders, but surfaces, character pieces and indirect
lighting can still be missing.

This work starts from fxceless/victory-fork `c96b4f2`, including its depth-bounds
support, indexed-texture rechecks, early decoration-bake uploads and faster
submission defaults. Earlier local experiments and upstream main are preserved.

## Integrated community changes

Original commit authors are retained in the history.

| Source | Change | Control |
|---|---|---|
| [PR 141](https://github.com/fxceless/victory-fork/pull/141) | Protect texture staging from an unfinished GPU copy. Its follow-up `d586d36` waits only before a real detile and marks actual uploads, avoiding waits on unchanged textures. | Correctness fix, always active |
| [PR 493](https://github.com/fxceless/victory-fork/pull/493) | Hash a card-memory piece with 256 threads instead of a serial walk on one thread. | `VICTORY_CARD_HASH_WIDE=1` |
| [PR 467](https://github.com/fxceless/victory-fork/pull/467) | Revisit marked buffer slots and retain their aggregate spans; split buffer/texture timing. | `VICTORY_HEAP_PASS_FULL=1` restores the old pass |
| [PR 271](https://github.com/fxceless/victory-fork/pull/271) | Detile large 2D subresources in independent row bands. | `VICTORY_TEXTURE_DETILE_WORKERS=2..4` |
| [PR 265](https://github.com/fxceless/victory-fork/pull/265) | Keep eligible read-only title-memory windows on the card without a whole CPU copy, using tiled buffers and an upload ring. | `VICTORY_LEAN_WINDOWS=1` |
| [PR 188](https://github.com/fxceless/victory-fork/pull/188), including [PR 32](https://github.com/fxceless/victory-fork/pull/32) | Composite the second present plane and answer Series S upscale dispatches with a plain bilinear stretch; use an FP16 stand-in for RGB9E5 render targets. | `--console series-s` |

The Series S stretch is a compatibility approximation. It is not the PC
FidelityFX SDK, has no temporal reconstruction, and ignores motion/depth inputs.
The compositor handles the second plane without implementing all placement
metadata or further planes. Lean windows assume the observed windows are not
shader-written; their diagnostic counter must be watched. The structure builder
disables that option because it can write them.

Local integration adaptations pass heap spans as one range and keep the default
and diagnostic configurations lint-clean. No broad lint suppression was added.

## Additional GPU verification and diagnostic fix

An actual FXC/D3D12 test runs both card-hash implementations, including skewed
pieces, changes at piece boundaries, compare-without-store and compare-and-store.
It checks that untouched result bytes remain intact. These tests use synthetic
buffers only.

During the Series S run, the native window displayed the logo while diagnostic
PNGs were black. The PNG and black-frame probe had read the primary plane before
the UI overlay. They now read the composed back buffer, after both planes, with
explicit copy-source transitions and restoration to present state. The existing
synthetic overlay test additionally checks the captured blue scene, green alpha
blend, opaque red overlay, and non-black probe result.

## Measured so far

Windows, RTX 4080 16 GB, Ryzen 5 5600, 32 GB RAM, NVIDIA driver 617.42. Diagnostic
builds with the debug layer and DRED off. Each run has a finite time limit and a
2 GiB available-commit guard. No operating-system watchdog or paging settings
were changed.

Caption samples are collected about every five seconds. The table uses the last
12 world samples of each completed run; these are sampled frame-rate summaries,
not per-frame percentiles or proof of hitch-free play.

| Completed run | Median frame time of final samples | Corresponding fps | Private memory |
|---|---:|---:|---:|
| Main plus original staging protection, Series X, 904 seconds | 340.2 ms | 2.94 | 22.94 GiB |
| Performance binary, old heap pass, narrow hash, one detile worker, 602 seconds | 385.7 ms | 2.59 | 22.72 GiB |
| Same binary, marked heap pass, wide hash, three detile workers, 602 seconds | 304.7 ms | 3.28 | 22.72 GiB |
| Series S, lean windows, wide/grouped hashes, three detile workers, 900 seconds | 238.9 ms | 4.19 | 17.15 GiB |
| Series S, staging follow-up and parallel buffer refresh, 602 seconds | 193.2 ms | 5.18 | 17.80 GiB |

These runs share a content-addressed cache, startup streaming varies, and tests
overlapped part of the control run. They do not establish a controlled causal
speed-up or an individual feature's benefit. Both completed Series X variants
visibly reached the prison window scene. The structure builder remains off.

Both Series S runs visibly reach gameplay. The final run also enables the
existing `VICTORY_WIDE_REFRESH=1` because buffer preparation remained expensive.
Before scripted input, seconds 350-470 of that run contain 23 world samples:
median 185.9 ms (5.38 fps), sampled 95th percentile 201.2 ms, and roughly 4,350
draws per frame. After moving and rotating into the room, the final samples
contain roughly 5,760-6,020 draws per frame at the 5.18 fps median above. These
views differ; the figures do not isolate the staging fix's causal benefit.

Four-second scripted holds of backward movement and camera rotation visibly
change the character/view from the prison window into the room. This confirms
those input paths, not complete mission playability. Missing surfaces and
lighting remain. The final combined Series S profile is the best measured
performance option here; Series X builds are retained as a visual fallback.

No device removal was observed in these runs. Every completed run ended at its
time guard. The lean-window counter reports zero declared writes discarded.
This combines a different console profile with memory and submission switches,
so the memory reduction cannot be attributed to lean windows alone. Final
physical memory available was about 6.45 GiB, with about 11.72 GiB working set.

The composed-capture fix was also checked in the title: a saved PNG now contains
the loading-screen image and later the gameplay prompt, matching the native
window instead of saving the primary plane's black menu image.

The earlier Series X run left roughly 1.2 GiB physical memory available and showed
heavy paging during streaming; the observed Series S run leaves several GiB more.
CPU buffer preparation and copy-back remain much larger than a 33.3 ms frame
budget. These timers overlap; they must not be summed, and a fence-wait duration
is not a measurement of the GPU's command execution time.

## Checks

- All-feature workspace suite at `ae3f3de`: 537 passed, zero failed, eight ignored,
  run serially. Optional tool-dependent tests may also return early internally;
  this count is not a claim that every test exercised hardware.
- Default and all-feature workspace Clippy passed at that integration.
- After composed-capture and staging-follow-up changes: GPU suite 141 passed,
  zero failed, one ignored; all-feature workspace Clippy and default GPU Clippy
  passed; diagnostic build passed.
- Repository asset policy passed. No game files, captures, saves, derived shader
  data or runtime logs are included here.

An earlier parallel suite hit an in-place-allocation decommit assertion. The
isolated retry and subsequent serial workspace suite passed. That experimental
in-place mode is not enabled in these title runs.

## Remaining work

Compare a matching warm checkpoint without overlapping builds/tests, and profile
the remaining CPU buffer preparation and copy-back costs. The final run still
settles about once a frame for a table constant-buffer fill; [PR 462](https://github.com/fxceless/victory-fork/pull/462)
is a candidate to evaluate with resource-state and lifetime checks, not integrated
here. Investigate the remaining refused wave64 compute programs and missing
lighting without enabling known-hanging builder or wide-wave experiments.
Reducing diagnostic overhead or menu pacing alone does not satisfy the 30 fps
world target.
