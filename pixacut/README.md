# PixaCut source handoff

PixaCut is a modified GPL-3.0-or-later edition of Nuzky. Its complete source is reproducible from the pinned upstream revision in SOURCE.json and branding.patch. The patch contains the UI/native branding, original C icons, quick guide, separate app/storage/credential/IPC/MCP identities, retained upstream notices and disabled upstream updater/support submission.

## Reconstruct

From this repository checkout, run:

```
python3 pixacut/prepare.py
```

The complete app will be at `pixacut/sources/PixaCut`. You can specify `--destination /a/new/path`. The script refuses an existing destination, checks patch integrity, fetches the exact upstream revision, applies the source and icon changes, then verifies every changed file against SOURCE.json. It never starts the app, installs dependencies, modifies existing projects, or uses signing credentials. Build inside that reconstructed folder, not at the EffectCraft repository root.

Read the root `CURSOR_PIXACUT_HANDOFF.txt` before native work. Do not run updater publication or inherited upstream release workflows blindly. This branch contains source preparation, not a customer installer. Existing app releases, billing and live website are unchanged.

## Status

Frontend lint, TypeScript and frontend production build passed during prototype preparation. Native compilation, real editor/save/import/export/captions acceptance, packaging, updater feed and Pixa account integration are pending. No Mac app has been built or notarized for PixaCut. The prepared source itself includes PIXACUT_STATUS.json, origin/modification records and a native branding E2E scenario that has not been executed.

## License

Retain LICENSE and upstream notices. PixaCut is modified from `nuzkyapp/nuzky` commit `13dec1d70da91b5506ccceec3e05b275de708314`, not endorsed by the upstream developers. This separate subproject does not change the licenses of the other applications. GPL source and redistribution obligations apply when distributing PixaCut. Source publication here does not mean the program is release-ready or that any proprietary/commercial relicensing has been obtained.
