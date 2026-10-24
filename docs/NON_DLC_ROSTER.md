# Non-DLC roster: compiled options and frame-data audit

All base-game playable fighter forms now have their own compiled profile,
including echoes, the three Pokémon, the Miis and alternate-costume shared kinds.
The original 15 profiles are retained. The expansion adds 61 fighter forms plus
an explicit Nana identity entry: 77 native kind entries, representing 76 playable
forms when Pokémon are counted separately. Nana's entry does not add AI control,
desyncs or partner automation. Piranha Plant and the DLC packs have no new profiles.
The prior Kirby fallback for valid unconfigured kinds is unchanged.

Source files are `fighter_profiles/<fighter>/timing.rs` for frame data and
`ledge_trap/fighters/<fighter>.rs` for named response functions/lists.
Dark Samus shares the Samus menu because its zair/missile/Bomb and attack timings
match; its profile and native movement observations remain separate.
No fighter JSON or scripts are read by the plugin. These tables document the
source audit; they are not runtime configuration or device calibration results.

## Choosing options

The new lists generally offer nair/tilts/up-tilt/a stronger smash for Neutral;
nair, fair, up-air and/or a grounded up-tilt for Jump; down-smash, grab and/or
body down-tilt for Roll; shield-to-aerial, grab and selected quick Up-B responses
for Attack; and shield, down-tilt-to-shield, nearby aerials and selected low-smash
coverage for Drop. No Off entries or combos are added. All individual entries are
visible in each fighter's source menu and the existing in-game list.

Examples of stronger/cheesier alternatives include Ganondorf's sword f-smash,
Ness's bat reflector or yo-yo sweep, Game & Watch's bury hammers, Zelda's magic
f-smash, Mewtwo's shadow blast, Bowser's dropkick, and Villager's bowling ball.
These are ordinary single-move attempts, not automatic kills or guaranteed hits.
Ness's yo-yo entry is an uncharged sweep, not a pre-positioned charging trap.

All new Drop defaults are Shield except Dark Samus, which shares Samus's existing
Bomb default. `Hop ... (near)` means a stationary short-hop attack for close or
jump-back coverage: it does not walk off or pursue someone falling deep. Dair
entries stay near the edge with zero added horizontal drift and no forced fast-fall.
They still need suitable spacing. Stalling/plunging dair recipes are omitted
where a stationary hop could turn into an unwanted dive. Slingshot/air-shot and
ranged smash entries are area denial, not promised projectile arrival timing.

## Default selections

The expansion tables below cover the added base-game profiles. The original
profiles are documented in [the starting roster](STARTING_CAST.md),
[Luigi/Falcon](FIGHTER_ADDITIONS.md), [Sonic](SONIC_LEDGE.md),
[K. Rool/Isabelle](KROOL_ISABELLE_LEDGE.md), and
[Pac-Man/Little Mac](PACMAN_MAC_LEDGE.md). [Fighters](FIGHTERS.md) collects them all.

Every row can be changed independently. Drop defaults are described above.

