# Sonic ledge options

Sonic has a compiled profile (native kind `0x29`) instead of Kirby fallback.
Controls and the five-row menu are unchanged; see [menu controls](LEDGE_MENU.md).

| Row | Saved choices | Default |
| --- | --- | --- |
| Neutral | Nair, down tilt, forward tilt | Nair |
| Jump | Nair | Nair |
| Roll | Down-smash, grab | Down-smash |
| Attack | Shield → nair, grab / rear nair | Shield → nair |
| Drop | Shield, down tilt → shield | Shield |

[Frame data](https://ultimateframedata.com/sonic) supplies nair 6–38, down tilt
6–7, down-smash 12–13 and grab startup 7/10/11 (standing/dash/pivot), with
11-frame shield grab. Forward tilt schedules its continuous second hit at 8–11,
not the inactive gap after its frame-6 first hit. Grounded Spring Jump has no
damaging hitbox, so it is excluded from shield punishes. Spin Charge is also
excluded because it needs a separate charge/release implementation.

Nair uses the existing paired X+A delivery. Roll uses the shared locked coverage
point and braking logic; a successful grab hands control back for pummels/throws.
Y attempts B-reverse. Held-L3 recovery cover is Kirby-only; Sonic has no live
two-frame-cover binding. These are punish
attempts, not guaranteed contact: spacing and online delivery still matter.

Only this fighter's selected response executes. No combo engine, background
scan, dependency or network code was added. Source formatting/static checks only;
no build, upload, Rust tests or Switch measurements were performed.
