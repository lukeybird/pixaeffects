# Pixa runtime

Shared, UI-independent development code for account connection, offline guide data and update verification. No publisher private key is included. The deterministic key in update tests is a fixture, never a signing identity.

## Account behavior

- Uses the existing first-party PixaShop device API; no billing endpoint or subscription configuration changes.
- Network calls happen on a worker after a person's account action, never during tests or editor startup.
- Tokens redact Debug output and zero their owned strings on drop. Sign-in can persist through OS Keychain, Credential Manager or Secret Service; unavailable storage falls back to RAM only, with a visible warning.
- Leases use monotonic time and last at most five minutes.
- A stale response from a disconnected account cannot restore its access state.
- TLS uses the operating system's certificate verifier; redirects are disabled and responses are bounded.

Default development builds permit editing. Experimental subscription builds are supported for all fourteen prepared editors: the wrapper gates editor callbacks, disables local control servers and offers native recovery while locked. Interface detached windows independently check the same monotonic deadline. Sound stops playback/recording when access locks. Anything supports credential-free draft snapshots and restoration into a fresh idle session. Locked close always requires an explicit decision. This is not verified production protection: live account flows and interactive platform QA remain outstanding.

One bounded worker serializes network and credential operations. Restored credentials require fresh server verification. Logout completes only after credential deletion acknowledges; stale operations cannot restore the disconnected lease. An already-started secure write is followed by deletion. Tests use mock stores; interactive OS vault behavior is not verified here.

## Update behavior

`updates::verify` validates an Ed25519 signature over the exact payload bytes before parsing a manifest. It checks product, platform, architecture, bundle ID, expiry, stable version precedence and publisher asset location. `verify_download` streams a bounded installer and checks its exact size and SHA-256.

The desktop wrappers now use `release_feed::latest` to discover stable, product-scoped releases in `lukeybird/pixaeffects`: `<product>-v<version>`, with an exact own-publisher URL, platform filename and SHA-256 asset digest. A different product, draft, prerelease or missing digest is rejected. No upstream feed is enabled. No matching suite releases are published yet.

On Mac, the native installer template independently verifies the digest, Developer ID team, bundle ID, hardened runtime, version and Gatekeeper before preparing a replacement. Install requires save acknowledgement and normal editor exit; canceling close disarms it. The helper restores the prior app if replacement or the immediate launch command fails. Windows and Linux open the product download page for manual installation; automatic install/restart is not implemented there.

The separate Ed25519 manifest validator remains available for a future signed-manifest publisher. It is not the feed used by this GitHub/Apple-signature path, and no publisher Ed25519 key has been configured. Interactive installation, canceled-close handling and update acceptance remain unverified. Source implementation does not authorize publishing.

## Tests

```
cargo +1.95.0 test --manifest-path pixa-suite/runtime/Cargo.toml
cargo +1.95.0 clippy --manifest-path pixa-suite/runtime/Cargo.toml --all-targets -- -D warnings
```

Tests use local fixtures only. They do not create production device records, use billing credentials, or contact the account service.
