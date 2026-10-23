# Starting-roster ledge profiles

Mario, Donkey Kong, Link, Samus, Yoshi, Kirby, Fox and Pikachu have independent
compiled move data and ledge choices. Luigi and Captain Falcon remain registered.
The remaining non-DLC fighters now have [dedicated profiles](NON_DLC_ROSTER.md).
Other unconfigured kinds retain Kirby fallback. Combo implementations are removed.
These are configured punish attempts; spacing, native readiness and input delivery
still determine contact. No device measurements or gameplay validation were run
for this simplification.

|Fighter|Neutral default|Attack default|
|---|---|---|
|[Mario](https://ultimateframedata.com/mario)|Nair|Shield → Up-B|
|[Donkey Kong](https://ultimateframedata.com/donkey_kong)|Forward tilt|Shield → grab / rear nair|
|[Link](https://ultimateframedata.com/link)|Nair|Shield → Up-B|
|[Samus](https://ultimateframedata.com/samus)|Down tilt|Shield → Up-B|
|[Yoshi](https://ultimateframedata.com/yoshi)|Nair|Shield → nair|
|[Kirby](https://ultimateframedata.com/kirby)|Nair|Shield → grab / rear nair|
|[Fox](https://ultimateframedata.com/fox)|Nair|Shield → nair|
|[Pikachu](https://ultimateframedata.com/pikachu)|Forward tilt|Shield → nair|

The general starting-roster lists offer Nair, down tilt and forward tilt for Neutral; Nair
for Jump; down-smash and/or grab for Roll; shield-nair and shield-grab for Attack; and shield
or one down-tilt-then-shield for Drop. Mario/Link/Samus also offer shield-Up-B.
Samus additionally has [zair choices in every response row](SAMUS_LEDGE.md),
Roll → Grab and immediate Drop → down-B. Samus now defaults to roll grab and
drop Bomb; its other defaults above remain the same.
Yoshi additionally offers the dedicated choices described below.
There are no Off entries. Kirby retains its additional specials, roll grab and
drop-dair choices. Shield-Up-B waits for a real block and shieldstun ending,
keeps shield with the upward B request, and returns control on native special
startup. Its startup is Mario 3, Link 7 (front only), Samus 4 animation frames;
those values do not include shieldstun or unknown online transport time.

## Yoshi's expanded choices

| Row | Choices (default first) |
| --- | --- |
| Neutral | Nair; down tilt; forward tilt; up tilt; headbutt forward-smash |
| Jump | Nair; up air; meteor forward-air (spacing dependent) |
| Roll | Down-smash; tongue grab |
| Attack | Shield → nair; grab / rear nair; shield → up air |
| Drop | Shield; down tilt → shield; meteor chase (risky); dair shred (risky) |

The original defaults and option indices stay unchanged. Nair retains its long
3–25 active interval; its later hits are weaker than the initial kick. Up-air
uses only frames 5–6, forward-air 16–20, up-tilt 8–15 and headbutt f-smash 14–16.
Forward-air only meteors with the appropriate part of its hitbox, so selecting
it does not guarantee a spike. Headbutt is a slower, stronger alternative to
the quicker tilts and is released on native smash startup rather than charged.
These windows are from [Yoshi's frame data](https://ultimateframedata.com/yoshi).

Yoshi Jump nair immediately uses a short hop. Up-air and fair retain adaptive
short/full-hop selection. Neutral/Jump nairs use the brief quarter-stick nudge
and earlier timing described in [Kirby/Yoshi](KIRBY_YOSHI_LEDGE.md). Shield up-air
waits for an actual block and shieldstun ending; nair remains the faster default.
Roll grab uses the existing approach/braking and native grab acceptance logic,
then hands control back without a pummel or automatic throw.

The drop chases begin a short-hop with paired Jump+A and face toward the dropped
opponent. Forward drift continues after the exact aerial is observed: through
frame 20 for fair, or through frame 41 for dair. Dair's separate hits are never
used as one continuous timed get-up window. These options do not automatically
fast-fall, double-jump or recover; spacing and recovery remain the player's job.
They can whiff, be interrupted or carry Yoshi offstage, which is why safe shield
remains the default. Spacing labels are cached estimates, not measured ranges.

These are compiled single-move functions using existing input/observation paths,
with no extra opponent scans, files or network calls. No build, upload, Rust
test execution or device validation was performed for this addition; source
fixtures cover paired inputs, startup release and fair drift handback. Timing
data and native acceptance improve attempts but cannot guarantee online contact
or establish a universally optimal punish without gameplay testing.

## Stored continuous ledge windows

All values below are inclusive animation frames. Disjoint hits remain separate;
for example Pikachu's nair does not count its inactive gaps as hitbox coverage.
The shared 25-unit roll coverage point remains a selectable global numeric
setting, not a measured universal roll endpoint.

| Fighter | Nair | Down tilt | Forward tilt | Down-smash front | Rear sweep | Grab stand / dash / pivot / shield |
| --- | --- | --- | --- | --- | --- | --- |
| Mario | 3–27 | 5–7 | 5–7 | 5–6 | 14 | 6 / 9 / 10 / 10 |
| Donkey Kong | 10–26 | 6–7 | 7–9 | 11–14 | Same | 8 / 11 / 12 / 12 |
| Link | 7–31 | 10–11 | 15–19 | 12–13 | 24–25 | 6 / 9 / 10 / 10 |
| Samus | 8–11 | 6–8 | 8–10 | 9–10 | 17–18 | 15 / 17 / 18 / 19 |
| Yoshi | 3–25 | 8–10 | 5–7 | 7–8 | 22–23 | 14 / 16 / 17 / 18 |
| Kirby | 8–32 | 4–6 | 5–8 | 7–19 | Same | 6 / 9 / 10 / 10 |
| Fox | 4–23 | 7–8 | 6–8 | 6–7 | Same | 6 / 10 / 11 / 10 |
| Pikachu | 3–6 | 7–8 | 6–8 | 8–9 | Same first pulse | 7 / 11 / 12 / 11 |

Rear down-smash timing is selected by direction relative to native facing in
roll/two-frame forecasts. Luigi's rear sweep is 14–15; Falcon's is 29–30. Shield
grab timing is now separate from pivot timing, fixing their different startup
on Samus, Yoshi, Fox and Pikachu. Actual contact still depends on geometry.

## Source and cost

Move data lives in `fighter_profiles/<fighter>/timing.rs`; response defaults and
handlers live in `ledge_trap/fighters/<fighter>.rs`. Catalogs initialize once;
inactive fighters are not simulated. The menu changes indices/function pointers
and caches resolved settings. Full movement/ledge observations run only when a
tool needs them, with no throttling of active action timing. See
[the menu](LEDGE_MENU.md) for current controls and validation limits.
