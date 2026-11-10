//! Presentation-only reference hints. No policy, input or live-game queries.
use super::PresetSlot;
use crate::fighter_data::{self, MoveData};
use std::sync::OnceLock;

/// Returns the `short` result.
///
/// Gets the short result.
/// ### Arguments
/// - `text` (`&str`): Represents the text being parsed or displayed; the function uses it as input to the operation described above.
/// - `limit` (`usize`): Represents the maximum allowed value; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting text.
fn short(text: &str, limit: usize) -> String {
    let mut chars = text.chars();
    let mut result: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}
/// Returns an optional `raw` value.
///
/// Gets raw if it is available.
/// ### Arguments
/// - `data` (`&'a MoveData`): Represents the data; the function uses it as input to the operation described above.
/// - `key` (`&str`): Represents the key or identifier being sampled; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
fn raw<'a>(data: &'a MoveData, key: &str) -> Option<&'a str> {
    data.raw
        .iter()
        .find(|s| s.key == key)
        .map(|s| s.value.as_str())
        .filter(|s| !s.is_empty() && *s != "--")
}
/// Returns the `timing` result.
///
/// Gets the timing result.
/// ### Arguments
/// - `data` (`&MoveData`): Represents the data; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting text.
fn timing(data: &MoveData) -> String {
    let startup = raw(data, "startup")
        .map(|s| short(s, 22))
        .or_else(|| data.active.first().map(|s| s.first.to_string()))
        .unwrap_or_else(|| "?".into());
    let active = raw(data, "activeframes")
        .map(|s| short(s, 28))
        .unwrap_or_else(|| {
            let spans: Vec<_> = data
                .active
                .iter()
                .take(3)
                .map(|s| {
                    if s.first == s.last {
                        s.first.to_string()
                    } else {
                        format!("{}-{}", s.first, s.last)
                    }
                })
                .collect();
            if spans.is_empty() {
                "?".into()
            } else {
                spans.join(" / ")
            }
        });
    let total = raw(data, "totalframes")
        .map(|s| short(s, 22))
        .or_else(|| data.total.map(|n| n.to_string()))
        .unwrap_or_else(|| "?".into());
    let landing = data
        .landing_lag
        .map(|n| format!(" | land {n}"))
        .unwrap_or_default();
    format!("Start {startup} | active {active} | total {total}{landing} (frames)")
}

/// Returns the `move_names` result.
///
/// Gets the move names result.
/// ### Arguments
/// - `label` (`&str`): Represents the label; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `&'static [&'static str]` value.
fn move_names(label: &str) -> &'static [&'static str] {
    match label {
        "Neutral A / Nair" => &["Jab 1", "Neutral Air"],
        "Short hop Nair" => &["Neutral Air"],
        "Forward A (Fair / Fsmash)" => &["Forward Smash", "Forward Air"],
        "Up A (Uair / Usmash)" => &["Up Smash", "Up Air"],
        "Down A (Dair / Dsmash)" => &["Down Smash", "Down Air"],
        "Bair" | "Short hop Bair" => &["Back Air"],
        "Short hop Fair" => &["Forward Air"],
        "Short hop Uair" => &["Up Air"],
        "Short hop Dair" => &["Down Air"],
        "Ftilt" => &["Forward Tilt"],
        "Utilt" => &["Up Tilt"],
        "Dtilt" => &["Down Tilt"],
        "Grab / Zair" => &["Grab", "Zair"],
        "Roll forward" => &["Forward Roll"],
        "Roll backward" => &["Backward Roll"],
        "Spot dodge" => &["Spot Dodge"],
        "Neutral B" => &["Neutral B"],
        "Side B" => &["Side B"],
        "Up B" => &["Up B", "Up B, Air"],
        "Down B" => &["Down B", "Down B, Air"],
        _ => &[],
    }
}
/// Finds the move.
///
/// Finds the move.
/// ### Arguments
/// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
/// - `name` (`&str`): Represents the name being parsed, looked up, or displayed; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
fn find_move(kind: i32, name: &str) -> Option<&'static MoveData> {
    if let Some(data) = fighter_data::named_move(kind, name) {
        return Some(data);
    }
    // Named special moves use "Up B (Whirling Fortress)", etc. Never choose
    // the first of several charge/stance variants as if it were universal.
    let prefix = format!("{name} (");
    let mut matches = fighter_data::fighter(kind)?
        .moves
        .iter()
        .filter(|m| m.name.starts_with(&prefix));
    let data = matches.next()?;
    matches.next().is_none().then_some(data)
}

