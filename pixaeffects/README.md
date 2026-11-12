# PixaEffects by AIWebDesignFirm

Motion graphics and compositing, with publisher branding, existing AIWebDesignFirm trial entitlement, a bundled manual, a 31-tool tour, and a publisher-verified Mac updater.

Source base: EffectCraft 4ed549588ef2ad43a3e445404295bf36003060c0.
Product branch: pixaeffects-product. Product version: 0.1.0.

On a CLEAN pinned checkout run `python pixaeffects/prepare.py`, then build Cargo target `effectcraft`. The overlay retains internal crate names and .ecproj compatibility. It is not a relabeled upstream binary.

`python pixaeffects/package.py` produces testing installers. Mac installers include a drag-to-Applications layout. Sign and notarize with `bash pixaeffects/sign_update_macos.sh /actual/path/to/pixaeffects-0.1.0-macos-universal.dmg` on the publisher Mac. That script reuses the existing Developer ID identity and PixaShop-Notary Keychain profile, but validates the PixaEffects bundle identity. No private key belongs in Git or app packages.

The existing trial/subscription entitlement is shared, with separate device token storage. No new prices, subscriptions or billing mutation are introduced. Desktop control-server access is disabled. The unbundled upstream CLI and browser builds are not commercial distribution artifacts.

Publish only after native build, activation, render/export, signature/notarization and real updater checks. Only Mac supports in-app installation. Windows and Linux use website downloads. CI signing credentials are not configured.

Required upstream licenses and contributor credits remain intact. In-app repository/Discord promotion is removed. Public fork source remains public; hiding links does not make it private.

## Branding audit candidate 0.1.1

See root PIXA_EFFECTS_BRANDING_AUDIT.json. Product preparation now removes dormant community/source-promotion commands, updates account controls, filters obsolete translated promotions, cleans Linux metadata/publisher branding and uses a PixaEffects demo title. Nine Python checks pass; full native and installed-app acceptance remains pending. Existing credits/licenses and original E icon are preserved. Build this revision before signing; do not relabel the previous 0.1.0 artifact.
