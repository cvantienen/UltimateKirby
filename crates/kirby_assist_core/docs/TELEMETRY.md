# Read-only Training telemetry

This feature is compiled only with `diagnostic-logging`. It is disabled by default and requires explicit activation in Training each session. It does not change fighter status, hitboxes, motion rates, physics, or assistance decisions. There are no remote controller commands, memory-address queries, or setters.

## Setup

Use the Switch and computer on the same trusted private LAN, preferably wired Ethernet. No token generation, manual key entry, or text configuration file is required. Old SD pairing files are not read by live activation. Do not forward router ports.

The explicitly activated Switch listens for telemetry subscriptions on **UDP 4243**. Enter its current numeric private IPv4 address and port in the viewer. The receiver subscribes from its own UDP socket; the Switch uses that source address and port, so there is no computer IP to copy onto the SD card. Subscriptions renew while the viewer runs and expire after three seconds without renewal. Permit the viewer's UDP traffic in the computer firewall.

**Token-free traffic is not authenticated or encrypted.** Private addressing and source filtering do not prove LAN isolation or prevent spoofing. Other private-LAN devices can subscribe, and can access the separately activated editor. Use this only on a trusted network; disable the services when finished. The localhost browser protections remain automatic and do not require entering a key.

Start the computer viewer from the repository root:

```sh
cargo run --offline --manifest-path crates/kirby_assist_core/Cargo.toml --features diagnostic-logging --example telemetry_viewer
```

Open `http://127.0.0.1:8765/` and enter the Switch IPv4 address and telemetry port (4243). An optional single numeric argument chooses a different localhost HTTP port. The UDP receiver binds an ephemeral port and accepts observations only from the selected Switch endpoint; changing endpoints clears the observation state. The browser server binds only localhost and never serves arbitrary files. Stop the viewer with Ctrl-C.

In the diagnostic mod, enter Training with Ultimate Training Modpack, open the Kirby Assist menu with Minus, select Fighter Settings, then use `L + Left/Right` to reach Developer tools. Select **Live telemetry UDP**, then press A or Left/Right to toggle. The sequence editor is activated separately and listens on **UDP and TCP 4242**. In Workbench's Training LAN tab, enter the Switch IPv4 address, use port 4242, and select UDP (preferred) or HTTP. There is no SD setup file or manual key. UDP transfers chunk the complete file with acknowledgments and preserve revision checks, validation, explicit save confirmation, and the existing SD save/reload path. Fragment retries retain the same transfer ID; saves are not automatically resubmitted or switched to another transport. If an acknowledgment is lost, reconnect and load the current file before deciding what to do next.

The menu shows STREAMING after the UDP socket is ready, not a verified connection acknowledgement. The receiver shows whether fresh observations are actually arriving. Socket/configuration failures appear on the menu row. Leaving Training or losing the 250 ms observation lease stops streaming and requires explicit reactivation. Telemetry remains available with gameplay assists disabled; only menu-owned controller inputs are consumed while navigating.

### Immediate shutdown / socket errors

The Switch Rust backend can report a failed native socket operation as the generic `os error 1` and leave its error kind unclassified. The diagnostic transports normalize the native socket error immediately: idle HTTP accepts (`WouldBlock`) keep listening, and temporary UDP send failures (`WouldBlock` / `ConnectionRefused`) count as dropped samples rather than stopping the worker. Other failures still stop the mode and report the operation and `Switch socket errno N`; do not treat every error 1 as a permissions problem or disable the Training safety gate.

An incomplete or malformed HTTP client request no longer shuts down the listener. Temporary reads/writes retry within the request deadline and Training lease; accepted TCP clients are explicitly put into blocking mode with short socket timeouts.

If activation still fails, record the complete menu status, including the operation and native errno. Verify the viewer's Switch IP is current, the diagnostic services are explicitly enabled in Training, and the firewall permits TCP/UDP 4242 for editing and UDP 4243 for telemetry. There are no required SD setup files. Host tests do not prove Switch connectivity; retest both modes on hardware with the matching updated desktop services.

## Viewer

