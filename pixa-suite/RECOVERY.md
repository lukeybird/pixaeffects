# Native recovery verification

All entries are unpublished restricted-preview adapters. Each has an offline Rust example using the actual native model/codec, synthetic projects, and no account API calls. Passing an example is not GUI, import/export or platform installer approval.

| Product | Recovery output | Coverage and boundary |
|---|---|---|
| PixaVector | .vectorcraft | All open documents; native reopen model comparison. |
| PixaFilm | .fcproj | Native save-copy; original path and dirty state retained. |
| PixaLight | Library folder | Live catalogue, presets, view and preferences; original photo files stay in place. |
| PixaPDF | .pdf | All open documents; native save/check pipeline. Encrypted/signed PDF cases not separately exercised. |
| PixaDesign | .designcraft | All open documents; native reopen comparison. |
| PixaInterface | .figcraft | Main documents with current branch contents and version history. |
| PixaTexture | .tcraft | Ends the active stroke before native save-copy. |
| PixaMaterial | .mcraft | All packages; native reader may add an empty Resources folder, the only fixture normalization. |
| PixaWord | JSON or encrypted DOCX | Active and parked documents; existing password protects encrypted copies. |
| PixaSound | .scraft and audio | Includes in-memory samples; access lock stops recording/playback using native UI code. Real recording hardware not tested. |
| PixaDeck | .deckcraft | All open documents; native reopen comparison. |
| PixaCAD | DWG | All drawings including pending block edits. Refuses unsupported unsaved entity data rather than silently dropping it. |
| PixaGrid | Native JSON | All workbooks with formulas and native model preservation. |
| PixaAnything | .pixasession and attachments | Drafts/chat and attachment copies; excludes keys/settings. Existing project folders must be retained; jobs resume manually. |

Recovery is written to a newly created directory and original files are not replaced. Keep the editor open if recovery fails. A successful recovery still does not authorize closing without review: incomplete text fields, external media and background work can require reconnecting first.

PixaAnything restores only into a fresh idle session, refuses existing drafts, limits JSON to 16 MiB and attachments to 64 MiB each / 512 MiB total, and never resumes an AI job automatically. Attachments are independent copies; original project directories remain external references.

The shared runtime tests cover account lease expiry, stale network/vault work, logout ordering, signed update/download verification and detached-window permit deadlines. Real operating-system credential dialogs, account access/expiry screens, native device activation and updater installation remain separate release tests.

For each product, hosted checks compile development and subscription modes and execute the native recovery example on Linux, Windows and Intel macOS. Check the workflow commit and actual results before stating a platform passed. No test uploads unsigned customer installers or publishes a release.
