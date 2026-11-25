# NZX Recomp development

25 November 2026. Initial donor: DexNom/VIctory at
`921d8d3b43b44f3a6ca4b44095edd3c4baf10a87`.

Stable 30 fps and correct visuals are the development target, not an achieved
result. The current hardware target is NVIDIA RTX 4080, Ryzen 5 5600, 32 GB RAM.

## Source and build boundaries

The complete donor renderer is the starting point. Internal crate names,
configuration keys and `VICTORY_*` environment variables are retained. Public
launchers, program titles and the README use NZX Recomp. The donor's Git
history, license and credits remain intact. Its README is retained as a
historical reference in `docs/README.dexnom.md`.

Existing jamesn-xt/VIctory binaries and the unfinished clean-watch cherry-pick
are preserved separately. Fixes that affect staging lifetime, descriptor
publication or frame time must be adapted to this source and tested.

## Validation status

The unmodified donor built successfully on the local Windows/MSVC toolchain.
Its first RTX 4080 runtime comparison is in progress using a copy of local
save data. No outcome or stable gameplay frame rate is claimed yet.

Measurements must name the executable revision, console profile, cache state,
scene, test duration and any overlapping work. Menu/loading 30 fps does not
count. Source compilation and GPU test activity are excluded from selected
runtime intervals. Per-frame statistics and sampled window-caption statistics
must be distinguished.

Radeon measurements in `BENCHMARK.md` belong to the donor project and cannot
be reused as NVIDIA results. Test counts from the older VIctory integration
likewise do not validate this new project's full source tree.