/// Defines the `Summary` data structure.
///
/// Holds the data for Summary.
#[derive(Clone, Copy, Default)]
struct Summary {
    /// Stores the `n` value in `Summary` as `u32`.
    ///
    ///
///  Holds the n value used by Summary.
    n: u32,
    /// Stores the `total` value in `Summary` as `u32`.
    ///
    ///
///  Holds the total value used by Summary.
    total: u32,
    /// Stores the `invulnerable` value in `Summary` as `u32`.
    ///
    ///
///  Holds the invulnerable value used by Summary.
    invulnerable: u32,
    /// Stores the `min` value in `Summary` as `u16`.
    ///
    ///
///  Holds the min value used by Summary.
    min: u16,
    /// Stores the `max` value in `Summary` as `u16`.
    ///
    ///
///  Holds the max value used by Summary.
    max: u16,
}
/// Defines the `KEYS` constant used by this module.
///
/// Provides the KEYS constant value.
const KEYS: [&str; 4] = [
    "ledgestats.neutralgetup",
    "ledgestats.roll",
    "ledgestats.jump",
    "ledgestats.attackframes",
];
/// Defines the `LEDGE` constant used by this module.
///
/// Provides the LEDGE constant value.
static LEDGE: OnceLock<[Summary; 4]> = OnceLock::new();
/// Returns an optional `ledge_reference` value.
///
/// Gets ledge reference if it is available.
/// ### Arguments
/// - `slot` (`PresetSlot`): Represents the configuration or storage slot being used; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
fn ledge_reference(slot: PresetSlot) -> Option<String> {
    let (index, name) = match slot {
        PresetSlot::Neutral => (0, "Neutral get-up"),
        PresetSlot::Roll => (1, "Ledge roll"),
        PresetSlot::Jump => (2, "Ledge jump"),
        PresetSlot::Attack => (3, "Get-up attack"),
        _ => return None,
    };
    let rows = LEDGE.get_or_init(|| {
        let mut rows = [Summary::default(); 4];
        for fighter in fighter_data::all() {
            for (index, key) in KEYS.iter().enumerate() {
                let Some((invulnerable, total)) = fighter.stat(key).and_then(ledge_frames) else {
                    continue;
                };
                let r = &mut rows[index];
                if r.n == 0 {
                    r.min = total;
                    r.max = total;
                }
                r.n += 1;
                r.total += u32::from(total);
                r.invulnerable += u32::from(invulnerable);
                r.min = r.min.min(total);
                r.max = r.max.max(total);
            }
        }
        rows
    });
    let r = rows[index];
    (r.n != 0).then(|| {
        format!(
            "{name}: roster avg {:.1}f total ({}-{}); invuln through avg {:.1}f",
            r.total as f32 / r.n as f32,
            r.min,
            r.max,
            r.invulnerable as f32 / r.n as f32
        )
    })
}
/// Returns an optional `ledge_frames` value.
///
/// Gets ledge frames if it is available.
/// ### Arguments
/// - `value` (`&str`): Represents the value being processed; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
fn ledge_frames(value: &str) -> Option<(u16, u16)> {
    let (span, rest) = value.split_once(" intangible / ")?;
    let end = span.strip_prefix("1-")?.parse::<u16>().ok()?;
    let total = rest.split_once(" total")?.0.parse::<u16>().ok()?;
    (end < total).then_some((end, total))
}

/// Returns the `hints` result.
///
/// Gets the hints result.
/// ### Arguments
/// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
/// - `slot` (`PresetSlot`): Represents the configuration or storage slot being used; the function uses it as input to the operation described above.
/// - `label` (`Option<&str>`): Represents the label; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting items; the vector is empty when there are none.
pub(super) fn hints(kind: i32, slot: PresetSlot, label: Option<&str>) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(label) = label {
        if let Some(profile) = fighter_data::fighter(kind) {
            let names = move_names(label);
            for name in names {
                if let Some(data) = find_move(kind, name) {
                    lines.push(format!("REF {} / {}", profile.name, data.name));
                    lines.push(timing(data));
                }
            }
            if !names.is_empty() && lines.is_empty() {
                lines.push(format!(
                    "REF {} / {label}: no single verified timing",
                    profile.name
                ));
            }
        }
        if label == "Neutral A / Nair" || label == "Controller input" || label == "Repeated taps" {
            lines.push(
                "Raw buttons: move depends on game state; hold time is not move startup".into(),
            );
        }
    }
    if let Some(reference) = ledge_reference(slot) {
        lines.push(reference);
        lines.push(
            "Ledge reference starts at option F1; actual invuln varies. Not a live countdown."
                .into(),
        );
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Regression test for `correct_fighter_move_and_special_variants_are_used`.
    ///
    ///
///  Checks that correct fighter move and special variants are used.
    #[test]
    fn correct_fighter_move_and_special_variants_are_used() {
        let kirby = hints(6, PresetSlot::X, Some("Dtilt")).join(" ");
        let yoshi = hints(5, PresetSlot::X, Some("Dtilt")).join(" ");
        assert!(kirby.contains("kirby / Down Tilt"));
        assert!(yoshi.contains("yoshi / Down Tilt"));
        assert_ne!(
            timing(find_move(6, "Down Tilt").unwrap()),
            timing(find_move(5, "Down Tilt").unwrap())
        );
        let bowser = hints(15, PresetSlot::X, Some("Up B")).join(" ");
        assert!(bowser.contains("Whirling Fortress"));
        assert!(bowser.contains("Air"));
    }
    /// Regression test for `roster_reference_is_labeled_and_does_not_claim_live_countdown`.
    ///
    ///
///  Checks that roster reference is labeled and does not claim live countdown.
    #[test]
    fn roster_reference_is_labeled_and_does_not_claim_live_countdown() {
        let hint = hints(6, PresetSlot::Neutral, None).join(" ");
        assert!(hint.contains("roster avg"));
        assert!(hint.contains("invuln through"));
        assert!(hint.contains("Not a live countdown"));
        assert!(hints(6, PresetSlot::Y, None).is_empty());
        assert!(hints(6, PresetSlot::Drop, None).is_empty());
        assert_eq!(ledge_frames("1-32 intangible / 33 total"), Some((32, 33)));
        assert_eq!(ledge_frames("?"), None);
    }
    /// Regression test for `input_aliases_never_claim_ground_or_air_state_is_known`.
    ///
    ///
///  Checks that input aliases never claim ground or air state is known.
    #[test]
    fn input_aliases_never_claim_ground_or_air_state_is_known() {
        let hint = hints(6, PresetSlot::X, Some("Neutral A / Nair")).join(" ");
        assert!(hint.contains("Jab 1"));
        assert!(hint.contains("Neutral Air"));
        assert!(hints(6, PresetSlot::X, Some("Controller input"))
            .join(" ")
            .contains("game state"));
    }
}
