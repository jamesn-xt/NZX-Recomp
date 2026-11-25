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

The bounded comparison has now completed. Quiet window-view samples from seconds
420–500 were 227.0 ms / 4.41 fps with the texture-prelude build (16 samples), and
210.9 ms / 4.74 fps with in-flight fetch enabled (15 samples). Sampled p95 values
were 246.9 and 225.5 ms. After backward movement and camera input, seconds
580–620 were 258.8 ms / 3.86 fps and 226.1 ms / 4.42 fps respectively. The final
views differ: about 5,960 draws in the former versus 5,400 in the latter. Cache,
view and code differences prevent attributing an exact percentage to the switch.
Background compilation/testing began after the selected comparison windows;
later samples include that load and should not be used for this comparison.

The enabled path used 1,024 slots and reported no separate fetch waits or slot
overflow in the observed steady lines. Both runs showed clothing, cell-wall and
floor detail, bunks, a cellmate, and the wire window mesh. Both ended deliberately
(replacement or the configured time limit), without observed device loss. This
is limited scene/input validation, not complete playability or a 30 fps result.

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

## Next profiling probe

Branch `test/rtx4080-heap-timing` splits the existing slot-pass time into planning
(including bookkeeping locks and deciding which slots are due) and mirroring
(resolving the selected views and their host copies). These are subsets of the
existing slot time, not additional frame costs. In the texture-prelude run the
whole slot stage sometimes took around 88 ms/frame; this split will show which
part deserves optimization. It changes timing/logging only. Workspace Clippy
and release compilation pass; a runtime measurement of the split is pending.

## Following CPU and descriptor corrections

`ed29794` merges upstream `9376fd6`: recent draw-table slots are left to their
draws during descriptor synchronization, and known guarded view memory avoids
failing GetWriteWatch calls. Page-batched locking is retained, using upstream's
multiply-and-fold record print. A new regression verifies that deferring a table
does not consume its record fingerprint, so expiry still resynchronizes it.
This merged source passed the complete serialized all-feature workspace suite:
**589 passed, 0 failed, 10 ignored**, plus workspace Clippy with `-D warnings`.

The next branch, `test/rtx4080-captured-descriptors` at `494f51a`, adapts Yusuf
Al-Khawaldeh's #792. The old path fingerprinted captured record A but reread live
record B when installing the descriptor; A returning later was then skipped.
That installed-key regression failed locally before the fix. The batched path
now retains each changed record's immutable 12-byte header and publishes that
same header, while ordinary API mirroring keeps its existing live-read path.

The two focused sampler regressions pass on the RTX 4080 and WARP, including a
fenced shader-output check with independent clamp/wrap/decoy values. The complete
GPU suite passes **185 tests, 0 failed, 3 ignored**; workspace Clippy and the
diagnostics release build pass. This is additional GPU validation, not a claim
that the full workspace suite was rerun on this child revision. The executable
SHA-256 is `A6A3DE82020241AD3D39C221C5CA60B047C90D0D9FC0096B972AF2404D71BCF0`.
Its bounded title run is in progress. These tests do not establish that a
particular title draw reaches the sampler race or that all flicker is fixed.
