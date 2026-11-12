# Pixa launcher

The launcher is not a finished product. This directory is the catalog and download gate that has to exist before a desktop shell can install editors.

## Stack

The validation core is a small Rust library, the same language as the editors. It has no GUI yet. A window that lists the sixteen apps comes after one editor can be installed from a signed manifest. The editors stay independently installable.

## Trust

A manifest entry is installable only when all of these hold:

- The product id is one of the sixteen explicit editor IDs, and its bundle ID matches that exact product.
- The version is plain numeric semver, with no prerelease tag.
- Artifact URLs match the exact product/version/architecture contract. Shop uses `lukeybird/pixashop` with `vVERSION`; Effects uses `lukeybird/pixaeffects` with `vVERSION`; other editors use product-prefixed tags in `lukeybird/pixaeffects`. Only universal Mac assets are currently accepted.
- The URL path does not contain `.` or `..` segments.
- Size is present, greater than zero, and at most 8 GiB.
- SHA-256 is 64 hex characters.
- The manifest body is exactly the bytes covered by a strict Ed25519 signature from the caller-supplied trusted verifying key, including whitespace. No production key is pinned yet.
- Schema 2 requires positive `sequence` and Unix-second `expires_at`; entries must be unexpired and no more than 31 days from the trusted current time. The caller supplies trusted current time and a persisted minimum sequence, not values from the received entry.
- Unknown/duplicate fields, bodies over 64 KiB, a foreign publisher and mismatched product/bundle/architecture are refused. Publisher identity is fixed to Team 945M5X4DY3.

API: `accept_manifest(body, signature_hex, verifying_key, now_unix_seconds, persisted_minimum_sequence)`. The installer must persist rollback protection and enforce minimum OS and installed-version ordering; the returned metadata is not installation approval.

The signing private key stays off this repository, off GitHub Actions artifacts, and off the website. Key rotation means shipping a new launcher build whose pinned verifying key accepts the next manifest key, then rejecting the old key after that launcher is itself installed. Expired or unsigned manifests are refused by the core. Rotation is a design requirement; an operational rotation/expiry state store is not implemented. There is no “latest” URL.

## What this does not do yet

It does not download, mount, or launch an editor. It does not talk to the account service. It does not replace the website.

## Current verification

Five Rust tests pass, including signed wrong-bundle/publisher/architecture rejection, expiry/replay, duplicate fields, tampering and bounded manifest input. The wrong-bundle regression failed against the previous core. Optimized Linux library compilation passes; this is not a runnable launcher or an installer. GUI, actual download/install/open/uninstall/rollback, pinned production key, account integration, OS builds and native acceptance remain pending. Do not mark the launcher ready for notarization.
