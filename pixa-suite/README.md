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

These are development preparations, not 14 verified shipping applications. All fourteen have experimental restricted-preview gates and native recovery adapters with offline native roundtrip checks. Real account/expiry and OS vault UI tests, comprehensive manuals/tool tours, complete promotional-brand audits, linked per-platform builds and editor/import/export/save QA remain outstanding. PixaDesign has a 37-tool guide; the other thirteen have starter orientations. Signed-update validation exists, but the publisher key/feed, native installation, restart and rollback are not configured. Upstream alpha features are not promised as professional-product parity.

Development packaging deliberately says it is unsigned and unnotarized. No secrets are needed or stored. Do not upload these packages to customer downloads, enable a release flag, publish GitHub releases, or claim completion until the gates pass. Mac signing and notarization happen later using the publisher's secure Keychain or an explicitly configured secure signing service.

## Future launcher contract

Keep each editor independently installable. A future launcher should consume a separately signed release manifest containing platform/architecture, immutable asset URL, size, SHA-256, bundle ID, exact version, signing identity, and minimum OS. Never treat a catalog entry or a mutable "latest" URL as proof that an installer is safe. The launcher must validate downloads and preserve unsaved work before restart. Catalog entries remain unpublished until individually verified.

## Hosted platform checks

The `Pixa suite native checks` workflow verifies each product on Linux, Windows and Intel macOS through `storytold/craft-common-actions` (contents-read permission only). A successful run means compilation checks, not linked installers, editor runtime QA, signing or notarization. No unsigned customer downloads are uploaded. Inspect the actual workflow run before claiming a platform passed.

## Restricted previews

`python build.py pixafilm --release --access-mode subscription` selects an experimental restricted build; all fourteen catalog products support the experimental mode. Default builds are development mode. Unknown modes fail closed. Native recovery roundtrip examples cover saved-copy contents, original preservation and dirty state. Hosted CI runs development and restricted pilot compile modes plus recovery examples, and the shared runtime tests on all three operating systems. These are not release approvals.

The PixaDesign Help menu now opens an offline Manual and a read-only 37-tool tutorial. Tool groups are highlighted when visible, with a connector to the explanation bubble; the tour does not select tools or alter documents. Its actual GUI layout still needs native visual QA. Other products retain starter orientations, not complete per-tool tutorials. Display-name cleanup preserves internal command IDs and legal attribution. PixaDesign uses an original pixel-P mark instead of the inherited inline publisher outline. PixaLight uses the existing PixaShop bundle/executable names for its external-image-editor lookup; actual handoff still needs native testing.


## Recovery behavior

Every restricted preview preserves the live editor in memory when access expires and offers recovery to a newly created folder. Local automation/control servers are disabled in restricted builds. Recovery files never replace the original project files. Native roundtrip fixtures cover all fourteen adapters; these are engine/model checks, not complete editor UI tests.

- Vector, Film, PDF, Design, Interface, Texture, Material, Word, Sound, Deck, CAD and Grid write native project copies. Interface includes branch/version data; CAD includes pending block edits. Password-protected Word documents remain encrypted. Sound copies memory-only audio into its recovery package and stops recording/playback on the access transition.
- Light snapshots its current catalogue, presets, view and preferences. Original photos remain referenced in place; retain them.
- Anything snapshots drafts/chat and bounded attachment copies, excluding provider settings, provider keys and account credentials. Restore drafts requires a fresh idle session and refuses existing drafts. Keep original project folders; active jobs are not resumed.

Interface detached windows check the same monotonic access deadline even if the main window is hidden. Locked close still requires an explicit decision after recovery; unfinished text fields or external/background operations may require reconnecting first.

The final local verification checkpoint records 28 passed Linux compilation checks (all fourteen apps in development and subscription modes), 14 passed native recovery roundtrips, 15 Python regressions and 17 runtime tests. See FINAL_CHECKS.json and RECOVERY_STATUS.json. These results do not approve customer releases.
