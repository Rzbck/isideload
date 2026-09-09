# HANDOFF — Apple Watch companion support in isideload

Date: **2026-09-09**

## Objective

Add generic support for embedded Apple Watch companion apps to `isideload`, so an IPA containing `Main.app/Watch/Companion.app` can be re-signed/provisioned coherently.

This is a separate tooling chantier from `Rzbck/ios-godot-lab` / Watch Sensor Lab. Do not modify the application repo from this fork unless explicitly requested.

## Repository / current branch

- repo: `Rzbck/isideload` (fork of `nab138/isideload`)
- local path: `E:\_Project\IOS APP\_Tools\iloader-watch\isideload`
- branch: `feat/watch-companion-support-20260909`
- upstream baseline used by iLoader 2.3.1: `3d42025ecac97a2548d5b88aefc8028307e369c1`
- latest code HEAD verified before this HANDOFF update: `7c8d2a3008890ac771597ef72092cc7204e93b0b`
- local worktree was verified clean at that exact SHA before this documentation update.
- after this HANDOFF commit, always re-fetch GitHub/Git before asserting current HEAD.

## Implemented Watch support so far

### Generic embedded Watch bundle support

At code SHA `04d25c73742d29c2b8936c6f95741f22640ef6da`:

- detect embedded `.app` bundles under `Watch/`;
- keep them as `watch_apps` in the bundle tree;
- rewrite Watch `CFBundleIdentifier` consistently with the re-signed iPhone app;
- rewrite `WKCompanionAppBundleIdentifier` to the rewritten main iPhone bundle ID;
- include Watch app bundles in App-ID registration/provisioning collection;
- persist rewritten `Info.plist` files recursively;
- include Watch bundles in isideload's own nested-bundle traversal;
- add a regression test for coordinated bundle-ID rewriting.

CI run `34366333338` at exact SHA `04d25c73742d29c2b8936c6f95741f22640ef6da`: **SUCCESS** on macOS, Ubuntu and Windows.

### Paired Watch discovery / provisioning attempt

At SHA `8923077eddfc6c70aa7225e2a06a744e1e0c3b0d`:

- enabled `idevice` companion proxy support;
- discovered paired Apple Watch from the connected iPhone;
- attempted registration of the paired Watch as a development device;
- routed Watch App ID/profile operations through `DeveloperDeviceType::Watchos`.

CI run `34374806678`: **SUCCESS** on all three OSes.

Physical result: failed before install with HTTP 404 on:

`https://developerservices2.apple.com/services/QH65B2/watchos/listDevices.action?...`

This proved companion discovery reached the Watch-registration path, but the QH65B2 URL routing was wrong.

### Corrected Apple Developer Services routing

At current code SHA `7c8d2a3008890ac771597ef72092cc7204e93b0b`:

- Watch operations use the `ios/` QH65B2 path;
- explicit Watch operations add `DTDK_Platform = watchos`;
- applied to device registration and App ID/profile calls;
- regression tests verify Watch uses the iOS service path and carries the Watch platform marker.

CI run `34378142101`: **COMPLETED SUCCESS** on macOS, Ubuntu and Windows.

Physical result through corrected iLoader backend:

- previous QH65B2 404 is gone: **physically validated**;
- iPhone app installs and launches: **physically validated**;
- Watch companion is detected/advertised but still fails installation with the Watch message equivalent to:
  **“Impossible d’installer Watch Sensor Lab — cette app ne peut pas être installée car son intégrité n’a pas pu être vérifiée.”**
- Watch icon is still missing; treat this as a separate asset/metadata issue until proven otherwise.

## Windows / Watch device validation completed

Using `pymobiledevice3` from Windows:

- iPhone USB communication works;
- paired Watch is visible through CompanionProxy;
- Watch lockdown forwarding/pairing works;
- Watch identified as `Watch7,14`, watchOS `26.6`;
- Watch Developer Mode option was revealed through AMFI and then manually enabled/restarted by the user.

Therefore the remaining integrity failure is **not explained by Developer Mode being absent**.

## Current signing diagnosis — IMPORTANT

A system-log capture during the real installation attempt shows:

- the rewritten iPhone bundle installs successfully;
- the rewritten Watch bundle ID is advertised as locally available:
  `com.rzbck.watchsensorlab.<TEAM_ID>.watchkitapp`;
- the Watch app does not become an installed Watch application;
- the iPhone application has its expected `embedded.mobileprovision` and valid signed entitlements during installation.

Source inspection now exposes a concrete signing-path mismatch:

1. `isideload/src/sideload/sign.rs` correctly populates `BundleSigningSettings.embedded_mobileprovisions_by_bundle_id` and `entitlements_by_bundle_id` for nested bundles.
2. `isideload` itself knows about `Watch/*.app` via its patched `watch_apps` tree.
3. However the actual `apple-codesign` implementation used by `sign_bundle()` discovers nested bundles only under:
   - `Frameworks/`
   - `PlugIns/`
4. It does **not** currently recurse into `Watch/`.

Consequences in the current code path:

- the Watch profile can be requested and stored in `all_profiles`, but the final signer never opens `Watch/<Companion>.app` as a nested bundle;
- therefore the Watch-specific `embedded.mobileprovision` / entitlements map is not applied through that recursive signing path;
- the Watch executable/bundle is not signed as a first-class nested app by `apple-codesign`;
- this matches the remaining physical “integrity could not be verified” failure.

This is now the primary root-cause candidate and is substantially stronger than the previous device-registration hypothesis. **Final confirmation still requires a code correction plus physical Watch install/launch validation.**

## Exact current stopping point

Before the next code modification, the user verified locally:

```text
branch: feat/watch-companion-support-20260909
HEAD:   7c8d2a3008890ac771597ef72092cc7204e93b0b
status: clean
```

No signing fix for `Watch/` has been applied yet.

## Next exact step

Patch the signing backend so `Watch/*.app` is treated as a nested application bundle by the actual `apple-codesign` recursion used by `sign_bundle()`.

Before choosing implementation strategy, inspect the exact pinned `Dadoum/apple-crates` revision and dependency behavior. Prefer the smallest generic fix that:

1. recursively signs `Watch/*.app`;
2. applies `embedded_mobileprovisions_by_bundle_id` to the Watch bundle ID;
3. applies Watch profile entitlements to the Watch executable;
4. preserves existing `Frameworks/` and `PlugIns/` behavior;
5. adds a regression test proving a Watch bundle gets a nested profile/signing pass;
6. passes CI on macOS, Ubuntu and Windows;
7. is pinned into iLoader;
8. is tested on the same physical iPhone + Apple Watch.

Only after physical success should this be considered fixed.

## Do not do

- do not hard-code the user's Team ID;
- do not special-case `WatchSensorLab` by name — support must stay generic;
- do not conflate CI/build success with physical Apple Watch validation;
- do not treat the missing Watch icon as the integrity root cause without evidence;
- do not merge to upstream/main automatically;
- do not modify `Rzbck/ios-godot-lab` from this tooling fork without explicit scope change;
- do not discard or overwrite local work with destructive Git commands.
