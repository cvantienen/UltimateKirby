# Ledge-menu source review

Live combo playback, combo target acquisition and combo-specific damage/attribution
reads have been removed. Existing ledge, trip, fair-confirm and SDI observations
remain. The menu only selects compiled ledge responses. See
[the current menu](LEDGE_MENU.md) for controls and performance changes.

The existing mapper overlay/restoration, local-human routing, higher-priority
mash/tech/SDI, freshness and cancellation guards remain. There are no new
network/service calls, dependencies, clock hooks, or fighter-state writes.
Rendering still costs work each displayed frame; no Switch frame-time or heat
measurement has been made. Source review cannot certify online enforcement
risk, runtime compatibility, or frame-perfect input delivery.
