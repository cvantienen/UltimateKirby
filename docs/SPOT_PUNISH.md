# Configured Spot punish

**Spot punish** reacts to an opponent's ordinary grounded spot dodge. It is
separate from **Grab dodge + punish**, which dodges an incoming grab and follows
up afterward. Both use their own ON/OFF rows and saved rule lists.

## Configure in Training

1. Open Assist modes, highlight **Spot punish**, and press **L3 or R3**.
2. Press X to add a rule, or A/R3 to edit an existing one.
3. Set your state, minimum/maximum X distance, height, opponent side and priority.
   Priority 1 is considered first; up to eight rules belong to this assist.
4. Choose response actions and delays. Press Y in the editor to save, then wait
   for the save result. R3/L3 returns to the list; B/L3 returns to Assist modes.
5. Enable Spot punish. It retains its existing ON/OFF default, but without a
   matching saved rule there is no response or compiled smash fallback.

```ini
[bowser.attack.spot_grab]
name = Spot dodge grab
start_delay = 0
timeout = 180
spatial_timeout = 30
default = false
reaction = spot_dodge:grounded:0:12:18:front:1
actions = face:opponent, input:r:0:0:4
```

This is a starting example, not calibrated grab reach. `input:r:0:0:4` holds the
Grab button for four input frames. `grab:NUMBER` instead means a range check;
use the editor's Hold buttons setting or explicit input syntax for duration.
Legacy storage headers such as `bowser.attack` remain compatible, but this
reaction-tagged recipe appears only in Spot punish's list, never Get-up attack.

## Trigger and timing

The `spot_dodge` event waits until the existing native dodge-window observation
reports that invulnerability has ended and recovery remains. Unknown windows,
non-spot-dodge states, range/side/height mismatches and invalid observations do
not start a response. There is one attempt per observed spot dodge; cancelling
an already started sequence does not retry the same dodge. A later genuine dodge
can qualify again. Move startup still occurs after the configured input starts.

A protected ledge setup and native safety retain their ownership. Spot punish
does not run stale events later. The configured timeline can continue after the
opponent's dodge ends; spatial steps require a fresh locked-target observation.
Unchanged native jump, shield, recovery and attack rules determine acceptance.

Detection reuses the existing bounded recovery scan. Following an active target
uses its validated identity directly. Floor support is queried only for relevant
configured ground movement, not idle Spot punish watching. There are no new
hooks, hitbox scans, worker threads or match-time file reads.

## Immediate spot-dodge variant

`spot_dodge_any` belongs to the same Spot punish list and switch. It can select a
response before the opponent becomes vulnerable, or when the native dodge-window
forecast is unavailable, provided the validated ordinary spot-dodge status/motion
is observed. It still respects own hitstop, ground/state conditions, ownership
and one attempt per dodge. The older `spot_dodge` event keeps its vulnerability
and remaining-recovery checks. Unknown forecast candidates are only retained by
the native observer when an explicit any-spot rule is configured.

A priority grab using `spot_dodge` can lose to a lower-priority `spot_dodge_any`
Up-B **before** vulnerability opens. That immediate response does not turn into a
grab later. This is the requested Bowser broad-chase behavior, not a timing guarantee.
