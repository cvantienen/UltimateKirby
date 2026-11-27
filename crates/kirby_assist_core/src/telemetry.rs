//! Read-only Training observations; never controller commands.
//! Empty keys select explicitly unauthenticated LAN packets; nonempty keys require HMAC.
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::net::{Ipv4Addr, SocketAddrV4};

pub const FIELD_COUNT: usize = 64;
/// Additive observation group; older receivers reject it without misdecoding it.
pub const POLICY_GROUP: u16 = 10;
pub const POLICY_FEATURE_COUNT: usize = 18;
pub const FEATURE_GROUP_START: u16 = 11;
pub const MAX_GROUP: u16 = FEATURE_GROUP_START + POLICY_FEATURE_COUNT as u16 - 1;
const LABEL_BYTES: usize = 96;

/// Fixed-size copied labels: no native pointers, allocations, or arbitrary queries.
#[derive(Clone, Copy, Debug)]
pub struct FeatureDetail {
    pub configured: Option<bool>,
    pub held: Option<bool>,
    pub blockers: Option<u64>,
    mode: [u8; LABEL_BYTES],
    recipe: [u8; LABEL_BYTES],
}
impl FeatureDetail {
    pub fn new(configured: Option<bool>, mode: &str, recipe: &str) -> Self {
        fn label(text: &str) -> [u8; LABEL_BYTES] {
            let mut end = text.len().min(LABEL_BYTES);
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            let mut value = [0; LABEL_BYTES];
            value[..end].copy_from_slice(&text.as_bytes()[..end]);
            value
        }
        Self {
            configured,
            held: None,
            blockers: None,
            mode: label(mode),
            recipe: label(recipe),
        }
    }
    pub fn sample(self, object_id: u32, kind: i32, feature: usize) -> Option<Sample> {
        if feature >= POLICY_FEATURE_COUNT {
            return None;
        }
        let mut sample = Sample {
            object_id,
            kind,
            group: FEATURE_GROUP_START + feature as u16,
            ..Sample::default()
        };
        sample.values[0] = self.configured.map(u64::from);
        sample.values[1] = self.held.map(u64::from);
        sample.values[2] = self.blockers;
        for (offset, bytes) in [(8, self.mode), (20, self.recipe)] {
            if bytes[0] == 0 {
                continue;
            }
            for (index, word) in bytes.chunks_exact(8).enumerate() {
                sample.values[offset + index] = Some(u64::from_le_bytes(word.try_into().unwrap()));
            }
        }
        Some(sample)
    }
}
/// Decode only complete, valid UTF-8 copied labels; malformed/missing is unknown.
pub fn feature_label(sample: &Sample, recipe: bool) -> Option<String> {
    if !(FEATURE_GROUP_START..=MAX_GROUP).contains(&sample.group) {
        return None;
    }
    let offset = if recipe { 20 } else { 8 };
    let mut bytes = [0; LABEL_BYTES];
    for (i, word) in sample.values[offset..offset + LABEL_BYTES / 8]
        .iter()
        .enumerate()
    {
        bytes[i * 8..i * 8 + 8].copy_from_slice(&(*word)?.to_le_bytes());
    }
    let end = bytes.iter().position(|v| *v == 0).unwrap_or(LABEL_BYTES);
    if end == 0 {
        return None;
    }
    std::str::from_utf8(&bytes[..end]).ok().map(str::to_owned)
}
/// Maximum encoded length; sparse packets use 92 + 8 bytes per present field.
pub const PACKET_LEN: usize = 604;
const MAGIC: &[u8; 4] = b"KATM";
const VERSION: u16 = 1;
const LAN_VERSION: u16 = 2;
const TAG_LEN: usize = 32;
pub const SUBSCRIPTION: &[u8] = b"KATM SUBSCRIBE 1";
pub const TELEMETRY_PORT: u16 = 4243;
pub const SUBSCRIBER_LEASE: std::time::Duration = std::time::Duration::from_secs(3);

pub fn private_peer(peer: SocketAddrV4) -> bool {
    peer.port() != 0 && peer.ip().is_private() && !matches!(peer.ip().octets()[3], 0 | 255)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub session: u64,
    pub sequence: u64,
    pub sample_us: u64,
    pub object_id: u32,
    pub category: u32,
    pub kind: i32,
    pub group: u16,
    pub dropped: u64,
    pub values: [Option<u64>; FIELD_COUNT],
}

