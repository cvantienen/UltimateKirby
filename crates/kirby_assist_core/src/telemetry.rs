//! Read-only Training observations; never controller commands.
//! Empty keys select explicitly unauthenticated LAN packets; nonempty keys require HMAC.
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::net::{Ipv4Addr, SocketAddrV4};

pub const FIELD_COUNT: usize = 64;
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
pub fn fields(group: u16) -> &'static [Field; FIELD_COUNT] {
    match group {
        0 => &FIELDS,
        9 => &INPUT_FIELDS,
        _ => &ATTACK_FIELDS,
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
    if sample.group > 9 {
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
    if group > 9 {
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