Select an object and group, search fields, browse the schema, and select a numeric field for a bounded history graph. Raw integers and hashes are preserved exactly as strings. Floats retain the original f32 bits. Unknown/nonfinite/unavailable fields are not converted to zero. The viewer marks data stale after 250 ms and expires old groups after one second. It reports packet gaps, reordering, rejected/wrong-source traffic, sender drops, throughput, and session changes.

## Current coverage and limitations

The catalog in `../src/telemetry.rs` defines names, types, and source labels. It covers common status/situation/motion, position/facing/scale, velocity and verified physics parameters, damage/stop, cancel/transition flags, capture, state-scoped work values, Game & Watch's verified Oil Panic gauge, attack activity, and verified geometry. Physical input is copied before mapper changes; mapped input is copied after the existing mapper/assist pipeline. The latter may include existing assistance and is not labeled physical input.

- Up to 16 validated roster identities, including validated Ice Climber partners.
- Up to 32 weapon/item IDs discovered through observed ATTACK callbacks. This is **not** a complete object-manager enumeration; nonattacking or previously unobserved objects are not guaranteed to appear. The tracker retains the latest 32 identities within the current stream session; a new activation clears remembered identities.
- Attack slots 0–7 only, matching the existing native scan. Local coordinates and geometry parameters are native; world endpoints/radius are derived using the existing joint transform and scaling convention.
- Opaque attack capability bytes, arbitrary work IDs, unverified getters, unknown offsets, and unrestricted engine memory are excluded. Unsupported shield-infliction and passive-wall-jump entries remain explicitly unavailable.
- Ground rays, dead-area probes, and arbitrary motion/parameter queries need query inputs; they are not universal live variables and are not remotely queryable.
- Sampling is wall-clock throttled to at most approximately 60 Hz from valid human-fighter callbacks, **not** an authoritative global simulation-frame counter. Values from multiple objects are sampled sequentially, not atomically across the engine. `sample_us` is the publication time since stream activation; `sequence` is the publication sequence, not an engine frame number.
- Packets/groups can be lost independently. No atomic whole-frame snapshot or object-allocation generation is promised. Native IDs are revalidated each pass; unobserved same-ID allocation reuse cannot be identified universally.

## Telemetry protocol v1 / v2

All scalar encoding is explicit little endian. Each packet is `92 + 8 * popcount(validity)` bytes, at most 604 bytes (below the 1,200-byte transport buffer). Missing fields consume no value bytes.

| Offset | Type | Meaning |
| --- | --- | --- |
| 0 | 4 bytes | `KATM` magic |
| 4 | u16 | Version 1 authenticated / version 2 token-free |
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
| final 32 bytes | Tag | v1 HMAC-SHA256 over preceding bytes; v2 all zero (not authenticated) |

Signed values are sign-extended into u64; floats use their f32 bits in the low 32 bits; bools are 0/1. The token-free viewer accepts only v2 packets with the expected zero tag, valid groups/lengths and selected source endpoint. The portable codec retains keyed v1 support, verifies its MAC, and rejects mixing keyed and token-free modes. Stale/reordered packet sequences are rejected. Remembered retired sessions are bounded; restart the viewer if its session-history capacity is exhausted. HMAC does not provide universal replay prevention across receiver restarts.

The game thread performs no networking or file I/O and publishes copied scalars through a bounded 512-sample mailbox using try-lock. Old queued samples are dropped on overflow; contended samples are dropped instead of blocking the game. Revocation invalidates the Training observation lease even if the mailbox is contended, and the next activation discards all queued samples. A worker handles subscriptions, encoding, and nonblocking UDP. It never owns native pointers.

## Verification

Focused host tests cover compact packet decoding/MAC validation, config validation, native stub sampling and scoped reads, geometry, missing values, Training gates, queue overflow, input preservation, and menu feature gates. These cannot certify actual game ABI, accuracy, or frame-time overhead.

Before relying on measurements, separately authorize and perform a diagnostic Switch deployment and test: activation/deactivation, Training exit/reset, assists off, CPU and partner fighters, projectile despawn, loss/disconnection, pauses/frame advance, accurate values, and acceptable native sampling cost. No hardware deployment is performed by setup or host checks.