impl Default for Sample {
    fn default() -> Self {
        Self {
            session: 0,
            sequence: 0,
            sample_us: 0,
            object_id: 0,
            category: 0,
            kind: 0,
            group: 0,
            dropped: 0,
            values: [None; FIELD_COUNT],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    Float,
    Signed,
    Unsigned,
    Boolean,
    Hex,
}
#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub name: &'static str,
    pub kind: ValueKind,
    pub source: &'static str,
}
macro_rules! field {
    ($name:literal, $kind:ident) => {
        Field {
            name: $name,
            kind: ValueKind::$kind,
            source: "native read-only observation",
        }
    };
}
pub const FIELDS: [Field; FIELD_COUNT] = [
    field!("status", Signed),
    field!("previous_status", Signed),
    field!("situation", Signed),
    field!("motion", Hex),
    field!("motion_frame", Float),
    field!("motion_rate", Float),
    field!("motion_end_frame", Float),
    field!("position_x", Float),
    field!("position_y", Float),
    field!("position_z", Float),
    field!("facing", Float),
    field!("scale", Float),
    field!("velocity_x", Float),
    field!("velocity_y", Float),
    field!("team", Unsigned),
    field!("team_owner", Unsigned),
    field!("stopped", Boolean),
    field!("damage_stopped", Boolean),
    field!("hitstop_remaining", Unsigned),
    field!("damage_percent", Float),
    field!("hit_status", Unsigned),
    field!("cancel_enabled", Boolean),
    field!("cancel_frame", Float),
    field!("normal_hit_frame", Float),
    field!("passable_ground", Boolean),
    field!("capture_link", Boolean),
    field!("capture_stop_link", Boolean),
    field!("clatter_time", Float),
    field!("hit_infliction", Boolean),
    Field {
        name: "shield_infliction",
        kind: ValueKind::Boolean,
        source: "unavailable: native getter not verified",
    },
    field!("attack_active_mask", Hex),
    field!("air_accel_y", Float),
    field!("air_speed_y_stable", Float),
    field!("ground_brake", Float),
    field!("damage_launch_x", Float),
    field!("damage_launch_y", Float),
    RESERVED,
    field!("shield_stun_frames", Signed),
    field!("cliff_hit_normal_frame", Float),
    field!("reaction_frames", Float),
    field!("passive_disabled", Boolean),
    field!("can_passive", Boolean),
    field!("can_passive_fb", Boolean),
    field!("can_passive_wall", Boolean),
    Field {
        name: "can_passive_wall_jump",
        kind: ValueKind::Boolean,
        source: "unavailable: native getter not verified",
    },
    field!("can_passive_ceiling", Boolean),
    field!("can_catch", Boolean),
    field!("can_jump_squat", Boolean),
    field!("can_guard", Boolean),
    field!("can_escape_air", Boolean),
    field!("game_guard", Boolean),
    field!("game_guard_hold", Boolean),
    field!("damage_fly_top_gravity", Float),
    field!("damage_fly_top_terminal_speed", Float),
    field!("walk_stick_x", Float),
    field!("dash_stick_x", Float),
    field!("gamewatch_oil_gauge", Float),
    field!("last_attacker_entry", Signed),
    field!("capture_object", Hex),
    field!("can_jump_squat_native", Boolean),
    field!("can_attack", Boolean),
    field!("can_special_lw", Boolean),
    Field {
        name: "fighter_roster_capacity",
        kind: ValueKind::Unsigned,
        source: "telemetry coverage limit",
    },
    Field {
        name: "projectile_capacity",
        kind: ValueKind::Unsigned,
        source: "telemetry coverage limit",
    },
];
const RESERVED: Field = Field {
    name: "reserved",
    kind: ValueKind::Hex,
    source: "reserved",
};
pub const ATTACK_FIELDS: [Field; FIELD_COUNT] = {
    let mut f = [RESERVED; FIELD_COUNT];
    f[0] = field!("active", Boolean);
    f[1] = field!("node", Hex);
    f[2] = field!("local_x", Float);
    f[3] = field!("local_y", Float);
    f[4] = field!("local_z", Float);
    f[5] = field!("local_x2", Float);
    f[6] = field!("local_y2", Float);
    f[7] = field!("local_z2", Float);
    f[8] = field!("radius", Float);
    f[9] = field!("power", Float);
    f[10] = field!("angle", Signed);
    f[11] = field!("kbg", Signed);
    f[12] = field!("fkb", Signed);
    f[13] = field!("bkb", Signed);
    f[14] = field!("hitlag", Float);
    f[15] = field!("sdi", Float);
    f[17] = field!("slip", Float);
    f[23] = field!("check_type", Unsigned);
    f[24] = field!("situation_mask", Unsigned);
    f[25] = field!("world_start_x", Float);
    f[26] = field!("world_start_y", Float);
    f[27] = field!("world_start_z", Float);
    f[28] = field!("world_end_x", Float);
    f[29] = field!("world_end_y", Float);
    f[30] = field!("world_end_z", Float);
    f[31] = field!("world_radius", Float);
    f
};
pub const INPUT_FIELDS: [Field; FIELD_COUNT] = {
    let mut f = [RESERVED; FIELD_COUNT];
    f[0] = field!("physical_held", Hex);
    f[1] = field!("physical_previous", Hex);
    f[2] = field!("physical_pressed", Hex);
    f[3] = field!("physical_released", Hex);
    f[4] = field!("left_stick_x", Float);
    f[5] = field!("left_stick_y", Float);
    f[6] = field!("right_stick_x", Float);
    f[7] = field!("right_stick_y", Float);
    f[8] = field!("left_trigger", Float);
    f[9] = field!("right_trigger", Float);
    f[10] = field!("physical_autorepeat", Hex);
    f[11] = field!("mapped_buttons", Hex);
    f[12] = field!("mapped_left_x", Signed);
    f[13] = field!("mapped_left_y", Signed);
    f[14] = field!("mapped_right_x", Signed);
    f[15] = field!("mapped_right_y", Signed);
    f
};
/// Configured preferences and current arbitration evidence are separate observations.
/// Neither a proposal nor a selected owner proves the game accepted an action.
pub const POLICY_FIELDS: [Field; FIELD_COUNT] = {
    let mut f = [RESERVED; FIELD_COUNT];
    f[0] = Field {
        name: "observation_valid",
        kind: ValueKind::Boolean,
        source: "fresh mapper policy scope; not a delivery receipt",
    };
    f[1] = Field {
        name: "selected_owner",
        kind: ValueKind::Hex,
        source: "arbitration owner bit; zero means no selected owner",
    };
    f[2] = Field {
        name: "proposed_owners",
        kind: ValueKind::Hex,
        source: "arbitration proposal mask; not delivered input",
    };
    f[3] = Field {
        name: "committed",
        kind: ValueKind::Boolean,
        source: "selected policy owns a committed input sequence",
    };
    f[4] = Field {
        name: "requested_buttons",
        kind: ValueKind::Hex,
        source: "policy request before final mapper delivery",
    };
    f[5] = Field {
        name: "safety_proposals",
        kind: ValueKind::Hex,
        source: "tech, mash, SDI, DI proposals; not delivery receipts",
    };
    f[6] = Field {
        name: "can_act",
        kind: ValueKind::Boolean,
        source: "policy movement readiness; not every move is legal",
    };
    f[7] = Field {
        name: "hit_confirmed",
        kind: ValueKind::Boolean,
        source: "optional policy hit observation; absent means unknown",
    };
    f[8] = Field {
        name: "status",
        kind: ValueKind::Signed,
        source: "native status observed by mapper policy",
    };
    f[9] = Field {
        name: "motion",
        kind: ValueKind::Hex,
        source: "native motion observed by mapper policy",
    };
    f[10] = Field {
        name: "stopped",
        kind: ValueKind::Boolean,
        source: "native animation stop observed by mapper policy",
    };
    let names = [
        "auto_tech",
        "auto_mash",
        "auto_sdi",
        "auto_di",
        "dodge_follow_up",
        "ledge_trap",
        "roll_read",
        "trip_follow_up",
        "fair_follow_up",
        "ledge_trump",
        "defense",
        "turbo",
        "posture",
        "out_of_shield",
        "grab_dodge",
        "reflect",
        "counter",
        "spacing",
    ];
    let mut i = 0;
    while i < POLICY_FEATURE_COUNT {
        f[16 + i] = Field {
            name: names[i],
            kind: ValueKind::Boolean,
            source: "configured preference; not active or delivered; absent means unknown",
        };
        i += 1;
    }
    f
};

pub fn policy_sample(
    object_id: u32,
    kind: i32,
    snapshot: crate::diagnostics::modes::Snapshot,
    configured: [Option<bool>; POLICY_FEATURE_COUNT],
) -> Sample {
    let mut sample = Sample {
        object_id,
        kind,
        group: POLICY_GROUP,
        ..Sample::default()
    };
    let v = &mut sample.values;
    v[0] = Some(u64::from(snapshot.valid));
    // Invalid policy scope must not turn stale/absent context into confirmed facts.
    if snapshot.valid {
        v[1] = Some(
            snapshot
                .winner
                .map_or(0, |o| u64::from(crate::diagnostics::modes::bit(o))),
        );
        v[2] = Some(u64::from(snapshot.present));
        v[3] = Some(u64::from(snapshot.committed));
        v[4] = Some(u64::from(snapshot.requested));
        v[5] = Some(u64::from(snapshot.safety));
        v[6] = Some(u64::from(snapshot.can_act));
        v[7] = snapshot.hit.map(u64::from);
        v[8] = Some(snapshot.status as i64 as u64);
        v[9] = Some(snapshot.motion);
        v[10] = Some(u64::from(snapshot.stopped));
    }
    for (i, configured) in configured.into_iter().enumerate() {
        v[16 + i] = configured.map(u64::from);
    }
    sample
}

pub const FEATURE_FIELDS: [Field; FIELD_COUNT] = {
    let mut f = [RESERVED; FIELD_COUNT];
    f[0] = Field {
        name: "configured",
        kind: ValueKind::Boolean,
        source: "effective preference; not active or delivered",
    };
    f[1] = Field {
        name: "hold_trigger",
        kind: ValueKind::Boolean,
        source: "observed hold trigger; absent if not applicable or unknown",
    };
    f[2] = Field {
        name: "observed_constraints",
        kind: ValueKind::Hex,
        source:
            "scope invalid, stopped, not ready, other owner, hold released; not exhaustive causes",
    };
    let mut i = 8;
    while i < 32 {
        f[i] = Field {
            name: "label_utf8_word",
            kind: ValueKind::Hex,
            source: "bounded copied UTF-8 label; mode words8–19, recipe words20–31",
        };
        i += 1;
    }
    f
};

pub fn fields(group: u16) -> &'static [Field; FIELD_COUNT] {
    match group {
        0 => &FIELDS,
        1..=8 => &ATTACK_FIELDS,
        9 => &INPUT_FIELDS,
        POLICY_GROUP => &POLICY_FIELDS,
        FEATURE_GROUP_START..=MAX_GROUP => &FEATURE_FIELDS,
        _ => &[RESERVED; FIELD_COUNT],
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub switch: SocketAddrV4,
    pub computer: SocketAddrV4,
    pub token: String,
}
fn valid_token(key: &[u8]) -> bool {
    (32..=128).contains(&key.len())
        && key
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-')
}
fn private_ip(value: &str) -> Result<Ipv4Addr, &'static str> {
    let ip: Ipv4Addr = value.parse().map_err(|_| "invalid numeric IPv4 address")?;
    // Conservatively exclude subnet boundary addresses as well as non-private ranges.
    if !ip.is_private() || matches!(ip.octets()[3], 0 | 255) {
        return Err("expected private unicast IPv4 address");
    }
    Ok(ip)
}
impl Config {
    pub fn parse(source: &str) -> Result<Self, &'static str> {
        if source.len() > 2048 {
            return Err("Telemetry setup exceeds 2048 bytes");
        }
        let mut values: [Option<&str>; 4] = [None; 4];
        for line in source.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (name, value) = line.split_once('=').ok_or("expected key=value")?;
            let index = match name.trim() {
                "switch_ip" => 0,
                "computer_ip" => 1,
                "port" => 2,
                "token" => 3,
                _ => return Err("unknown config field"),
            };
            if values[index].replace(value.trim()).is_some() {
                return Err("duplicate config field");
            }
        }
        let [switch, computer, port, token] = values;
        let switch = private_ip(switch.ok_or("missing switch_ip")?)?;
        let computer = private_ip(computer.ok_or("missing computer_ip")?)?;
        if switch == computer {
            return Err("switch and computer must differ");
        }
        let port = port.ok_or("missing port")?;
        if !port.bytes().all(|b| b.is_ascii_digit()) {
            return Err("invalid port");
        }
        let port: u16 = port.parse().map_err(|_| "invalid port")?;
        if port == 0 {
            return Err("invalid port");
        }
        let token = token.ok_or("missing token")?;
        if !valid_token(token.as_bytes()) {
            return Err("invalid token");
        }
        Ok(Self {
            switch: SocketAddrV4::new(switch, port),
            computer: SocketAddrV4::new(computer, port),
            token: token.into(),
        })
    }
}

