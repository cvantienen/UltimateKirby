# Pac-Man and Little Mac

Both have their own compiled ledge profile, using the existing five-row menu
and [unchanged controls](LEDGE_MENU.md). Bold choices below are defaults.

| Row | Pac-Man | Little Mac |
| --- | --- | --- |
| Neutral | **Nair**, down tilt, forward tilt, ghost f-smash | **Up tilt**, down tilt, forward tilt, armored f-smash, low f-smash, tiny nair |
| Jump | **Nair**, up-air, forward-air | **Grounded up tilt**, tiny up-air, tiny nair |
| Roll | Lingering down-smash ghosts, **long grab** | **Down-smash**, grab |
| Attack | **Shield → nair**, grab / rear nair, shield → trampoline | Tiny nair, grab / rear nair, **shield → up-B** |
| Drop | **Shield**, down tilt → shield, short-hop bite, low ghosts | **Shield**, down tilt → shield, low sweep, f-tilt poke |

Pac-Man's nair is active 3–19, forward-smash ghost 16–29, down-smash ghosts
15–28, and grab 12–33. The roll ghost variant schedules within 15–27, the existing
scheduler's maximum 13-frame window. This is native coverage, not an extended
hitbox. Grab returns control on capture. Shield-to-trampoline is a playful
alternative that hands control back on special startup. Fruit inventory and
hydrant setups are not automated.

Little Mac's up-tilt window is 4–10, so it offers more timing margin than his
one-frame nair at frame 2. His default jump response stays grounded; height and
reach still matter. Down-smash uses front 10–11 or rear 17–18, never the gap.
Shield-to-up-B uses its frame-3 startup after block evidence. Forward-smash uses
14–15; the down-angled variant has its own narrower frame-15 timing. Weak aerials
are labeled jokes and aren't default coverage. KO meter, charge moves and
counters aren't guessed or automated.

Move measurements: [Pac-Man](https://ultimateframedata.com/pac_man),
[Little Mac](https://ultimateframedata.com/little_mac).

The shared bounded single-move engine retains native acceptance, paired aerial
inputs, release-on-smash-startup, cancellation and manual handoff. No background
scan, combo engine, runtime fighter file, dependency or network code was added.
Wider active windows can improve tolerance but cannot guarantee contact, especially
with changing online delay or incorrect spacing. Formatting/static checks and
regression fixtures are included; no build, Rust tests, upload or Switch gameplay
measurements were performed for this update.
