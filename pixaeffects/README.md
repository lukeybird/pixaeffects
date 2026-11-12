## Yellow-stage audit candidate — 0.1.3

This revision removes obsolete translated website promotion and fixes updater status handling: available releases no longer claim the app is current, failed checks clear stale offers, installation errors remain visible, and rechecking is disabled during a Mac installation. The manual explains how to open the offline guide/tour and how update failures recover. Native UI regressions cover manual search, all 31 tour steps, Back/Finish/Escape, and visible checksum errors. A separate shell harness exercises replacement/launch rollback with fake platform tools; it is not Apple verification.

The successful 0.1.2 CI packages are the previous source. This revision needs a new build before packaged acceptance. Source tests do not complete installed branding, real account approval, or signed Mac update acceptance. Those stages must remain pending until their actual results are recorded. No signing, publishing, billing, website deployment, download enablement or release settings are changed by this source revision.

## Current free-access candidate — 0.1.2

The production Pixa account service now grants free early access without a card or scheduled account expiration. This candidate updates the activation-screen wording and bundled manual to match. Sign-in, explicit device approval, short monotonic security leases and existing access guards remain in place. No billing endpoint, paid-service subscription, published installer or release setting is changed here. Build artifacts remain unsigned testing candidates until separate platform acceptance and signing. Historical release notes below describe earlier candidates and may refer to the former paid trial.

# PixaEffects by AIWebDesignFirm

Motion graphics and compositing, with publisher branding, free Pixa account access, a bundled manual, a 31-tool tour, and a publisher-verified Mac updater.

Source base: EffectCraft 4ed549588ef2ad43a3e445404295bf36003060c0.
Product branch: pixaeffects-product. Current product candidate: 0.1.3.

On a CLEAN pinned checkout run `python pixaeffects/prepare.py`, then build Cargo target `effectcraft`. The overlay retains internal crate names and .ecproj compatibility. It is not a relabeled upstream binary.

`python pixaeffects/package.py` produces testing installers. Mac installers include a drag-to-Applications layout. Sign and notarize with `bash pixaeffects/sign_update_macos.sh /actual/path/to/pixaeffects-0.1.3-macos-universal.dmg` on the publisher Mac. That script reuses the existing Developer ID identity and PixaShop-Notary Keychain profile, but validates the PixaEffects bundle identity. No private key belongs in Git or app packages.

Free Pixa account access is shared, with separate device token storage. No new prices, subscriptions or billing mutation are introduced. Desktop control-server access is disabled. The unbundled upstream CLI and browser builds are not commercial distribution artifacts.

Publish only after native build, activation, render/export, signature/notarization and real updater checks. Only Mac supports in-app installation. Windows and Linux use website downloads. CI signing credentials are not configured.

Required upstream licenses and contributor credits remain intact. In-app repository/Discord promotion is removed. Public fork source remains public; hiding links does not make it private.

## Branding audit candidate 0.1.1

See root PIXA_EFFECTS_BRANDING_AUDIT.json. Product preparation now removes dormant community/source-promotion commands, updates account controls, filters obsolete translated promotions, cleans Linux metadata/publisher branding and uses a PixaEffects demo title. Nine Python checks pass; full native and installed-app acceptance remains pending. Existing credits/licenses and original E icon are preserved. Build this revision before signing; do not relabel the previous 0.1.0 artifact.
