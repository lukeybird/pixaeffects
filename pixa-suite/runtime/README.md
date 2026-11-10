# Pixa runtime

Shared, UI-independent development code for account connection, offline guide data and update verification. No publisher private key is included. The deterministic key in update tests is a fixture, never a signing identity.

## Account behavior

- Uses the existing first-party PixaShop device API; no billing endpoint or subscription configuration changes.
- Network calls happen on a worker after a person's account action, never during tests or editor startup.
- Tokens stay in memory, redact Debug output and zero their owned strings on drop.
- Leases use monotonic time and last at most five minutes.
- A stale response from a disconnected account cannot restore its access state.
- TLS uses the operating system's certificate verifier; redirects are disabled and responses are bounded.

This provides an account **connection test** in the development apps. It does not enforce editing access. Safe enforcement still needs per-editor save/recovery adapters and automation gating. Persistent sign-in needs OS credential storage. Do not publish the development apps as subscription-protected products.

## Update behavior

`updates::verify` validates an Ed25519 signature over the exact payload bytes before parsing a manifest. It checks product, platform, architecture, bundle ID, expiry, stable version precedence and publisher asset location. `verify_download` streams a bounded installer and checks its exact size and SHA-256.

This is the validation layer, not an installer. No release feed or publisher verification key is configured. Native installer signature/notarization checks, save-safe restart/rollback and real install tests still have to be connected before enabling update delivery. Never fall back to an unsigned feed.

## Tests

```
cargo +1.95.0 test --manifest-path pixa-suite/runtime/Cargo.toml
cargo +1.95.0 clippy --manifest-path pixa-suite/runtime/Cargo.toml --lib -- -D warnings
```

Tests use local fixtures only. They do not create production device records, use billing credentials, or contact the account service.
