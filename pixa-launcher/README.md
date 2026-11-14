# Pixa Launcher 0.2.0

Native Rust/egui desktop library for all sixteen Pixa editors. The Mac installer and release-discovery implementation now exist. This source is not an Apple-signed launcher release; the website launcher download must remain unavailable until the universal Mac candidate passes real-Mac acceptance and notarization.

## What it does

- Browse sixteen apps with original Pixa product icons, category, search and installed-only filter.
- Show Install, Update, Reinstall and Open, including installed and available versions.
- Confirm each install; stream the download with progress/cancel and an exact size/SHA-256 check.
- Check current stable GitHub releases on demand. Only the publisher repositories, exact product-prefixed tags (Shop uses vVERSION), matching universal DMG asset names and API SHA-256 digests are accepted. A missing digest or older release is never installed as an update.
- On Mac, mount read-only, verify the app's exact bundle ID/version, universal Mach-O architectures, Apple signing chain, publisher Team 945M5X4DY3, and Gatekeeper verdict. Verify the staged copy again before replacement.
- Install without elevation into ~/Applications. Existing /Applications apps can be opened; the launcher does not overwrite system-wide apps or request an administrator password.
- Refuse replacement of a running app, a newer local version, a symlink destination or an untrusted existing app. Serialize installs across launcher processes with a filesystem lock.
- Preserve a previous app in a hidden .pixa-previous-* folder. A failed file promotion restores it. Never remove documents, app data, or accounts.
- Open account settings in the browser. Editors retain their own activation screens and account checks; the launcher does not create a second login/billing system or bypass editor access.
- Provide an offline Help window with installation, updates, cancellation, account and recovery guidance.

## Build and test

Requires Rust stable with rustfmt/clippy and platform build tools.

```
cargo test --locked --manifest-path pixa-launcher/Cargo.toml
cargo clippy --locked --manifest-path pixa-launcher/Cargo.toml --all-targets -- -D warnings
cargo run --locked --manifest-path pixa-launcher/Cargo.toml
cargo run --locked --manifest-path pixa-launcher/Cargo.toml -- --check-releases
```

Linux/Windows builds are catalog previews: native install/Open is Mac only. They are not advertised as Windows/Linux launcher installers.

## Universal Mac candidate

On a Mac with Rust, the Xcode command-line tools and sufficient free disk:

```
python3 pixa-launcher/package_macos.py --out "$HOME/Downloads/PixaLauncher-0.2.0-candidate"
```

This builds both aarch64-apple-darwin and x86_64-apple-darwin with Cargo.lock, combines them, generates the original launcher icon and bundle metadata, includes notices, ad-hoc signs the local candidate, verifies its structure and records provenance in CANDIDATE.json. It refuses an existing output folder. It never submits to Apple or exports a key. You can provide already-built --arm64 and --intel binaries to avoid rebuilding; both are required.

The candidate has an ad-hoc signature, not a Developer ID identity. Do not publish it to customers or bypass Gatekeeper settings to call it approved.

## Mac acceptance before signing

Read ../CURSOR_PIXA_LAUNCHER_HANDOFF.txt. Exercise the actual candidate on a Mac:

1. Library, icons, search, installed-only, Help and resize at minimum/default sizes.
2. Live release discovery and offline/error fallback; independent versions and correct product pages.
3. Install one small app and one larger app; observe real hash, mount, app signature, exact bundle/version, universal architecture and Gatekeeper checks. Verify the launcher refuses deliberately corrupt/foreign local fixtures in the unit tests.
4. Cancel a download, retry, and close while busy. No partial app may appear.
5. Open a correctly signed existing app. A running editor must block replacement. Save a document, quit, reinstall/update, then reopen and confirm the document remains intact.
6. Verify another launcher instance cannot install concurrently. Previous app is preserved; restore it using the Help instructions if needed.
7. Activation/account settings remain in the editor/browser. Test one real account without printing credentials.
8. Retest installation after Apple signing/notarization before public distribution.

## Release trust and updates

The catalog shipped in the launcher is a fixed versioned fallback. Live discovery uses HTTPS and GitHub's published asset digests; it is **not** an Ed25519-signed feed. No discovered executable is trusted solely because GitHub returned it: the native install gate independently requires the exact Pixa bundle identity and publisher's Apple signature/Gatekeeper acceptance.

The previous strict schema-2 Ed25519 manifest validator remains as a separate library API with expiry/sequence protections and tests. It is not presented as a configured production feed: no production Ed25519 key, rotation, or persistent signed-feed sequence store is installed. Do not confuse that unused API with live GitHub discovery.

Check for app updates discovers editor releases only. It does not update/restart the launcher itself. Launcher self-update and automatic silent installs are deliberately absent; obtain future signed launcher versions from the website. Updates of editors are explicit and require quitting the editor.

## Platform tools

End-user Macs use /usr/bin/hdiutil, /usr/bin/ditto, /usr/bin/codesign, /usr/sbin/spctl, /usr/bin/sw_vers, /bin/ps and /usr/bin/open. The launcher reads the Mach-O universal header itself; customers do not need Rust, Xcode or lipo. Only the publisher's packaging script uses lipo/iconutil and Rust.

No website, billing, editor binary, signing identity or published download is changed by building the launcher. No credentials belong in source or command arguments.

## Finished Mac candidate and signing helper

The universal Mac candidate completed in Actions run 43571955021 at source 93a3cfe5ce4eb2338ea8e638c3e21ace52c5a95a. See ../PIXA_LAUNCHER_MAC_BUILD.json and the updated Cursor handoff for the exact artifact; it is unsigned for public distribution. Reuse it rather than repeating both builds.

After actual Mac acceptance, copy ACCEPTANCE_TEMPLATE.json into a separate result file, record the candidate source/executable hash and observed pass results, then run finish_macos.py with the app, --receipt, --acceptance, --profile and a fresh --output folder. Add --publish only to upload the final notarized DMG and report. The helper never enables the website automatically, exports a key, or accepts incomplete/different-build test results. It uses the existing publisher Mac Keychain and shared release-tools/macos_candidate.py. Its receipt/acceptance guards have Linux unit tests; actual Apple signing still runs only on the publisher Mac.
