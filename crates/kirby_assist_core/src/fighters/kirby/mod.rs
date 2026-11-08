//! Kirby-specific input-only follow-ups.

mod fair_followup;
mod trip_followup;

pub use fair_followup::{FairContext, FairFollowup};
pub use trip_followup::{TripContext, TripFollowup, TripTarget};

// Follow-ups also suppress taunts; Plus/Minus remain available.
const FOLLOWUP_BUTTONS: u32 =
    crate::native_input::ASSIST_BUTTONS | crate::native_input::MAPPER_DPAD_BUTTONS;
