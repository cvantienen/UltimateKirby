use hmac::{Hmac, Mac};
use kirby_assist_core::telemetry::{decode, encode, Config, Sample, FIELD_COUNT, PACKET_LEN};
use sha2::Sha256;
const KEY: &[u8] = b"0123456789abcdef0123456789abcdef";
fn sample() -> Sample {
    let mut s = Sample {
        session: u64::MAX,
        sequence: 0x0102030405060708,
        sample_us: 123456,
        object_id: 0xfedcba98,
        category: 2,
        kind: -7,
        group: 8,
        dropped: 25,
        ..Sample::default()
    };
    s.values[0] = Some(0);
    s.values[1] = Some(u64::MAX);
    s.values[2] = Some((-123i32 as i64) as u64);
    s.values[3] = Some(f32::NAN.to_bits() as u64);
    s.values[FIELD_COUNT - 1] = Some(1);
    s
}
fn resign(bytes: &mut [u8]) {
    let body = bytes.len() - 32;
    let mut mac = Hmac::<Sha256>::new_from_slice(KEY).unwrap();
    mac.update(&bytes[..body]);
    bytes[body..].copy_from_slice(&mac.finalize().into_bytes());
}
#[test]
fn unauthenticated_lan_wire_is_explicit_and_never_downgrades_keyed_mode() {
    let mut buffer = [0; 1200];
    let s = sample();
    let n = encode(&s, b"", &mut buffer).unwrap();
    assert_eq!(&buffer[4..6], &2u16.to_le_bytes());
    assert_eq!(&buffer[n - 32..n], &[0; 32]);
    assert_eq!(decode(&buffer[..n], b"").unwrap(), s);
    assert!(decode(&buffer[..n], KEY).is_err());
    buffer[n - 1] = 1;
    assert!(decode(&buffer[..n], b"").is_err());
    let n = encode(&s, KEY, &mut buffer).unwrap();
    assert!(decode(&buffer[..n], b"").is_err());
    buffer[4] = 2;
    assert!(decode(&buffer[..n], b"").is_err());
}
#[test]
fn roundtrip_preserves_validity_raw_bits_and_little_endian() {
    let s = sample();
    let mut buffer = [0u8; 1200];
    let len = encode(&s, KEY, &mut buffer).unwrap();
    assert_eq!(len, 92 + 5 * 8);
    assert!(len <= 1200);
    assert_eq!(&buffer[16..24], &[8, 7, 6, 5, 4, 3, 2, 1]);
    assert_eq!(decode(&buffer[..len], KEY).unwrap(), s);
    assert_eq!(decode(&buffer[..len], KEY).unwrap().values[4], None);
}
#[test]
fn rejects_tampering_wrong_keys_lengths_versions_and_groups() {
    let mut buffer = [0u8; 1200];
    let len = encode(&sample(), KEY, &mut buffer).unwrap();
    assert!(decode(&buffer[..len], b"another_valid_token_01234567890123").is_err());
    for i in [0, 4, 8, 52, 60, len - 1] {
        let mut changed = buffer;
        changed[i] ^= 1;
        assert!(decode(&changed[..len], KEY).is_err());
    }
    for n in [0, len - 1, len + 1, 1200] {
        assert!(decode(&buffer[..n], KEY).is_err());
    }
    let mut changed = buffer;
    changed[4] = 2;
    resign(&mut changed[..len]);
    assert!(decode(&changed[..len], KEY).is_err());
    changed = buffer;
    changed[0] = b'X';
    resign(&mut changed[..len]);
    assert!(decode(&changed[..len], KEY).is_err());
    changed = buffer;
    changed[6] = 29;
    resign(&mut changed[..len]);
    assert!(decode(&changed[..len], KEY).is_err());
    changed = buffer;
    changed[52] ^= 1 << 4;
    resign(&mut changed[..len]);
    assert!(decode(&changed[..len], KEY).is_err());
    let mut invalid = sample();
    invalid.group = 29;
    assert!(encode(&invalid, KEY, &mut buffer).is_err());
    assert!(encode(&sample(), b"short", &mut buffer).is_err());
}
#[test]
fn full_empty_and_input_packets_use_popcount_length() {
    let mut buffer = [0u8; 1200];
    let mut s = sample();
    s.values = [Some(u64::MAX); FIELD_COUNT];
    let len = encode(&s, KEY, &mut buffer).unwrap();
    assert_eq!(len, PACKET_LEN);
    assert_eq!(decode(&buffer[..len], KEY).unwrap(), s);
    s.values = [None; FIELD_COUNT];
    s.group = 9;
    s.values[12] = Some((-127i64) as u64);
    let len = encode(&s, KEY, &mut buffer).unwrap();
    assert_eq!(len, 100);
    assert_eq!(decode(&buffer[..len], KEY).unwrap(), s);
    s.values[12] = None;
    let len = encode(&s, KEY, &mut buffer).unwrap();
    assert_eq!(len, 92);
    assert_eq!(decode(&buffer[..len], KEY).unwrap(), s);
    let mut extra = buffer[..60].to_vec();
    extra.extend_from_slice(&0u64.to_le_bytes());
    extra.extend_from_slice(&[0; 32]);
    resign(&mut extra);
    assert!(decode(&extra, KEY).is_err());
}
#[test]
fn policy_observations_preserve_unknown_configuration_and_invalid_scope() {
    use kirby_assist_core::{action_owner::Owner, diagnostics::modes, telemetry::policy_sample};
    let mut snapshot = modes::Snapshot {
        winner: Some(Owner::Safety),
        present: modes::bit(Owner::Safety),
        committed: false,
        requested: 0,
        stick: Some([-1, 1]),
        status: 0,
        motion: 0,
        native_frame: 37,
        stopped: false,
        can_act: false,
        hit: None,
        valid: true,
        safety: modes::SAFETY_DI,
    };
    let mut configured = [None; 18];
    configured[3] = Some(true);
    configured[0] = Some(false);
    let sample = policy_sample(42, 6, snapshot, configured);
    assert_eq!(sample.values[16], Some(0));
    assert_eq!(sample.values[17], None);
    assert_eq!(sample.values[19], Some(1));
    assert_eq!(sample.values[7], None);
    assert_eq!(sample.values[11], Some(37));
    assert_eq!(sample.values[12], Some((-1i64) as u64));
    assert_eq!(sample.values[13], Some(1));
    assert_eq!(sample.values[1], Some(u64::from(modes::bit(Owner::Safety))));
    for key in [KEY, b"".as_slice()] {
        let mut buffer = [0; 1200];
        let length = encode(&sample, key, &mut buffer).unwrap();
        assert!(length <= PACKET_LEN);
        assert_eq!(decode(&buffer[..length], key).unwrap(), sample);
    }
    snapshot.valid = false;
    snapshot.hit = Some(true);
    let sample = policy_sample(42, 6, snapshot, configured);
    assert_eq!(sample.values[0], Some(0));
    assert!(sample.values[1..16].iter().all(Option::is_none));
    assert_eq!(sample.values[19], Some(1));
}

