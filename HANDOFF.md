# HANDOFF — Apple Watch companion support in isideload

Date: **2026-09-09**

## Objective

Add generic support for embedded Apple Watch companion apps to `isideload`, so an IPA containing `Main.app/Watch/Companion.app` can be re-signed/provisioned coherently instead of only treating `PlugIns/` and `Frameworks/` as child bundles.

This is a separate tooling chantier from `Rzbck/ios-godot-lab` / Watch Sensor Lab. Do not modify the application repo from this fork unless explicitly requested.

## Repository / branch

- repo: `Rzbck/isideload` (fork of `nab138/isideload`)
- local path: `E:\_Project\IOS APP\_Tools\iloader-watch\isideload`
- branch: `feat/watch-companion-support-20260909`
- upstream baseline used by iLoader 2.3.1: `3d42025ecac97a2548d5b88aefc8028307e369c1`
- last code HEAD before this HANDOFF commit: `04d25c73742d29c2b8936c6f95741f22640ef6da`
- after this HANDOFF commit, always re-fetch GitHub/Git before asserting current HEAD.

## Why this patch exists

Original iLoader/isideload behavior exposed two hardware errors with Watch Sensor Lab:

1. `InvalidWatchKitApp` because the watchOS app initially lacked `WKApplication = true` — fixed in the application repo, not here.
2. Then `InvalidCompanionAppBundleIdentifier`: iLoader changed the iPhone bundle ID from `com.rzbck.watchsensorlab` to `com.rzbck.watchsensorlab.<TEAM_ID>` while the embedded Watch app still had `WKCompanionAppBundleIdentifier = com.rzbck.watchsensorlab`.

Inspection of upstream `isideload` showed child bundle discovery handled `PlugIns/` and `Frameworks/`, but not `Watch/` as a first-class app bundle. That meant the embedded Watch app did not participate correctly in bundle-ID rewriting/App-ID registration/provisioning.

## Implemented patch

At code SHA `04d25c73742d29c2b8936c6f95741f22640ef6da`:

- detect embedded `.app` bundles under `Watch/`;
- keep them as `watch_apps` in the bundle tree;
- rewrite Watch `CFBundleIdentifier` consistently with the re-signed main iPhone app;
- rewrite `WKCompanionAppBundleIdentifier` to the rewritten main iPhone bundle ID;
- include Watch app bundles in App-ID registration/provisioning collection;
- persist rewritten `Info.plist` files recursively before `apple-codesign` scans/signs the bundle tree;
- include Watch bundles in nested bundle traversal/signing order;
- add a Rust regression test proving coordinated rewrite:
  - main: `com.example.app` -> `com.example.app.TEAM`
  - watch: `com.example.app.watchkitapp` -> `com.example.app.TEAM.watchkitapp`
  - `WKCompanionAppBundleIdentifier` -> `com.example.app.TEAM`.

## CI validation

Workflow run: `34366333338`

Exact tested code SHA: `04d25c73742d29c2b8936c6f95741f22640ef6da`

Result: **SUCCESS** on all three platforms:

- macOS: test PASS, build PASS, artifact upload PASS;
- Ubuntu/Linux: test PASS, build PASS, artifact upload PASS;
- Windows: test PASS, build PASS, artifact upload PASS.

Artifacts from that run included:

- `minimal-windows.exe`
- `minimal-macos`
- `minimal-linux`

This validates compilation/tests only. It does not by itself validate Apple Watch hardware installation.

## Hardware result after integrating this backend into iLoader

The patched backend was pinned into `Rzbck/iloader` and a Windows iLoader build was produced/tested by the user.

Observed on real hardware:

- patched iLoader reports signing/install operation completed;
- Watch Sensor Lab installs and launches on the real iPhone: **VALIDATED BY USER**;
- the companion appears on Apple Watch but without a proper icon yet;
- attempting to install it on the Watch fails with the Watch message equivalent to: **“Impossible d’installer Watch Sensor Lab — cette app ne peut pas être installée car son intégrité n’a pas pu être vérifiée.”**

So the previous `InvalidCompanionAppBundleIdentifier` blocker is no longer the observed failure. The current blocker is later in the Watch installation/signature/provisioning path.

## Current leading hypothesis — NOT YET CONFIRMED

`Sideloader::install_app()` currently registers only the directly connected iPhone device using `IdeviceInfo::from_device(device_provider)` + `ensure_device_registered(...)` before profiles are generated.

There is no current logic in this patch to discover/register the paired Apple Watch UDID as a developer device before downloading the Watch provisioning profile.

This is therefore the leading hypothesis for the Watch “integrity could not be verified” failure:

- iPhone is registered and present in the iPhone provisioning profile;
- Watch App ID/profile is created;
- paired Apple Watch itself may not be registered/included in the Watch provisioning profile.

Do not present this as proven until the Watch UDID/profile contents are inspected.

## Developer Mode finding

On the user's real Apple Watch, `Settings > Privacy & Security > Developer Mode` is currently **absent**, not merely disabled.

Apple documentation indicates Developer Mode may not appear until a development pairing/session has been initiated. The user has no Mac/Xcode, so a Windows-only path is being investigated.

## Windows-only investigation in progress

Candidate tool: `pymobiledevice3`, because it exposes companion-device services and AMFI/developer-mode related functionality.

Local diagnostic directory:

`E:\_Project\IOS APP\_Tools\pymobiledevice3-watch`

A Python venv was created/being created and `pip install -U pymobiledevice3` was started. Current output shows intermittent DNS/network failures (`getaddrinfo failed`) while downloading from PyPI, followed by some successful metadata/download attempts. **Installation completion has NOT been confirmed yet.**

Planned read-only first check once install succeeds:

- list USB iPhone;
- list paired companion devices via `pymobiledevice3 companion list`;
- verify whether the paired Watch and an identifier/UDID are visible from Windows.

Do not attempt irreversible or undocumented developer-mode changes before confirming the companion can be enumerated.

## Next exact steps

1. finish/install `pymobiledevice3` on Windows or retry if network/DNS interrupted;
2. with iPhone USB-connected/unlocked and Watch nearby/unlocked, run the read-only companion listing;
3. determine whether the Watch UDID can be obtained through companion services;
4. determine whether Developer Mode can be exposed/enabled safely from Windows using supported device services;
5. inspect whether Apple developer registration/provisioning APIs can register both iPhone and paired Watch before profile generation;
6. patch `isideload` to register the Watch device only if the above is confirmed;
7. add tests/CI;
8. rebuild patched iLoader and retest on the same real iPhone + Watch;
9. only after hardware success, prepare a clean upstream PR to `nab138/isideload`.

## Do not do

- do not hard-code the user's Team ID into Watch plists;
- do not special-case `WatchSensorLab` by name — support must stay generic;
- do not claim Watch hardware validation until the app actually installs/launches on the physical Watch;
- do not merge to upstream/main automatically;
- do not modify `Rzbck/ios-godot-lab` from this tooling fork without explicit scope change.
