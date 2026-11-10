# Pixa suite — unpublished development sources

This directory prepares 14 additional Pixa-branded editors from exact pinned upstream commits. PixaShop and PixaEffects remain in their existing product branches. No website, billing, subscription, price, or production configuration changes are included. There is no launcher yet. The separate ArtCraft Studio AI application is assessed in the catalog but is not prepared by this native-editor pipeline: its hosted backend and account integrations require separate implementation.

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

Requires Git, Python 3.11+, Pillow, Rust 1.95 (1.97 for PixaAnything) and each upstream app's native build prerequisites. macOS and Windows binaries must be built on the corresponding platform/toolchain; this Linux workspace cannot verify a Mac installer.

```
python -m pip install -r requirements.txt
python prepare.py
python check.py
python -m unittest discover -s . -p 'test_*.py'
python build.py pixavector --release
python package.py pixavector
```

Use `python prepare.py pixavector` for one product. Preparation refuses dirty, already prepared, or wrongly pinned checkouts. Generated source trees stay out of git. To regenerate, first preserve any work you intend to keep and use a separate clean `--sources` directory. Do not silently reset another person's edits.

## What is implemented

- Exact upstream pins; original engine/crate/file-format IDs preserved.
- Pixa application display names and separate bundle IDs.
- Original matching pixel icons, generated locally with attribution.
- Desktop wrapper with publisher footer, account connection window, searchable product-specific starter manual and read-only quick orientation.
- Shared Rust account state and signed update/download validation library, with offline tests.
- Hash-bound build receipts include shared runtime source, so changing the SDK invalidates old receipts.
- Hosted verification workflow calls the existing shared Craft gate for all 14 products on Linux, Windows and Intel macOS; it does not create or publish installers.
- Preservation of editor lifecycle callbacks, saving and shutdown hooks.
- Own product update identity and a refusal source that blocks upstream update checks.
- Repeatable checks, native build commands, hash-bound build receipts and local development packaging.
- Original licenses, copyright and contributor records retained.

## What is NOT finished

These are development preparations, not 14 verified shipping applications. Subscription enforcement, per-app device/backend integration, comprehensive Pixa manuals/tool tours, complete promotional-brand audits, per-platform native builds, real editor/import/export/save QA, and verified signed Pixa updates remain work. The starter manuals and four-step orientations are not complete manuals or tours of every tool. Account connectivity is integrated, but editing enforcement, secure persistent sign-in and save-safe expiry handling remain incomplete. Signed-update validation is implemented, but a feed, publisher verification key and native install/restart integration remain unset. Upstream alpha features are not promised as professional-product parity.

Development packaging deliberately says it is unsigned and unnotarized. No secrets are needed or stored. Do not upload these packages to customer downloads, enable a release flag, publish GitHub releases, or claim completion until the gates pass. Mac signing and notarization happen later using the publisher's secure Keychain or an explicitly configured secure signing service.

## Future launcher contract

Keep each editor independently installable. A future launcher should consume a separately signed release manifest containing platform/architecture, immutable asset URL, size, SHA-256, bundle ID, exact version, signing identity, and minimum OS. Never treat a catalog entry or a mutable "latest" URL as proof that an installer is safe. The launcher must validate downloads and preserve unsaved work before restart. Catalog entries remain unpublished until individually verified.

## Hosted platform checks

The `Pixa suite native checks` workflow verifies each product on Linux, Windows and Intel macOS through `storytold/craft-common-actions` (contents-read permission only). A successful run means compilation checks, not linked installers, editor runtime QA, signing or notarization. No unsigned customer downloads are uploaded. Inspect the actual workflow run before claiming a platform passed.
