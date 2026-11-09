# Pixa suite — unpublished development sources

This directory prepares 14 additional Pixa-branded editors from exact pinned upstream commits. PixaShop and PixaEffects remain in their existing product branches. No website, billing, subscription, price, or production configuration changes are included. There is no launcher yet.

## Products

| Upstream | Pixa product |
|---|---|
| VectorCraft | PixaVector |
| FilmCraft | PixaFilm |
| LightCraft | PixaLight |
| PdfCraft | PixaPDF |
| DesignCraft | PixaDesign |
| FigCraft | PixaInterface |
| TextureCraft | PixaTexture |
| MaterialCraft | PixaMaterial |
| WordCraft | PixaWord |
| SoundCraft | PixaSound |
| DeckCraft | PixaDeck |
| CADCraft | PixaCAD |
| GridCraft | PixaGrid |
| Craft Anything | PixaAnything |

`catalog.json` is a versioned product catalog for future launcher integration. It records exact source revisions, product names, bundle IDs, versions, accents and honest integration status. It is not a published update feed.

## Prepare and build

Requires Git, Python 3.11+, Pillow, Rust 1.95 and each upstream app's native build prerequisites. macOS and Windows binaries must be built on the corresponding platform/toolchain; this Linux workspace cannot verify a Mac installer.

```
python -m pip install -r requirements.txt
python prepare.py
python check.py
python build.py pixavector --release
python package.py pixavector
```

Use `python prepare.py pixavector` for one product. Preparation refuses dirty, already prepared, or wrongly pinned checkouts. Generated source trees stay out of git. To regenerate, first preserve any work you intend to keep and use a separate clean `--sources` directory. Do not silently reset another person's edits.

## What is implemented

- Exact upstream pins; original engine/crate/file-format IDs preserved.
- Pixa application display names and separate bundle IDs.
- Original matching pixel icons, generated locally with attribution.
- Desktop wrapper with publisher footer and a small getting-started window.
- Preservation of editor lifecycle callbacks, saving and shutdown hooks.
- Own product update identity and a refusal source that blocks upstream update checks.
- Repeatable checks, native build commands, hash-bound build receipts and local development packaging.
- Original licenses, copyright and contributor records retained.

## What is NOT finished

These are development preparations, not 14 verified shipping applications. Subscription enforcement, per-app device/backend integration, comprehensive Pixa manuals/tool tours, complete promotional-brand audits, per-platform native builds, real editor/import/export/save QA, and verified signed Pixa updates remain work. The getting-started window is not a complete manual or tool tutorial. Upstream alpha features are not promised as professional-product parity.

Development packaging deliberately says it is unsigned and unnotarized. No secrets are needed or stored. Do not upload these packages to customer downloads, enable a release flag, publish GitHub releases, or claim completion until the gates pass. Mac signing and notarization happen later using the publisher's secure Keychain or an explicitly configured secure signing service.

## Future launcher contract

Keep each editor independently installable. A future launcher should consume a separately signed release manifest containing platform/architecture, immutable asset URL, size, SHA-256, bundle ID, exact version, signing identity, and minimum OS. Never treat a catalog entry or a mutable "latest" URL as proof that an installer is safe. The launcher must validate downloads and preserve unsaved work before restart. Catalog entries remain unpublished until individually verified.