#[test]
fn feature_details_are_bounded_utf8_and_preserve_missing_vs_explicit_choices() {
    use kirby_assist_core::telemetry::{feature_label, FeatureDetail, POLICY_FEATURE_COUNT};
    let mut detail = FeatureDetail::new(Some(false), &"🦀".repeat(30), "None selected");
    detail.held = Some(false);
    detail.blockers = Some(16);
    detail.proposed = Some(true);
    detail.selected = Some(false);
    for feature in 0..POLICY_FEATURE_COUNT {
        let sample = detail.sample(42, 6, feature).unwrap();
        assert_eq!(feature_label(&sample, false).unwrap(), "🦀".repeat(24));
        assert_eq!(
            feature_label(&sample, true).as_deref(),
            Some("None selected")
        );
        assert_eq!(sample.values[..5], [Some(0), Some(0), Some(16), Some(1), Some(0)]);
        let mut bytes = [0; 1200];
        let size = encode(&sample, b"", &mut bytes).unwrap();
        assert!(size <= PACKET_LEN);
        assert_eq!(decode(&bytes[..size], b"").unwrap(), sample);
    }
    assert!(detail.sample(42, 6, POLICY_FEATURE_COUNT).is_none());
    let mut unknown = FeatureDetail::new(None, "", "").sample(42, 6, 0).unwrap();
    assert!(feature_label(&unknown, false).is_none());
    unknown.values[8..20].fill(Some(0));
    unknown.values[8] = Some(0xff);
    assert!(feature_label(&unknown, false).is_none());
    unknown.values[8] = Some(u64::from_le_bytes(*b"a\0\0\0\0\0\0\0"));
    unknown.values[19] = None;
    assert!(feature_label(&unknown, false).is_none());
}

