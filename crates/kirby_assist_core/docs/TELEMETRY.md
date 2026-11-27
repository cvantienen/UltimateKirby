# Read-only Training telemetry

This feature is compiled only with `diagnostic-logging`. It is disabled by default and requires explicit activation in Training each session. It does not change fighter status, hitboxes, motion rates, physics, or assistance decisions. There are no remote controller commands, memory-address queries, or setters.

## Setup

Use the Switch and computer on the same trusted private LAN, preferably wired Ethernet. Allow inbound UDP on the chosen computer port in your local firewall. No router port forwarding is needed or recommended.

Create `/ultimate/kirby_assist/telemetry.txt` on the Switch SD card and a matching local computer file:

```ini
switch_ip=192.168.1.50
computer_ip=192.168.1.20
port=8766
token=REPLACE_WITH_A_RANDOM_32_TO_128_CHARACTER_SECRET
```

Replace both IP addresses and the example token. Use a randomly generated 32–128 character token containing only letters, digits, underscores, and hyphens. Keep the token private; do not commit the configured file. The Switch source UDP port and computer receiver UDP port are both `port`. Addresses must be distinct numeric private IPv4 addresses; the parser conservatively rejects final octets 0 and 255. Unknown, duplicate, missing, and invalid fields are rejected. Switch configuration is bounded to 2 KiB.

HMAC-SHA256 authenticates each packet and detects modification. It does **not** encrypt values. Use a trusted LAN, not an exposed or hostile network. Address restrictions do not prove subnet isolation.

Start the computer viewer from the repository root:

```sh
cargo run --manifest-path crates/kirby_assist_core/Cargo.toml --features diagnostic-logging --example telemetry_viewer -- /absolute/path/to/telemetry.txt
```

Open `http://127.0.0.1:8765/`. An optional second argument chooses a different localhost HTTP port. The UDP receiver binds only the configured computer address and accepts packets only from the configured Switch address and port with a valid MAC. The browser server binds only localhost; it never serves arbitrary files or exposes the configuration token. Stop the viewer with Ctrl-C.

In the diagnostic mod, enter Training with Ultimate Training Modpack, open the Kirby Assist menu with Minus, select Fighter Settings, then use `L + Left/Right` to reach Developer tools. Select **Live telemetry UDP**, then press A or Left/Right to toggle. The sequence editor HTTP service is a separate switch and configuration.

The menu shows STREAMING after the UDP socket is ready, not a verified connection acknowledgement. The receiver shows whether fresh authenticated observations are actually arriving. Socket/configuration failures appear on the menu row. Leaving Training or losing the 250 ms observation lease stops streaming and requires explicit reactivation. Telemetry remains available with gameplay assists disabled; only menu-owned controller inputs are consumed while navigating.

## Viewer

Select an object and group, search fields, browse the schema, and select a numeric field for a bounded history graph. Raw integers and hashes are preserved exactly as strings. Floats retain the original f32 bits. Unknown/nonfinite/unavailable fields are not converted to zero. The viewer marks data stale after 250 ms and expires old groups after one second. It reports packet gaps, reordering, rejected/wrong-source traffic, sender drops, throughput, and session changes.

## Current coverage and limitations

The catalog in `../src/telemetry.rs` defines names, types, and source labels. It covers common status/situation/motion, position/facing/scale, velocity and verified physics parameters, damage/stop, cancel/transition flags, capture, state-scoped work values, Game & Watch's verified Oil Panic gauge, attack activity, and verified geometry. Physical input is copied before mapper changes; mapped input is copied after the existing mapper/assist pipeline. The latter may include existing assistance and is not labeled physical input.

- Up to 16 validated roster identities, including validated Ice Climber partners.
- Up to 32 weapon/item IDs discovered through observed ATTACK callbacks. This is **not** a complete object-manager enumeration; nonattacking or previously unobserved objects are not guaranteed to appear. The tracker retains the latest 32 identities.
- Attack slots 0–7 only, matching the existing native scan. Local coordinates and geometry parameters are native; world endpoints/radius are derived using the existing joint transform and scaling convention.
- Opaque attack capability bytes, arbitrary work IDs, unverified getters, unknown offsets, and unrestricted engine memory are excluded. Unsupported shield-infliction and passive-wall-jump entries remain explicitly unavailable.
- Ground rays, dead-area probes, and arbitrary motion/parameter queries need query inputs; they are not universal live variables and are not remotely queryable.
- Sampling is wall-clock throttled to at most approximately 60 Hz from valid human-fighter callbacks, **not** an authoritative global simulation-frame counter. Values from multiple objects are sampled sequentially, not atomically across the engine. `sample_us` is the publication time since stream activation; `sequence` is the publication sequence, not an engine frame number.
- Packets/groups can be lost independently. No atomic whole-frame snapshot or object-allocation generation is promised. Native IDs are revalidated each pass; unobserved same-ID allocation reuse cannot be identified universally.

## Protocol v1

All scalar encoding is explicit little endian. Each packet is `92 + 8 * popcount(validity)` bytes, at most 604 bytes (below the 1,200-byte transport buffer). Missing fields consume no value bytes.

| Offset | Type | Meaning |
| --- | --- | --- |
| 0 | 4 bytes | `KATM` magic |
| 4 | u16 | Version 1 |
| 6 | u16 | Group: 0 object state, 1–8 attack slots 0–7, 9 input |
| 8 | u64 | Stream session |
| 16 | u64 | Publication sequence |
| 24 | u64 | Publication microseconds since activation |
| 32 | u32 | Battle-object ID |
| 36 | u32 | Native object category |
| 40 | i32 | Fighter/object kind |
| 44 | u64 | Cumulative sender drop count |
| 52 | u64 | Field validity bitset |
| 60 | packed u64 values | Present fields in ascending bit order |
| final 32 bytes | HMAC-SHA256 | MAC over all preceding bytes |

Signed values are sign-extended into u64; floats use their f32 bits in the low 32 bits; bools are 0/1. The receiver rejects invalid versions, groups, lengths, MACs, and stale/reordered packet sequences. Remembered retired sessions are bounded; restart the viewer if its session-history capacity is exhausted. HMAC does not provide universal replay prevention across receiver restarts.

The game thread performs no networking or file I/O and publishes copied scalars through a bounded 512-sample mailbox using try-lock. Old queued samples are dropped on overflow; contended samples are dropped instead of blocking the game. A worker handles config, encoding/MAC, and nonblocking UDP. It never owns native pointers.

## Verification

Focused host tests cover compact packet decoding/MAC validation, config validation, native stub sampling and scoped reads, geometry, missing values, Training gates, queue overflow, input preservation, and menu feature gates. These cannot certify actual game ABI, accuracy, or frame-time overhead.

Before relying on measurements, separately authorize and perform a diagnostic Switch deployment and test: activation/deactivation, Training exit/reset, assists off, CPU and partner fighters, projectile despawn, loss/disconnection, pauses/frame advance, accurate values, and acceptable native sampling cost. No hardware deployment is performed by setup or host checks.
