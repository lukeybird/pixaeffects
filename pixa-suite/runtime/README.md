# Pixa runtime

Shared, UI-independent development code for account connection, offline guide data and update verification. No publisher private key is included. The deterministic key in update tests is a fixture, never a signing identity.

## Account behavior

- Uses the existing first-party PixaShop device API; no billing endpoint or subscription configuration changes.
- Network calls happen on a worker after a person's account action, never during tests or editor startup.
- Tokens redact Debug output and zero their owned strings on drop. Sign-in can persist through OS Keychain, Credential Manager or Secret Service; unavailable storage falls back to RAM only, with a visible warning.
- Leases use monotonic time and last at most five minutes.
- A stale response from a disconnected account cannot restore its access state.
- TLS uses the operating system's certificate verifier; redirects are disabled and responses are bounded.

Default development builds permit editing. Experimental subscription builds are permitted only for PixaFilm, PixaTexture, PixaDesign and PixaDeck: their wrapper gates editor callbacks, disables the control server, and offers native Save a Copy recovery while locked. Other ten editors remain blocked from restricted builds until their recovery adapters exist. Locked close always requires an explicit decision. This is not verified production protection: live account flows and interactive platform QA remain outstanding.

One bounded worker serializes network and credential operations. Restored credentials require fresh server verification. Logout completes only after credential deletion acknowledges; stale operations cannot restore the disconnected lease. An already-started secure write is followed by deletion. Tests use mock stores; interactive OS vault behavior is not verified here.

## Update behavior

`updates::verify` validates an Ed25519 signature over the exact payload bytes before parsing a manifest. It checks product, platform, architecture, bundle ID, expiry, stable version precedence and publisher asset location. `verify_download` streams a bounded installer and checks its exact size and SHA-256.

This is the validation layer, not an installer. No release feed or publisher verification key is configured. Native installer signature/notarization checks, save-safe restart/rollback and real install tests still have to be connected before enabling update delivery. Never fall back to an unsigned feed.

## Tests

```
cargo +1.95.0 test --manifest-path pixa-suite/runtime/Cargo.toml
cargo +1.95.0 clippy --manifest-path pixa-suite/runtime/Cargo.toml --all-targets -- -D warnings
```

Tests use local fixtures only. They do not create production device records, use billing credentials, or contact the account service.