#[test]
fn feature_evidence_distinguishes_safety_siblings_avoidance_and_follow_up() {
    use kirby_assist_core::{action_owner::Owner, diagnostics::modes, telemetry::FeatureDetail};
    let mut snapshot = modes::Snapshot {
        winner: Some(Owner::Safety), present: modes::bit(Owner::Safety),
        committed: false, requested: 0, stick: None, status: 0, motion: 0,
        native_frame: 0, stopped: true, can_act: false, hit: None,
        valid: true, safety: modes::SAFETY_DI,
    };
    let mut detail = FeatureDetail::new(Some(true), "", "");
    for feature in 0..4 {
        detail.observe_policy(snapshot, feature);
        assert_eq!(detail.proposed, Some(feature == 3));
        assert_eq!(detail.selected, Some(feature == 3));
        assert_eq!(detail.blockers, Some(0));
    }
    snapshot.winner = Some(Owner::GrabDodge);
    snapshot.present = modes::bit(Owner::GrabDodge);
    detail.observe_policy(snapshot, 14);
    assert_eq!(detail.selected, Some(true));
    detail.observe_policy(snapshot, 4);
    assert_eq!(detail.proposed, Some(false));
    snapshot.present |= modes::bit(Owner::OptionRead);
    detail.observe_policy(snapshot, 4);
    assert_eq!(detail.proposed, Some(true));
    assert_eq!(detail.selected, Some(false));
    assert_eq!(detail.blockers, Some(8));
    snapshot.valid = false;
    detail.held = Some(false);
    detail.observe_policy(snapshot, 4);
    assert_eq!(detail.proposed, None);
    assert_eq!(detail.selected, None);
    assert_eq!(detail.blockers, Some(17));
}

fn config(switch: &str, computer: &str, port: &str, token: &str) -> String {
    format!("# local trusted LAN\nswitch_ip={switch}\ncomputer_ip={computer}\nport={port}\ntoken={token}\n")
}
#[test]
fn config_accepts_private_addresses_and_boundary_ports() {
    for port in ["1", "65535"] {
        let c = Config::parse(&config(
            "192.168.1.10",
            "192.168.1.20",
            port,
            std::str::from_utf8(KEY).unwrap(),
        ))
        .unwrap();
        assert_eq!(c.switch.port(), c.computer.port());
        assert_eq!(c.token.as_bytes(), KEY);
    }
    assert!(Config::parse(&config(
        "10.0.0.1",
        "172.31.255.254",
        "9999",
        &"_".repeat(128)
    ))
    .is_ok());
}
#[test]
fn config_size_limit_matches_native_setup_admission() {
    let mut source = config(
        "10.0.0.1",
        "10.0.0.2",
        "1234",
        std::str::from_utf8(KEY).unwrap(),
    );
    source.push('#');
    source.extend(std::iter::repeat('x').take(2048 - source.len()));
    assert!(Config::parse(&source).is_ok());
    source.push('x');
    assert_eq!(
        Config::parse(&source),
        Err("Telemetry setup exceeds 2048 bytes")
    );
}

#[test]
fn config_rejects_unsafe_ambiguous_and_incomplete_inputs() {
    let key = std::str::from_utf8(KEY).unwrap();
    for ip in [
        "8.8.8.8",
        "127.0.0.1",
        "0.0.0.0",
        "224.0.0.1",
        "255.255.255.255",
        "192.168.1.255",
        "192.168.1.0",
        "169.254.1.1",
        "localhost",
        "192.168.001.10",
    ] {
        assert!(
            Config::parse(&config(ip, "192.168.1.20", "1234", key)).is_err(),
            "{ip}"
        );
    }
    assert!(Config::parse(&config("192.168.1.20", "192.168.1.20", "1234", key)).is_err());
    for port in ["0", "65536", "-1", "+1", "1.5", ""] {
        assert!(Config::parse(&config("10.0.0.1", "10.0.0.2", port, key)).is_err());
    }
    for token in [
        "short".to_string(),
        "a".repeat(129),
        "=".repeat(32),
        "é".repeat(32),
    ] {
        assert!(Config::parse(&config("10.0.0.1", "10.0.0.2", "1234", &token)).is_err());
    }
    let valid = config("10.0.0.1", "10.0.0.2", "1234", key);
    for extra in ["port=1234", "unknown=hello", "malformed"] {
        assert!(Config::parse(&format!("{valid}{extra}\n")).is_err());
    }
    assert!(Config::parse("switch_ip=10.0.0.1\n").is_err());
}
