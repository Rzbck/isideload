# HANDOFF — Apple Watch companion support public review

Date: 2026-09-10

## Objective

Turn the already-working generic Apple Watch companion support into a minimal, professional, upstream-reviewable contribution without mixing any application-specific code or data into isideload.

## Repository / branch

- repository: `Rzbck/isideload`
- branch: `feat/watch-companion-support-20260909`
- physically validated backend regression anchor: `f7b9f3da570edd6824c29680545e710846d07df5`
- matching physically validated iLoader source revision: `70f37e9b4afc659ab44ec1944c034093f4cda416`

Later commits may be documentation-only. Verify actual branch HEAD, diff and CI before source work.

## Physical evidence

The matching iLoader/isideload revisions were exercised on real iPhone + Apple Watch hardware through the Windows one-click path. The iPhone host installed/launched and the embedded Watch companion was provisioned, signed, installed and launched on the paired Watch.

This validates the tested path only; do not generalize to all entitlements, all IPA topologies or all watchOS versions without evidence.

## Generic backend responsibilities

The working backend changes address:

- recursive discovery of nested app-ID-bearing bundles;
- coherent identifier/companion relationship rewriting after sideload signing changes IDs;
- paired-Watch provisioning;
- watchOS-compatible profile/platform handling;
- explicit signing of Watch content;
- capability-aware provisioning, including the tested HealthKit path;
- context needed by the iLoader companion-install integration.

## Public sanitation rules

Do not include:

- local workstation paths;
- real device identifiers;
- Apple credentials, certificates, keys or provisioning profiles;
- Watch Tracker source or Health/GPS/activity data;
- tracker-specific bundle identifiers except sanitized examples;
- comments that only narrate discarded debugging hypotheses.

Source comments should explain invariants and platform constraints.

## Review plan

1. read `docs/WATCH_COMPANION_PORTING_NOTES.md`;
2. inspect the exact diff from the upstream-compatible base to the known-good revision;
3. isolate the smallest generic patch series preserving the validated behavior;
4. retain/add focused tests for nested bundle discovery, relationship rewriting, platform selection and capability decisions;
5. prepare a focused PR to upstream `nab138/isideload`;
6. do not merge or publish a crate/release without explicit approval;
7. after backend review shape is ready, prepare the matching iLoader PR.

## Separate public entry-point repository

A new public repository is intended as the clean documentation/integration landing page for this work. It should link to isideload/iLoader source branches and future PRs rather than duplicating both codebases.

Recommended files: `README.md`, `docs/ARCHITECTURE.md`, `docs/COMPATIBILITY.md`, `docs/TESTING.md`, `docs/UPSTREAM.md`, `SECURITY.md`, `LICENSE`.

## Start of next conversation

Verify repository, branch, HEAD and CI first. Then read this file, `docs/WATCH_COMPANION_PORTING_NOTES.md`, and the matching iLoader `HANDOFF_ONE_CLICK_WATCH_SUCCESS.md` before modifying source.
