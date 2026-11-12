PIXA FREE EARLY ACCESS — ACTIVE 2026-11-12

Owner explicitly selected everyone: no card, no subscription requirement, no scheduled expiration for Pixa app access. This is free early access, not a promise of lifetime commercial licensing.

CURRENT PRODUCTION
Backend: dpl_DSsgnFjuJGtsa2N8wmkjjtMKGajG
Marketing: dpl_FPJwJpHWs5wXfWHsjKkr4Rzsr6N8
Routing: 1b6189a2-b30f-4306-818d-495e32554536
Backend rollback: dpl_6w71EygorJmZkXWmq1vvTQG8GYLe
Marketing routing rollback: 6fe50bc7-ebe8-4729-8c11-65eb2d7ed354

The complete immutable production source tree (535 original files) was recovered through Vercel source metadata and referenced by exact file SHA. It includes the previously missing builder, Shop/device and billing work. DEPLOYMENT.json preserves that inventory. No stale GitHub main was used or promoted. This directory is a patch, NOT a complete production checkout.

The backend directory contains reviewed new source files and the narrowly scoped build-time transformation. Map access-route.ts -> app/api/pixa-access/route.ts; start-page.tsx -> app/start/page.tsx; account-page.tsx -> app/account/page.tsx. app/settings/page.tsx re-exports the account page. Map patch.mjs -> scripts/pixa-free-early-access.mjs. Build with node scripts/pixa-free-early-access.mjs && npm run build. Start from the original production source; do not apply twice. The transformation refuses an unexpected original handler.

App download authorization now uses signed-in Pixa free access. Device access checks grant a renewable 300-second lease to an existing approved account without querying Stripe. This short lease is a device security refresh, not a trial expiration. Device association, user existence, hashed tokens, origin checks, rate limits, approval and revocation remain in the original handler. Unapproved/unknown devices remain denied. Authentication, builder and Stripe billing endpoints are unchanged. Previous paid services are separately accessible from the account page; NO existing Stripe subscription was canceled or modified. Download enable flags, pinned versions and signing/release settings are unchanged.

New users: /start -> free account creation -> /account. Existing users: /start redirects to /account. /settings is the same Pixa account hub. TDG profile forms are no longer rendered at /account. Global legacy header may still show its existing navigation; Pixa hub content has no TDG links.

Verified:
- Production candidate compiled and type-checked successfully before manual promotion.
- Signed-out /start: 200, no card checkout; free-copy present.
- Signed-in /start: redirects to the new /account hub.
- Signed-in /api/pixa-access: hasAccess=true, expiresAt=null, cardRequired=false.
- Signed-out /api/pixa-access: 401 with hasAccess=false.
- Activation without valid Origin denied; valid-Origin activation without sign-in returns 401.
- Random unknown tokens at both Shop and Effects check endpoints return access=false/accessSeconds=0.
- Signed-in Shop page exposes existing Mac/Windows/Linux installer choices without a paid trial.
- node website-free-access/test-patch.mjs: existing users free, unknown users denied, auth/device guard preserved, builder/billing unchanged, unexpected source refuses mutation.
- Homepage browser screenshot confirms free early access/no card/no subscription/no scheduled expiration.

Not claimed: full new-account registration submission, approval of a new test device, native editor acceptance or native signing. Native binaries may retain older trial wording in their buttons until rebuilt; those links now lead to free account access.

FUTURE DEPLOYMENTS
Do not deploy stale aiwebdesignfirm/main or an incomplete local checkout. Preserve this immutable production baseline and the patch, or materialize the complete current production tree before integrating. Stage with custom-domain auto-assignment held, test, then explicitly promote. Never export a signing key or print production credentials.
