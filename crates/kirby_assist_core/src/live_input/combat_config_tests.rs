//! Existing regression cases for `live_input`. Kept outside the runtime code.
use super::LiveConfig;
#[test]
fn trip_and_sdi_defaults_disable_independently_and_reject_bad_values() {
    let defaults = LiveConfig::parse("").unwrap();
    assert!(defaults.down_tilt_trip_smash && defaults.auto_sdi);
    let disabled = LiveConfig::parse("down_tilt_trip_smash = false\nauto_sdi = false").unwrap();
    assert!(!disabled.down_tilt_trip_smash && !disabled.auto_sdi);
    assert!(LiveConfig::parse("auto_sdi = fastest").is_err());
    assert!(LiveConfig::parse("down_tilt_trip_smash = maybe").is_err());
}
