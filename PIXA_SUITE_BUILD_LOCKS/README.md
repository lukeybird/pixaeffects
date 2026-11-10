# Verified Linux dependency snapshots

These Cargo.lock snapshots belong to the prepared Pixa sources and shared runtime at commit `cc3befe9a83928a35c2b18e3715b797a6df1ac87` in `lukeybird/pixaeffects`. They preserve the resolved Linux dependency versions; upstream source pins alone do not freeze newly introduced dependency resolution.

For reproduction, use the overlay/runtime files from that source commit together with this verification record. The verification commit adds records without changing the `pixa-suite` source tree. Prepare the selected product with `pixa-suite/prepare.py`, then copy its `<product-id>.Cargo.lock` snapshot into the prepared source folder as `Cargo.lock`. Run the corresponding native Cargo command with `--locked` and the catalogued Rust toolchain. Keep the source overlay, runtime and lock snapshot together. The verification report records snapshot hashes and the exact build scope.

The snapshots describe Linux development verification, not signed installers, a release feed, or complete graphical/platform acceptance. They contain no publisher signing identity or account credentials. Build outputs remain unpublished. See `PIXA_SUITE_VERIFICATION.json` for the final result and source identity.