| Fighter | Neutral | Jump | Roll | Attack |
| --- | --- | --- | --- | --- |
| [Dark Samus](https://ultimateframedata.com/dark_samus) | Down tilt | Nair | Grab | Shield > Up-B |
| [Ness](https://ultimateframedata.com/ness) | Nair | Nair | Grab | Shield > nair |
| [Jigglypuff](https://ultimateframedata.com/jigglypuff) | Nair | Nair | Down smash | Shield > nair |
| [Peach](https://ultimateframedata.com/peach) | Nair | Nair | Down smash | Shield > nair |
| [Daisy](https://ultimateframedata.com/daisy) | Nair | Nair | Down smash | Shield > nair |
| [Bowser](https://ultimateframedata.com/bowser) | Nair | Nair | Down smash | Shield > Up-B |
| [Sheik](https://ultimateframedata.com/sheik) | Nair | Nair | Down smash | Shield > nair |
| [Zelda](https://ultimateframedata.com/zelda) | Down tilt | Nair | Down smash | Shield > nair |
| [Dr Mario](https://ultimateframedata.com/dr_mario) | Nair | Nair | Down smash | Shield > Up-B |
| [Pichu](https://ultimateframedata.com/pichu) | Nair | Nair | Down smash | Shield > nair |
| [Falco](https://ultimateframedata.com/falco) | Nair | Nair | Down smash | Shield > nair |
| [Marth](https://ultimateframedata.com/marth) | Forward tilt | Up air | Down smash | Shield > Up-B |
| [Lucina](https://ultimateframedata.com/lucina) | Forward tilt | Up air | Down smash | Shield > Up-B |
| [Young Link](https://ultimateframedata.com/young_link) | Nair | Nair | Down smash | Shield > nair |
| [Ganondorf](https://ultimateframedata.com/ganondorf) | Nair | Nair | Down smash | Shield > nair |
| [Mewtwo](https://ultimateframedata.com/mewtwo) | Down tilt | Nair | Grab | Shield > nair |
| [Roy](https://ultimateframedata.com/roy) | Forward tilt | Up air | Down smash | Shield > nair |
| [Chrom](https://ultimateframedata.com/chrom) | Forward tilt | Up air | Down smash | Shield > nair |
| [Mr. Game & Watch](https://ultimateframedata.com/mr_game_and_watch) | Nair | Nair | Down smash | Shield > Up-B |
| [Meta Knight](https://ultimateframedata.com/meta_knight) | Nair | Nair | Grab | Shield > nair |
| [Pit](https://ultimateframedata.com/pit) | Down tilt | Nair | Down smash | Shield > nair |
| [Dark Pit](https://ultimateframedata.com/dark_pit) | Down tilt | Nair | Down smash | Shield > nair |
| [Zero Suit Samus](https://ultimateframedata.com/zero_suit_samus) | Down tilt | Up air | Grab | Shield > Up-B |
| [Wario](https://ultimateframedata.com/wario) | Nair | Nair | Down smash | Shield > nair |
| [Snake](https://ultimateframedata.com/snake) | Down tilt | Nair | Down smash | Shield > nair |
| [Ike](https://ultimateframedata.com/ike) | Down tilt | Nair | Down smash | Shield > nair |
| [Squirtle](https://ultimateframedata.com/squirtle) | Nair | Nair | Down smash | Shield > nair |
| [Ivysaur](https://ultimateframedata.com/ivysaur) | Down tilt | Nair | Down smash | Shield > nair |
| [Charizard](https://ultimateframedata.com/charizard) | Nair | Nair | Down smash | Shield > nair |
| [Diddy Kong](https://ultimateframedata.com/diddy_kong) | Nair | Nair | Down smash | Shield > nair |
| [Lucas](https://ultimateframedata.com/lucas) | Down tilt | Nair | Grab | Shield > nair |
| [King Dedede](https://ultimateframedata.com/king_dedede) | Nair | Nair | Down smash | Shield > nair |
| [Olimar](https://ultimateframedata.com/olimar) | Down tilt | Nair | Down tilt | Shield > nair |
| [Lucario](https://ultimateframedata.com/lucario) | Down tilt | Nair | Down smash | Shield > nair |
| [R.O.B.](https://ultimateframedata.com/rob) | Forward tilt | Nair | Down smash | Shield > nair |
| [Toon Link](https://ultimateframedata.com/toon_link) | Down tilt | Up air | Down smash | Shield > nair |
| [Wolf](https://ultimateframedata.com/wolf) | Nair | Nair | Down smash | Shield > nair |
| [Villager](https://ultimateframedata.com/villager) | Nair | Nair | Down smash | Shield > nair |
| [Mega Man](https://ultimateframedata.com/mega_man) | Down tilt | Fair | Grab | Shield > fair |
| [Wii Fit Trainer](https://ultimateframedata.com/wii_fit_trainer) | Nair | Nair | Down smash | Shield > nair |
| [Rosalina And Luma](https://ultimateframedata.com/rosalina_and_luma) | Nair | Nair | Down smash | Shield > nair |
| [Greninja](https://ultimateframedata.com/greninja) | Down tilt | Nair | Down smash | Shield > nair |
| [Palutena](https://ultimateframedata.com/palutena) | Down tilt | Up air | Down smash | Shield > nair |
| [Robin](https://ultimateframedata.com/robin) | Nair | Up air | Down smash | Shield > nair |
| [Shulk](https://ultimateframedata.com/shulk) | Down tilt | Nair | Down smash | Shield > nair |
| [Bowser Jr](https://ultimateframedata.com/bowser_jr) | Nair | Nair | Down smash | Shield > nair |
| [Duck Hunt](https://ultimateframedata.com/duck_hunt) | Nair | Nair | Grab | Shield > nair |
| [Ryu](https://ultimateframedata.com/ryu) | Nair | Nair | Down smash | Shield > nair |
| [Ken](https://ultimateframedata.com/ken) | Nair | Nair | Down smash | Shield > nair |
| [Cloud](https://ultimateframedata.com/cloud) | Nair | Nair | Down smash | Shield > nair |
| [Corrin](https://ultimateframedata.com/corrin) | Nair | Nair | Down smash | Shield > nair |
| [Bayonetta](https://ultimateframedata.com/bayonetta) | Nair | Nair | Grab | Shield > nair |
| [Inkling](https://ultimateframedata.com/inkling) | Nair | Nair | Down smash | Shield > nair |
| [Ridley](https://ultimateframedata.com/ridley) | Nair | Nair | Grab | Shield > nair |
| [Simon](https://ultimateframedata.com/simon) | Forward tilt | Nair | Down smash | Shield > nair |
| [Richter](https://ultimateframedata.com/richter) | Forward tilt | Nair | Down smash | Shield > nair |
| [Incineroar](https://ultimateframedata.com/incineroar) | Nair | Nair | Down smash | Shield > nair |
| [Mii Brawler](https://ultimateframedata.com/mii_brawler) | Nair | Nair | Down smash | Shield > nair |
| [Mii Swordfighter](https://ultimateframedata.com/mii_swordfighter) | Down tilt | Nair | Down smash | Shield > nair |
| [Mii Gunner](https://ultimateframedata.com/mii_gunner) | Nair | Nair | Down tilt | Shield > nair |
| [Ice Climbers](https://ultimateframedata.com/ice_climbers) | Nair | Nair | Down smash | Shield > nair |

## Active intervals used by the normal recipes

Values below are inclusive animation frames, before input delivery, jump squat
or a one-frame smash-flick primer. Separate rear sweeps are explicitly stored.
A dash means there is no separate rear interval; front-only recipes turn first.
Multi-hit entries use a continuous first interval, or conservatively the first
active frame when a span includes a rehit rate. Inactive gaps are never filled.
A continuous global active interval is not a promise that every point inside a
move's visual range stays covered for that entire interval.

The linked pages are the per-fighter frame-data sources. Comments in each profile
record additional cases and its movement reference. Extra fair/up-air/up-tilt/
f-smash/dair windows are in those same modules beside the main data.

| Fighter | Nair | D-tilt | F-tilt | D-smash front | Rear | Grab stand / dash / pivot / shield |
| --- | --- | --- | --- | --- | --- | --- |
| [Dark Samus](https://ultimateframedata.com/dark_samus) | 8–11 | 6–8 | 8–10 | 9–10 | 17–18 | 15 / 17 / 18 / 19 |
| [Ness](https://ultimateframedata.com/ness) | 5–15 | 3–4 | 7–10 | 23–25 | 12–15 | 6 / 9 / 10 / 10 |
| [Jigglypuff](https://ultimateframedata.com/jigglypuff) | 6–30 | 10–12 | 7–10 | 14–16 | — | 6 / 9 / 10 / 10 |
| [Peach](https://ultimateframedata.com/peach) | 5–19 | 8–9 | 7 | 6–7 | — | 6 / 9 / 10 / 10 |
| [Daisy](https://ultimateframedata.com/daisy) | 5–19 | 8–9 | 7 | 6–7 | — | 6 / 9 / 10 / 10 |
| [Bowser](https://ultimateframedata.com/bowser) | 8–29 | 10–12 | 10–14 | 12–14 | 28–30 | 8 / 11 / 12 / 12 |
| [Sheik](https://ultimateframedata.com/sheik) | 3–30 | 5–6 | 5–8 | 8–9 | 15–16 | 6 / 9 / 11 / 10 |
| [Zelda](https://ultimateframedata.com/zelda) | 6–7 | 5–11 | 12–13 | 5–6 | 13–14 | 10 / 13 / 14 / 14 |
| [Dr Mario](https://ultimateframedata.com/dr_mario) | 3–27 | 5–7 | 5–7 | 5–6 | 14 | 6 / 9 / 10 / 10 |
| [Pichu](https://ultimateframedata.com/pichu) | 3–27 | 7–8 | 5–12 | 8–9 | — | 6 / 8 / 9 / 10 |
| [Falco](https://ultimateframedata.com/falco) | 3–13 | 8–10 | 6–8 | 8–10 | — | 6 / 9 / 10 / 10 |
| [Marth](https://ultimateframedata.com/marth) | 6–7 | 7–8 | 8–11 | 6–7 | 21–23 | 6 / 9 / 10 / 10 |
| [Lucina](https://ultimateframedata.com/lucina) | 6–7 | 7–8 | 8–11 | 6–7 | 21–23 | 6 / 9 / 10 / 10 |
| [Young Link](https://ultimateframedata.com/young_link) | 4–27 | 8–9 | 10–12 | 9–10 | 21–22 | 12 / 14 / 15 / 16 |
| [Ganondorf](https://ultimateframedata.com/ganondorf) | 7–12 | 10–12 | 10–12 | 15–18 | 35–38 | 8 / 11 / 12 / 12 |
| [Mewtwo](https://ultimateframedata.com/mewtwo) | 7–8 | 6–7 | 10–11 | 21–23 | — | 7 / 10 / 11 / 11 |
| [Roy](https://ultimateframedata.com/roy) | 6–7 | 7–8 | 8–10 | 6–7 | 21–22 | 7 / 10 / 11 / 11 |
| [Chrom](https://ultimateframedata.com/chrom) | 6–7 | 7–8 | 8–10 | 6–7 | 21–22 | 7 / 10 / 11 / 11 |
| [Mr. Game & Watch](https://ultimateframedata.com/mr_game_and_watch) | 7–10 | 6–8 | 8–20 | 12–16 | — | 6 / 9 / 10 / 10 |
| [Meta Knight](https://ultimateframedata.com/meta_knight) | 6–20 | 3–4 | 6 | 4 | 9 | 7 / 11 / 12 / 11 |
| [Pit](https://ultimateframedata.com/pit) | 4–5 | 6–7 | 10–14 | 5–6 | 18–20 | 6 / 9 / 10 / 10 |
| [Dark Pit](https://ultimateframedata.com/dark_pit) | 4–5 | 6–7 | 10–14 | 5–6 | 18–20 | 6 / 9 / 10 / 10 |
| [Zero Suit Samus](https://ultimateframedata.com/zero_suit_samus) | 10–11 | 8–9 | 6–8 | 20–24 | — | 15 / 17 / 18 / 19 |
| [Wario](https://ultimateframedata.com/wario) | 4–17 | 5 | 12–15 | 8–36 | — | 8 / 11 / 12 / 12 |
| [Snake](https://ultimateframedata.com/snake) | 10–11 | 6–7 | 4–5 | 8–10 | 20–22 | 8 / 11 / 9 / 12 |
| [Ike](https://ultimateframedata.com/ike) | 10–22 | 7–8 | 12–13 | 13–15 | 32–36 | 7 / 10 / 11 / 11 |
| [Squirtle](https://ultimateframedata.com/squirtle) | 4–26 | 8–13 | 5–6 | 18–19 | 24–25 | 6 / 8 / 9 / 10 |
| [Ivysaur](https://ultimateframedata.com/ivysaur) | 7 | 4–6 | 10 | 13–14 | — | 13 / 13 / 13 / 17 |
| [Charizard](https://ultimateframedata.com/charizard) | 8–20 | 9–10 | 12–13 | 14–16 | — | 8 / 11 / 12 / 12 |
| [Diddy Kong](https://ultimateframedata.com/diddy_kong) | 8–20 | 4–5 | 10–16 | 7–8 | 12–13 | 7 / 10 / 11 / 11 |
| [Lucas](https://ultimateframedata.com/lucas) | 7 | 3–4 | 7–9 | 20–22 | — | 12 / 14 / 15 / 16 |
| [King Dedede](https://ultimateframedata.com/king_dedede) | 7–29 | 6–11 | 12–21 | 14–22 | — | 8 / 11 / 12 / 12 |
| [Olimar](https://ultimateframedata.com/olimar) | 7–8 | 6–12 | 15–17 | 10–13 | — | 12 / 12 / 12 / 16 |
| [Lucario](https://ultimateframedata.com/lucario) | 10–14 | 7–10 | 12–16 | 16–18 | — | 7 / 10 / 11 / 11 |
| [R.O.B.](https://ultimateframedata.com/rob) | 14–32 | 3 | 7–9 | 7–8 | — | 6 / 9 / 10 / 10 |
| [Toon Link](https://ultimateframedata.com/toon_link) | 6–7 | 9–10 | 9–13 | 9–10 | 17–18 | 12 / 14 / 15 / 16 |
| [Wolf](https://ultimateframedata.com/wolf) | 7–26 | 5–6 | 8–10 | 14–15 | 21–22 | 6 / 8 / 9 / 10 |
| [Villager](https://ultimateframedata.com/villager) | 3–23 | 9–11 | 8–11 | 9–11 | 29–31 | 14 / 16 / 17 / 18 |
| [Mega Man](https://ultimateframedata.com/mega_man) | 7–9 | 5–21 | 7–8 | 17–33 | — | 6 / 9 / 10 / 10 |
| [Wii Fit Trainer](https://ultimateframedata.com/wii_fit_trainer) | 9–13 | 9–12 | 6–10 | 18–21 | — | 6 / 9 / 10 / 10 |
| [Rosalina And Luma](https://ultimateframedata.com/rosalina_and_luma) | 9–33 | 5–8 | 7–9 | 6–7 | 17–18 | 6 / 9 / 10 / 10 |
| [Greninja](https://ultimateframedata.com/greninja) | 12–19 | 5–6 | 10–12 | 11–13 | — | 10 / 13 / 14 / 14 |
| [Palutena](https://ultimateframedata.com/palutena) | 5 | 14–27 | 14–35 | 17–19 | — | 7 / 10 / 11 / 11 |
| [Robin](https://ultimateframedata.com/robin) | 7–10 | 7–8 | 9–10 | 16–17 | — | 7 / 10 / 11 / 11 |
| [Shulk](https://ultimateframedata.com/shulk) | 13–30 | 10–11 | 12–13 | 18–19 | 23–24 | 7 / 10 / 11 / 11 |
| [Bowser Jr](https://ultimateframedata.com/bowser_jr) | 7–19 | 4–6 | 7–9 | 12–14 | — | 11 / 14 / 15 / 15 |
| [Duck Hunt](https://ultimateframedata.com/duck_hunt) | 6–37 | 6–7 | 8–11 | 12–13 | — | 6 / 9 / 10 / 10 |
| [Ryu](https://ultimateframedata.com/ryu) | 4–31 | 2–3 | 8–11 | 5–6 | — | 6 / 9 / 10 / 10 |
| [Ken](https://ultimateframedata.com/ken) | 6–17 | 2–3 | 8–11 | 5–6 | — | 6 / 9 / 10 / 10 |
| [Cloud](https://ultimateframedata.com/cloud) | 5–15 | 7–17 | 9–10 | 8–9 | 21–23 | 9 / 12 / 13 / 13 |
| [Corrin](https://ultimateframedata.com/corrin) | 6–19 | 5–7 | 8–9 | 13–15 | — | 7 / 10 / 11 / 11 |
| [Bayonetta](https://ultimateframedata.com/bayonetta) | 9–25 | 7–8 | 12 | 22–25 | — | 7 / 10 / 11 / 11 |
| [Inkling](https://ultimateframedata.com/inkling) | 6–9 | 5–8 | 8–10 | 11–12 | 20–21 | 8 / 9 / 10 / 12 |
| [Ridley](https://ultimateframedata.com/ridley) | 8–20 | 9–11 | 10–11 | 24–27 | — | 8 / 11 / 12 / 12 |
| [Simon](https://ultimateframedata.com/simon) | 8–9 | 7–23 | 12–13 | 14–15 | 20–21 | 10 / 13 / 14 / 14 |
| [Richter](https://ultimateframedata.com/richter) | 8–9 | 7–23 | 12–13 | 14–15 | 20–21 | 10 / 13 / 14 / 14 |
| [Incineroar](https://ultimateframedata.com/incineroar) | 5–25 | 9–11 | 12–14 | 18–20 | — | 7 / 11 / 12 / 11 |
| [Mii Brawler](https://ultimateframedata.com/mii_brawler) | 3–29 | 7–8 | 6–8 | 9–10 | — | 6 / 9 / 10 / 10 |
| [Mii Swordfighter](https://ultimateframedata.com/mii_swordfighter) | 10–23 | 5–6 | 10–11 | 7–8 | 15–16 | 6 / 9 / 10 / 10 |
| [Mii Gunner](https://ultimateframedata.com/mii_gunner) | 8–21 | 8–10 | 7–12 | 9 | 23–25 | 6 / 9 / 10 / 10 |
| [Ice Climbers](https://ultimateframedata.com/ice_climbers) | 6–23 | 8–11 | 9–10 | 9–11 | 16 | 8 / 10 / 10 / 12 |

Stored Mega Man pellet and shoto light-normal reference values above are not
registered as ordinary tilt/nair choices where their input/state rules differ.
Ness's front yo-yo interval is 23–25 and rear interval 12–15. Bayonetta's roll
smash uses the foot at 22–25, not the earlier grounded stomp. Robin uses common
early sword windows; Inkling's smash windows are conservative initial intervals.
No optional charge/gun/command followup is included in a hitbox window.

## Movement and drift audit

These are vanilla reference statistics from each linked page. The runtime uses
that fighter's native jump-speed parameters, gravity/fall-speed parameters,
current velocity and ground braking in the existing forecast/approach helpers.
Reference numbers below are not copied into a generic Kirby movement model.
Jump choices prefer a feasible short hop, otherwise a full hop, using the
selected move's own active interval. Grounded options remain grounded. Squat
and attack delivery are separate; a full hop holds jump until observed takeoff.
Stationary aerial choices deliberately add **zero drift**, so different air
speeds do not become arbitrary unmeasured forward-chase durations.

| Fighter | Gravity | Air speed | Air acceleration | Walk / run speed | Fall / fast-fall |
| --- | --- | --- | --- | --- | --- |
| [Dark Samus](https://ultimateframedata.com/dark_samus) | 0.075 | 1.103 | 0.09 | 1.115 / 1.654 | 1.33 / 2.128 |
| [Ness](https://ultimateframedata.com/ness) | 0.077 | 1.007 | 0.1 | 0.907 / 1.609 | 1.31 / 2.096 |
| [Jigglypuff](https://ultimateframedata.com/jigglypuff) | 0.053 | 1.332 | 0.13 | 0.735 / 1.271 | 0.98 / 1.568 |
| [Peach](https://ultimateframedata.com/peach) | 0.070 | 1.029 | 0.12 | 0.924 / 1.595 | 1.19 / 1.904 |
| [Daisy](https://ultimateframedata.com/daisy) | 0.070 | 1.029 | 0.12 | 0.924 / 1.595 | 1.19 / 1.904 |
| [Bowser](https://ultimateframedata.com/bowser) | 0.125 | 1.155 | 0.05 | 0.901 / 1.971 | 1.77 / 2.832 |
| [Sheik](https://ultimateframedata.com/sheik) | 0.150 | 1.155 | 0.09 | 1.47 / 2.42 | 1.75 / 2.8 |
| [Zelda](https://ultimateframedata.com/zelda) | 0.071 | 1.092 | 0.065 | 0.914 / 1.43 | 1.35 / 2.16 |
| [Dr Mario](https://ultimateframedata.com/dr_mario) | 0.087 | 0.9238784 | 0.061184 | 0.917301 / 1.397792 | 1.5 / 2.4 |
| [Pichu](https://ultimateframedata.com/pichu) | 0.140 | 1.029 | 0.1 | 1.302 / 1.892 | 1.9 / 2.5 |
| [Falco](https://ultimateframedata.com/falco) | 0.130 | 0.977 | 0.1 | 1.344 / 1.619 | 1.8 / 2.88 |
| [Marth](https://ultimateframedata.com/marth) | 0.075 | 1.071 | 0.08 | 1.575 / 1.964 | 1.58 / 2.528 |
| [Lucina](https://ultimateframedata.com/lucina) | 0.075 | 1.071 | 0.08 | 1.575 / 1.964 | 1.58 / 2.528 |
| [Young Link](https://ultimateframedata.com/young_link) | 0.096 | 0.966 | 0.06 | 1.26 / 1.749 | 1.8 / 2.88 |
| [Ganondorf](https://ultimateframedata.com/ganondorf) | 0.108 | 0.83 | 0.04 | 0.767 / 1.34 | 1.65 / 2.64 |
| [Mewtwo](https://ultimateframedata.com/mewtwo) | 0.082 | 1.313 | 0.07 | 1.26 / 2.255 | 1.55 / 2.48 |
| [Roy](https://ultimateframedata.com/roy) | 0.114 | 1.302 | 0.05 | 1.208 / 2.145 | 1.8 / 2.88 |
| [Chrom](https://ultimateframedata.com/chrom) | 0.114 | 1.302 | 0.05 | 1.208 / 2.145 | 1.8 / 2.88 |
| [Mr. Game & Watch](https://ultimateframedata.com/mr_game_and_watch) | 0.080 | 1.176 | 0.1 | 1.18 / 1.679 | 1.24 / 1.984 |
| [Meta Knight](https://ultimateframedata.com/meta_knight) | 0.110 | 1.04 | 0.075 | 1.239 / 2.09 | 1.66 / 2.656 |
| [Pit](https://ultimateframedata.com/pit) | 0.081 | 0.935 | 0.075 | 1.259 / 1.828 | 1.48 / 2.368 |
| [Dark Pit](https://ultimateframedata.com/dark_pit) | 0.081 | 0.935 | 0.075 | 1.259 / 1.828 | 1.48 / 2.368 |
| [Zero Suit Samus](https://ultimateframedata.com/zero_suit_samus) | 0.120 | 1.26 | 0.095 | 1.47 / 2.31 | 1.7 / 2.72 |
| [Wario](https://ultimateframedata.com/wario) | 0.107 | 1.271 | 0.12 | 0.889 / 1.65 | 1.61 / 2.576 |
| [Snake](https://ultimateframedata.com/snake) | 0.080 | 0.987 | 0.03 | 0.882 / 1.595 | 1.73 / 2.768 |
| [Ike](https://ultimateframedata.com/ike) | 0.092 | 1.134 | 0.05 | 0.912 / 1.507 | 1.65 / 2.64 |
| [Squirtle](https://ultimateframedata.com/squirtle) | 0.128 | 1.01 | 0.105 | 1.281 / 1.76 | 1.35 / 2.16 |
| [Ivysaur](https://ultimateframedata.com/ivysaur) | 0.082 | 0.998 | 0.095 | 1.103 / 1.595 | 1.38 / 2.208 |
| [Charizard](https://ultimateframedata.com/charizard) | 0.110 | 1.103 | 0.06 | 1.187 / 2.2 | 1.52 / 2.432 |
| [Diddy Kong](https://ultimateframedata.com/diddy_kong) | 0.125 | 0.924 | 0.05 | 1.313 / 2.006 | 1.75 / 2.8 |
| [Lucas](https://ultimateframedata.com/lucas) | 0.090 | 1.155 | 0.095 | 0.893 / 1.65 | 1.37 / 2.192 |
| [King Dedede](https://ultimateframedata.com/king_dedede) | 0.097 | 0.735 | 0.05 | 1.029 / 1.496 | 1.95 / 3.12 |
| [Olimar](https://ultimateframedata.com/olimar) | 0.068 | 0.861 | 0.08 | 0.945 / 1.617 | 1.35 / 2.16 |
| [Lucario](https://ultimateframedata.com/lucario) | 0.084 | 1.281 | 0.09 | 1.103 / 1.705 | 1.68 / 2.688 |
| [R.O.B.](https://ultimateframedata.com/rob) | 0.090 | 1.134 | 0.085 | 1.178 / 1.725 | 1.6 / 2.56 |
| [Toon Link](https://ultimateframedata.com/toon_link) | 0.081 | 1.05 | 0.09 | 1.288 / 1.906 | 1.38 / 2.208 |
| [Wolf](https://ultimateframedata.com/wolf) | 0.130 | 1.281 | 0.09 | 1.208 / 1.54 | 1.8 / 2.88 |
| [Villager](https://ultimateframedata.com/villager) | 0.078 | 0.987 | 0.06 | 1.092 / 1.397 | 1.32 / 2.112 |
| [Mega Man](https://ultimateframedata.com/mega_man) | 0.107 | 1.155 | 0.11 | 1.132 / 1.602 | 1.8 / 2.88 |
| [Wii Fit Trainer](https://ultimateframedata.com/wii_fit_trainer) | 0.090 | 1.019 | 0.09 | 1.224 / 1.866 | 1.3 / 2.08 |
| [Rosalina And Luma](https://ultimateframedata.com/rosalina_and_luma) | 0.062 | 1.05 | 0.11 | 1.138 / 1.795 | 1.2 / 1.92 |
| [Greninja](https://ultimateframedata.com/greninja) | 0.180 | 1.239 | 0.08 | 1.502 / 2.288 | 1.85 / 2.96 |
| [Palutena](https://ultimateframedata.com/palutena) | 0.120 | 1.0 | 0.115 | 1.271 / 2.077 | 1.55 / 2.48 |
| [Robin](https://ultimateframedata.com/robin) | 0.089 | 1.05 | 0.08 | 0.936 / 1.265 | 1.5 / 2.4 |
| [Shulk](https://ultimateframedata.com/shulk) | 0.098 | 1.113 | 0.05 | 1.155 / 1.672 | 1.58 / 2.528 |
| [Bowser Jr](https://ultimateframedata.com/bowser_jr) | 0.092 | 1.134 | 0.08 | 0.924 / 1.566 | 1.65 / 2.64 |
| [Duck Hunt](https://ultimateframedata.com/duck_hunt) | 0.076 | 1.155 | 0.08 | 1.213 / 1.793 | 1.65 / 2.64 |
| [Ryu](https://ultimateframedata.com/ryu) | 0.120 | 1.12 | 0.035 | 0.75 / 1.6 | 1.6 / 2.24 |
| [Ken](https://ultimateframedata.com/ken) | 0.12 | 1.12 | 0.035 | 0.825 / 1.76 | 1.6 / 2.24 |
| [Cloud](https://ultimateframedata.com/cloud) | 0.098 (Limit — 0.1078) | 1.155 (Limit — 1.386) | 0.07 (Limit — 0.084) | 1.155 (Limit — 1.32825) / 2.167 (Limit — 2.3837) | 1.68 (Limit — 1.848) / 2.688 (Limit — 2.9568) |
| [Corrin](https://ultimateframedata.com/corrin) | 0.092 | 1.019 | 0.05 | 1.208 / 1.595 | 1.65 / 2.64 |
| [Bayonetta](https://ultimateframedata.com/bayonetta) | 0.120 | 1.019 | 0.095 | 0.945 / 1.76 | 1.77 / 2.832 |
| [Inkling](https://ultimateframedata.com/inkling) | 0.087 | 1.208 | 0.08 | 1.134 / 1.925 | 1.58 / 2.528 |
| [Ridley](https://ultimateframedata.com/ridley) | 0.090 | 1.05 | 0.07 | 1.1 / 2.2 | 1.78 / 2.848 |
| [Simon](https://ultimateframedata.com/simon) | 0.085 | 0.94 | 0.03 | 0.76 / 1.52 | 1.85 / 2.96 |
| [Richter](https://ultimateframedata.com/richter) | 0.085 | 0.94 | 0.03 | 0.76 / 1.52 | 1.85 / 2.96 |
| [Incineroar](https://ultimateframedata.com/incineroar) | 0.126 | 0.88 | 0.07 | 0.62 / 1.18 | 1.76 / 2.816 |
| [Mii Brawler](https://ultimateframedata.com/mii_brawler) | 0.192 | 1.15 | 0.068 | 1.26 / 1.92 | 1.92 / 3.072 |
| [Mii Swordfighter](https://ultimateframedata.com/mii_swordfighter) | 0.106 | 1.17 | 0.06 | 1.07 / 1.58 | 1.55 / 2.48 |
| [Mii Gunner](https://ultimateframedata.com/mii_gunner) | 0.098 | 0.93 | 0.053 | 1.00 / 1.37 | 1.45 / 2.32 |
| [Ice Climbers](https://ultimateframedata.com/ice_climbers) | 0.082 | 0.83 (0.8715) | 0.09 (0.0945) | 1.008 (1.1088) / 1.53 (1.683) | 1.3 / 2.08 |

## Native-state and resource exceptions

- Mega Man omits pellet nair/forward-tilt and the rear-nair grab fallback. Fair,
  body tilts and grabs have separate choices; projectile up-air isn't treated as
  a timed body hitbox.
- Ryu/Ken omit ambiguous light/heavy tilts and command inputs. Their ordinary
  aerials and smashes use their own windows; front-only roll sweeps turn first.
- Meta Knight's special down/forward-tilt followup statuses are not treated as
  generic tilt acceptance. The expansion offers his nair, up-tilt, smashes and
  aerial alternatives instead.
- Bayonetta's first fair recognizes native `ATTACK_AIR_F` (0x1E1) as well as the
  generic air-attack state, while still requiring the exact first-fair motion.
  It does not chain fair 2/3 or extend bullet arts. Motion/state mismatches end it.
- Olimar defaults to body down-tilt for Neutral/Roll and body nair for aerial
  coverage. Pikmin-labelled aerials, smash and grab require available Pikmin.
- Robin uses common early bronze/Levin intervals. No tome/Levin recharge or
  forced weapon form is attempted. Rosalina timing describes her own hitboxes;
  independently positioned Luma is not assumed to connect.
- Miis use common normals and aerials only; no selected custom special is assumed.
  Shulk has no automatic art switching. Arts, buffs, size changes and transformations
  can alter physics or geometry beyond the neutral reference values.
- Projectile choices fire as coverage rather than guessing a constant speed or
  treating generation as collision. Their travel, reflection, absorption, item/
  resource requirements and stage interaction remain native game behavior.

## Verification and limits

The source pass checks native kind registration, nonempty/unique choice lists,
valid defaults, timing bounds, declared handlers and published movement references.
Regression fixtures cover roster dispatch, paired aerial inputs through target
loss/hitstop, custom fair acceptance, front-only roll turns, and the resource-free
roll down-tilt route. Formatting and static checks do not execute those fixtures.
**No Rust tests, build, upload or Switch calibration was performed.**

These are source-checked attempts, not a claim of universal optimal or guaranteed
frame-perfect contact. Hitbox/hurtbox geometry, player spacing, opponent steering,
platforms, native action readiness and varying input delivery still determine
hits. Missing native physics keeps the existing documented short-hop fallback.
The shared 25-unit roll coverage point remains a heuristic, not a measured roll
endpoint for every opponent. Disjoint phases are not extended to hide missed
frames, and no position/status/velocity/hitbox or networking writes are added.

The roster expansion adds immutable data/functions and startup catalog entries.
Only the selected fighter/response runs. No per-frame all-fighter simulation,
weapon scan, background file loading, network traffic or new UI telemetry is
introduced. Code/data size grows; actual performance and heat require profiling.
Current bindings are L autopilot (including Kirby in-range recovery cover),
R3 menu, Auto-mash menu row and held Y for configured modes (Kirby Auto-duck).
B-reverse and the separate R3 recovery binding are removed. See [controls](CONTROLS.md).

## Roll-grab placement

All profiles now include a standing-grab maximum-reach reference from
[UFD's grab-range table](https://ultimateframedata.com/stats#grabrange).
The initial approach uses the nearer side of the common roll estimate, with a
35%-of-standing-reach standoff bounded to 3–9 stage units. Yoshi's 26-unit
reference gives a 9-unit standoff; Isabelle's 19-unit reference gives 6.65.
The navigation area uses 60% of standing reach. These are approximate positioning
margins, not animated collision bounds; required resources such as Pikmin still
matter.

Kirby and Yoshi use 0.65 inward walking stick and can extend the approach point
inward when observed roll travel goes beyond it. The navigation forecast is
bounded to six frames at the live zero-extra-delay setting. They brake against
crossing the target instead of stopping outside the area's edge. They do not
chase back toward the ledge. Other fighters retain their prior navigation style.

All shared roll grabs check predicted opponent reach/facing, height and
vulnerability on the same active frame, using the full configured standing
range. An already useful position can request without reaching the common
point. Native can_grab, movement release and bounded delivery remain necessary;
confirmed capture hands back controls without pummel/throw. Stance preparation
cannot replace recognized roll navigation. See [Kirby/Yoshi](KIRBY_YOSHI_LEDGE.md#roll-grabs)
for the logged failure and changes.

The common estimate and velocity forecasts remain approximate across opponents.
Grab shape, height, acceleration, slopes and input arrival require Switch tests;
these checks cannot guarantee every grab.


### Grab delivery after approach

Grab delivery now starts at the actual R request, with a separate short clock.
A walking/running animation wrap during approach cannot expire the request
before R is ever emitted. Native catch readiness is checked across the allowed
catch variants during locomotion/braking; a guard grab uses its native catch
readiness even when ordinary movement is unavailable. Guard remains held while
waiting for that grab window.

Turning has a separate attempt budget from dash preparation. A rejected R
request gets a clean release before another attempt, without the old special
request's eight-frame retry spacing. Delivery stops on native grab acceptance;
only observed capture hands back pummel/throw control. Attempts remain bounded,
and the existing range, protection, ground-support and recovery gates remain.

Regression fixtures cover animation wraps before/during delivery, a spent dash
budget, shield readiness, retries, and actual R press/hold/release through the
shared mapper overlay. These fixtures have not been compiled or run as part of
this source-only change; device validation is still required.
