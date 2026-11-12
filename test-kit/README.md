# Pixa editor test kit

This kit covers all sixteen editors. It separates source testing from actual installed-app testing and does not enable public downloads or submit anything to Apple.

1. Refresh the correct product branch and read `CURSOR_PROJECT_STATUS.txt`, `PIXA_BRANDING_HELP_CHECKPOINT.json` and `test-kit/catalog.json`.
2. Obtain the matching candidate for the intended OS/architecture. Verify version, exact source provenance and artifact SHA-256. A source checklist is not an installer. Never relabel an old binary with a newer version.
3. Copy the product's JSON checklist outside the repository as your working acceptance form. Record pass/fail/blocked/not-run and evidence for each item; leave unknowns pending. Do not record account tokens, passwords or personal project contents.
4. Follow its branding, offline manual, tool-tour, read-only behavior, layout and core editing tests. Use disposable projects and retain original media.
5. Report unsupported/missing tools or explanations before claiming full coverage. Seven suite guides currently cover source-anchored control groups rather than an exhaustively verified command inventory.
6. Keep signing/notarization, post-notarization Gatekeeper checks, real signed updates and public publication as separate gates. On Mac, follow the current handoff and the owner's stop-before-notarization boundary. Do not bypass Gatekeeper or remove quarantine to manufacture an acceptance pass.

## Automated suite checks

In a prepared suite checkout with the pinned Rust toolchains, Python and packaging dependencies:

```sh
python3 -m unittest discover -s pixa-suite -p 'test_*.py'
python3 pixa-suite/check.py
python3 pixa-suite/check_help_ui.py --sources pixa-suite/sources --output pixa-suite/build-logs/help-ui-test-run
```

Use a fresh output directory. The help harness tests the actual shared component against all fourteen generated guides; it does not test the full editor or each installed app's fonts/layout. Shop and Effects use their separate `pixashop/tests` and `pixaeffects/tests` on their product branches.

## Unsigned candidate build

For one prepared suite product, replacing the ID with an entry from catalog.json:

```sh
python3 pixa-suite/build.py pixalight --release --access-mode subscription
python3 pixa-suite/package.py pixalight --candidate
```

The existing build/packaging helpers enforce receipt/source/binary consistency. Candidate mode is account-gated, unsigned and unpublished. Host-only Mac packaging does not create a universal app. Use the documented universal Mac candidate workflow for that purpose; do not change shared concurrency or cancel existing owner jobs merely to rush a build.

The fourteen-suite catalog excludes Shop/Effects, which retain their own build workflows. The launcher is not one of these sixteen editors and remains a separate unfinished product.

No checklist starts with a passing result. Testing readiness must identify a real matching candidate, a supported test machine and the explicit features under test; it is not production readiness.
