# Ledge recovery and the retired two-frame/regrab punish

The separate vulnerable-ledge/regrab automatic punish is **retired**. Hanging,
regrabbing or temporary ledge vulnerability alone does not launch an attack.
The retained `ledge_regrab` token is compatibility data: the defensive selector
explicitly refuses it. Saving that token does not restore the removed feature.

A **two-frame punish** is a player term for striking the brief vulnerable part of
some recoveries before ledge protection. This implementation does not prove such
a window, force vulnerability, or automatically time a hit there.

Ordinary climb/jump/roll/attack/drop responses still use
[Ledge trap](LEDGE_TRAP.md)'s saved slot timeline, one completed attempt per
visit. Drop requires witnessed hang-to-air release; merely seeing someone below
stage is insufficient. Ground anchor/standing preparation is separate playback.

To practice recovery coverage, edit Drop in Training using
[Sequence files](SEQUENCE_FILES.md), choose literal inputs/checks and test both
ledges. Native startup, stage geometry, spacing and opponent choices determine
whether those inputs hit. Disable the ledge role or remove its saved response;
there is no hidden two-frame toggle to turn off.

For a Rust extension, begin with `ledge_trap::response_slot` and
`defense_response::select`. The inert-token check is intentional. Reviving a
recovery-window event would require verified native observations, activation/
config/editor wiring and tests for visits, eligibility, cancellation and missing
data—not just deleting that check. Such an extension must remain input-only.
This documentation does not enable it.