pub fn encode(sample: &Sample, key: &[u8], out: &mut [u8; 1200]) -> Result<usize, &'static str> {
    if !key.is_empty() && !valid_token(key) {
        return Err("invalid token");
    }
    if sample.group > MAX_GROUP {
        return Err("invalid group");
    }
    let mut pos = 0;
    macro_rules! put {
        ($bytes:expr) => {{
            let bytes = $bytes;
            out[pos..pos + bytes.len()].copy_from_slice(&bytes);
            pos += bytes.len();
        }};
    }
    put!(*MAGIC);
    put!((if key.is_empty() { LAN_VERSION } else { VERSION }).to_le_bytes());
    put!(sample.group.to_le_bytes());
    put!(sample.session.to_le_bytes());
    put!(sample.sequence.to_le_bytes());
    put!(sample.sample_us.to_le_bytes());
    put!(sample.object_id.to_le_bytes());
    put!(sample.category.to_le_bytes());
    put!(sample.kind.to_le_bytes());
    put!(sample.dropped.to_le_bytes());
    let valid = sample
        .values
        .iter()
        .enumerate()
        .fold(0u64, |bits, (i, value)| {
            bits | (u64::from(value.is_some()) << i)
        });
    put!(valid.to_le_bytes());
    for value in sample.values.into_iter().flatten() {
        put!(value.to_le_bytes());
    }
    if key.is_empty() {
        put!([0u8; TAG_LEN]);
    } else {
        let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| "invalid key")?;
        mac.update(&out[..pos]);
        put!(mac.finalize().into_bytes());
    }
    debug_assert_eq!(pos, 60 + valid.count_ones() as usize * 8 + TAG_LEN);
    Ok(pos)
}

