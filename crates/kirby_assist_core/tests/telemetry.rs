#![cfg(feature = "diagnostic-logging")]
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
    changed[6] = 10;
    resign(&mut changed[..len]);
    assert!(decode(&changed[..len], KEY).is_err());
    changed = buffer;
    changed[52] ^= 1 << 4;
    resign(&mut changed[..len]);
    assert!(decode(&changed[..len], KEY).is_err());
    let mut invalid = sample();
    invalid.group = 10;
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
