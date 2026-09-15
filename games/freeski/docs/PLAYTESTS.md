# Human playtest log

Human reports belong here. Automated checks remain in [VERIFICATION.md](VERIFICATION.md).
A feature reported working is not acceptance of every mode or the entire release.
For later entries, record date, build when known, mode/seed/course, input, window
conditions, observed outcome and unresolved questions. Leave unknowns explicit.

## 2026-09-14 — Tyler, first recorded acceptance session

Practice and a separate Free Ski run were exercised. Tyler clarified that the
yeti observations came from Free Ski. The latest prepared build was fc9a45a;
the exact running revision was not independently confirmed for this report.
Seed, duration, window size and scaling were not recorded. Successful completion
of the full Practice course was not explicitly reported.

| Area | Human result | Detail |
| --- | --- | --- |
| Keyboard and mouse steering | Reported working well | Controls feel great with both input methods. Preserve this feel. |
| Obstacles and collisions | Reported working well | Obstacles and crashes behave and look good. |
| Jumping | Reported working well | Jump works and looks good. |
| Crash allowance | Reported working | Third crash ends the run. |
| Pause and focus loss | Reported working well | Manual pause and automatic pause on window focus changes work well. |
| Artwork and sound | Approved in exercised play | Positive feedback on the updated presentation; not a full theme/scale audit. |
| Free Ski warning, chase and catch | Reported working well | Yeti warning appears, pursuit occurs and catching works very well. |
| Difficulty and speed | Needs revision | Game feels much too easy and insufficiently fast; escaping the yeti feels too forgiving compared with remembered SkiFree. |
| Yeti navigation | Unconfirmed concern | May get stuck too often. Tyler was unsure; no seed, replay or specific stall was captured. |

Do not reinterpret the navigation concern as a confirmed recurrence of the rock
deadlock fixed earlier. That defect and its bounded regression evidence remain
documented in [TUNING.md](TUNING.md).

## Current human acceptance

Updated from the follow-up session below. Broad approval is recorded as reported;
it does not imply unreported per-course or persistence edge-case coverage.

| Test | Status / expected observation |
| --- | --- |
| Difficulty revision | General speed/game feel approved; interception fix implemented after reported overshooting/delayed catches. Accepted by final sign-off below; no additional detailed run recorded. |
| Practice finish | Passed by report: reached the end and finished successfully. Restart was not separately described. |
| Slalom Cup | Approved in exercised play; missed gates add five seconds. All-five-course coverage, individual medals and unlocks not itemized. |
| Longer Free Ski runs | Approved: longer pursuit-off run; user explicitly marked test 5 good. Seeds and edge-route coverage not itemized. |
| Saved continuation | Passed in exercised Free Ski flow. Exact shelf/close sequence and mid-jump/race cases not itemized. |
| Records and settings | Broad settings approval; separate records and persistence details not itemized. |
| Menus and display variants | User marked test 6 good. Exact window sizes, themes, scale and mouse-only coverage not itemized. Actual-gameplay Arcade preview implemented; accepted by final sign-off below. |
| Mute and reduced effects | Covered by broad test-6 approval; individual toggle/reopen sequences not itemized. |
| Packaged Omarchy release | Pending: build/install/upgrade the final package, native switching and existing-save retention. |

## 2026-09-14 — difficulty changes approved, retest pending

Tyler approved implementing the proposed pursuit, speed and terrain changes and
explicitly requested F-key fast mode. This authorizes the implementation; it is
not a playtest of the resulting build. Preserve the first session's observations
above. Next human pass should start a new mountain and compare normal/fast chase
pressure, then cover Slalom and saved continuation. Record the tested revision.

## 2026-09-14 — Tyler, difficulty revision follow-up

The preceding launch opened the updated release app after implementation commit
f413344. This report follows that launch; no binary hash was captured during play.
Seeds, course names/count, input method, window size and scaling were not recorded.

| Area | Human result | Detail |
| --- | --- | --- |
| Free Ski speed and general feel | Approved in exercised play | Game feels pretty good now; preserve the improved movement and controls. |
| Fast-mode presentation | Fix requested | Skier should visibly tuck when fast mode is engaged. A distinct pose is sufficient; animation is optional. |
| Pursuit / catching | Fix required | Yeti repeatedly passes the skier, circles nearby and falls behind without catching. After a crash it came alongside and stopped; a catch eventually occurred. Reproducible seed/timing not captured; root cause unconfirmed. |
| Slalom Cup | Approved in exercised play | Cup looks and plays great; deliberate misses add five seconds. No explicit count of completed courses or medal/unlock breakdown. |
| Practice finish | Passed | Reached the end and finished; no problem reported. |
| Saved continuation | Passed in exercised flow | Free Ski save/resume works well. Exact transition sequence and saved jump/chase/race state not specified. |
| Longer pursuit-off Free Ski | Passed by report | Completed a longer run without pursuit; explicitly marked checklist test 5 good. |
| Settings and presentation | Approved in exercised checks | Explicitly marked checklist test 6 good, including settings. Individual configurations not listed. |
| Arcade shelf preview | Fix requested | Replace the current FreeSki selection image with an interesting actual gameplay screenshot. User also allowed a faithful recreation; prefer an actual capture. |

The pursuit report is concrete human evidence of missed interception/delayed
catching, superseding the earlier uncertain sticking concern. Do not infer that
all visible approaches must instantly catch, or that post-crash protection is
itself defective: reproduce approach geometry and protection timing first.
The user's memory of original SkiFree is a comparison, not verified historical
behavior. No gameplay or artwork fix was implemented while recording this report.

Next acceptance: confirm all five Cup courses were covered if needed, then
retest tuck readability and close-range pursuit after the fixes. Final packaged
install/upgrade acceptance remains separate from this development-build playtest.


## 2026-09-14 — follow-up fixes implemented, human retest pending

Implemented the tuck pose, tighter close-range pursuit and actual gameplay shelf
capture requested above. Automated reproduction and native checks belong in
VERIFICATION.md and do not replace Tyler's observations. Next human pass should
compare normal/fast approaches, post-crash catching and chase continuation, then
review tuck and shelf readability. All-five-course coverage and final packaged
release acceptance remain as scoped in NEXT.md.


## 2026-09-14 — Tyler, final sign-off

After delivery and reopening of commit 5458784, Tyler replied: “perfect. we're
done here.” This records acceptance of the finished gameplay and presentation,
including the three follow-up fixes. No additional seed, timed chase, individual
medal result or per-course test details accompanied the sign-off; do not invent
those observations. Final package installation/upgrade and integration with the
latest main branch remain engineering/release checks, separate from this approval.