pub fn decode(bytes: &[u8], key: &[u8]) -> Result<Sample, &'static str> {
    if !(92..=PACKET_LEN).contains(&bytes.len()) {
        return Err("invalid packet length");
    }
    let valid = u64::from_le_bytes(bytes[52..60].try_into().map_err(|_| "invalid validity")?);
    if bytes.len() != 60 + valid.count_ones() as usize * 8 + TAG_LEN {
        return Err("invalid packet length");
    }
    if !key.is_empty() && !valid_token(key) {
        return Err("invalid token");
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let expected_version = if key.is_empty() { LAN_VERSION } else { VERSION };
    if version != expected_version {
        return Err("unsupported version");
    }
    let body = bytes.len() - TAG_LEN;
    if key.is_empty() {
        if bytes[body..].iter().any(|b| *b != 0) {
            return Err("invalid unauthenticated tag");
        }
    } else {
        let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| "invalid key")?;
        mac.update(&bytes[..body]);
        mac.verify_slice(&bytes[body..])
            .map_err(|_| "authentication failed")?;
    }
    if &bytes[..4] != MAGIC {
        return Err("invalid magic");
    }
    let mut pos = 4;
    macro_rules! read {
        ($ty:ty, $n:expr) => {{
            let mut raw = [0u8; $n];
            raw.copy_from_slice(&bytes[pos..pos + $n]);
            pos += $n;
            <$ty>::from_le_bytes(raw)
        }};
    }
    if read!(u16, 2) != expected_version {
        return Err("unsupported version");
    }
    let group = read!(u16, 2);
    if group > MAX_GROUP {
        return Err("invalid group");
    }
    let session = read!(u64, 8);
    let sequence = read!(u64, 8);
    let sample_us = read!(u64, 8);
    let object_id = read!(u32, 4);
    let category = read!(u32, 4);
    let kind = read!(i32, 4);
    let dropped = read!(u64, 8);
    let valid = read!(u64, 8);
    let mut values = [None; FIELD_COUNT];
    for (i, value) in values.iter_mut().enumerate() {
        if valid & (1 << i) != 0 {
            *value = Some(read!(u64, 8));
        }
    }
    debug_assert_eq!(pos, body);
    Ok(Sample {
        session,
        sequence,
        sample_us,
        object_id,
        category,
        kind,
        group,
        dropped,
        values,
    })
}
