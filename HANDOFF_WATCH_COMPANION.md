# HANDOFF — isideload Watch companion support

Date: 2026-09-09

## Objective

Add first-class support for embedded watchOS companion apps stored under `Main.app/Watch/*.app`, so iLoader can re-sign an iPhone app and its Watch companion coherently.

## Repository state

- repository: `Rzbck/isideload`
- upstream: `nab138/isideload`
- branch: `feat/watch-companion-support-20260909`
- base SHA: `3d42025ecac97a2548d5b88aefc8028307e369c1`
- base selected because iLoader 2.3.1 locks exactly this isideload commit
- local path used by user: `E:\_Project\IOS APP\_Tools\iloader-watch\isideload`

Always re-fetch branch/HEAD/status before further writes.

## Root cause being fixed

iLoader/isideload 0.3.17 rewrites the main iPhone bundle ID to `<original>.<TEAM_ID>`, but its bundle inventory only knew `PlugIns/` and `Frameworks/`. An embedded Watch app under `Watch/` was therefore not treated as an app bundle requiring its own rewritten ID, App ID and provisioning profile. `WKCompanionAppBundleIdentifier` also remained pointed at the pre-signing iPhone bundle ID.

Observed real-device failure:

```text
InvalidCompanionAppBundleIdentifier
... WKCompanionAppBundleIdentifier ... "com.rzbck.watchsensorlab"
... companion app's bundle identifier "com.rzbck.watchsensorlab.59858TV9N2"
```

## Patch scope

- detect `Watch/*.app` as nested bundles;
- recursively keep Watch bundles in the signing inventory;
- rewrite Watch `CFBundleIdentifier` using the same main-app suffix rule used for extensions;
- rewrite `WKCompanionAppBundleIdentifier` to the actual re-signed iPhone bundle ID;
- include Watch bundles in App ID registration/provisioning profile acquisition;
- persist rewritten Watch `Info.plist` files before `apple_codesign::sign_bundle()`;
- add a unit test for coordinated iPhone + Watch ID rewriting;
- run that test on Windows/macOS/Linux CI before the existing minimal build.

## Not yet validated

- CI compile/test result for the patch;
- Apple Developer API accepting registration/provisioning of the Watch bundle under a Personal Team;
- whether a paired Watch UDID must be registered separately before watchOS provisioning succeeds;
- real iLoader Windows build using this fork;
- physical install on iPhone and Apple Watch.

## Next exact step

1. obtain green `Rzbck/isideload` CI for this patch;
2. point `Rzbck/iloader` 2.3.1 branch at the validated isideload fork/commit;
3. build Windows iLoader in GitHub Actions;
4. install that experimental iLoader build locally;
5. retry the exact Watch Sensor Lab IPA and record the next real-device result.

## Do not modify

- `Rzbck/ios-godot-lab` as part of this signer patch;
- upstream `nab138/isideload` or `nab138/iloader` directly;
- `main` branches without explicit user approval.
