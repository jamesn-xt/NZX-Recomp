# In-flight copy-back experiment

The integration builds on `test/rtx4080-texture-prelude` and adapts Jonirulah's
[#648](https://github.com/fxceless/victory-fork/pull/648). It preserves the lean
window upload ring and the submission-time texture-copy fence tracking.

At code revision `155d483`, the complete GPU suite passes both with the switch
unset and with `VICTORY_CARD_FETCH_INFLIGHT=1`: **181 passed, 0 failed, 3 ignored**
in each run. Tests include WARP execution of the fetch shader, ticket wrap and
slot arithmetic, and preserving a title byte sent after an earlier flight.
Workspace all-feature Clippy with `-D warnings` passes. The release build passes;
its executable SHA-256 is
`97D7D978DAA79DD04128D264131FBB99BD32C503C90080D2620521267BB20779`.

The feature remains **off by default**. Enable it explicitly for a comparison:

```powershell
$env:VICTORY_CARD_FETCH_INFLIGHT = '1'
```

The default ring holds 1,024 slots (approximately 64 MiB). Unsupported allocation,
overflow, missing comparison results, and the reserved ticket sentinel use the
existing fetch path. The runtime log identifies activation and fallback counts.
Passing tests does not establish a title frame-rate improvement.

## Rendering and timing before this experiment

The material/shader build at `2294a5e` reached the prison-window view with
clothing, detailed cell walls/posters, and the wire window mesh visible. These
were absent in the preceding local build. Rendering still has defects; this
observation is not a proof that window flicker is fixed.

Seconds 480–540 of that run contained 12 caption samples: median 255.6 ms
(3.91 fps), about 4,450–4,620 draws, and 18.85 GiB private memory. The sampled
95th percentile was 3,203 ms, including a substantial stall. This is a sampled
caption statistic, not a per-frame percentile. Different scene progression,
shader work, cache state, and overlapping builds/tests limit comparisons with
the earlier 5 fps build. **Stable 30 fps is not achieved.**

The later texture-prelude build at `766a71a` also reaches that textured view.
Its bounded runtime/input evaluation and the in-flight-fetch comparison are
tracked separately from the unit-test results above.

## Launcher

`scripts/Launch-PerformanceProfile.ps1` takes explicit local game, configuration
and DXC paths. In a binary package place it beside `victory.exe`; from the source
checkout pass `-Executable` explicitly. `-CheckOnly` validates paths and reports
the executable hash without launching. `-InflightFetch` enables this experiment.
The launcher sends no scripted keys and stops its owned process at the time
limit or below 2 GiB available commit. The default time limit is 1,800 seconds.

## Deferred integration

#462's constant-buffer fill on the card needs correct buffer-state transitions
when a source was already written as a UAV in the same list. #583's proposed
per-list state tracker assumes COMMON at each list start, but D3D12 buffer decay
is at the ExecuteCommandLists boundary, not between lists in one call. Its test
uses separate submissions and does not cover that case. Neither patch is merged
here. A future integration should handle multi-list state chaining or restore
states at actual list boundaries, then test the combined copy path and its cost.

No title files, derived shaders, captures, saves or raw runtime logs are included.
