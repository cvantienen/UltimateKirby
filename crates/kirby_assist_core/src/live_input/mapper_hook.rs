//! Startup-only recognition of the FIM mapper, including Skyline entry patches.
//! No fixed game address, pointer dereference, or instruction modification here.

pub const MAPPING_NEEDLE: &[u8] = &[
    0xff, 0x03, 0x02, 0xd1, 0xf7, 0x23, 0x00, 0xf9, 0xf6, 0x57, 0x05, 0xa9, 0xf4, 0x4f, 0x06, 0xa9,
    0xfd, 0x7b, 0x07, 0xa9, 0xfd, 0xc3, 0x01, 0x91, 0x3f, 0x04, 0x00, 0x31, 0xe0, 0x77, 0x00, 0x54,
];

fn instruction(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

/// Accept only the original entry or Skyline's B / LDR+BR entry replacements.
/// The original bytes after the replaced region must still match. Reject two
/// candidates even if one is unpatched: silently choosing one is unsafe.
pub fn mapper_offset(text: &[u8]) -> Option<usize> {
    if text.len() < MAPPING_NEEDLE.len() {
        return None;
    }
    let mut found = None;
    for offset in (0..=text.len() - MAPPING_NEEDLE.len()).step_by(4) {
        let entry = &text[offset..offset + MAPPING_NEEDLE.len()];
        // Skyline overwrites at most five instructions. Filter by the intact
        // suffix first, then validate the exact layout and remaining prologue.
        if entry[20..] != MAPPING_NEEDLE[20..] {
            continue;
        }
        let original = entry == MAPPING_NEEDLE;
        let branch =
            instruction(entry, 0) & 0xfc00_0000 == 0x1400_0000 && entry[4..] == MAPPING_NEEDLE[4..];
        let absolute = instruction(entry, 0) == 0x5800_0051
            && instruction(entry, 4) == 0xd61f_0220
            && instruction(entry, 8) as u64 | ((instruction(entry, 12) as u64) << 32) != 0
            && entry[16..] == MAPPING_NEEDLE[16..];
        let aligned_absolute = instruction(entry, 0) == 0xd503_201f
            && instruction(entry, 4) == 0x5800_0051
            && instruction(entry, 8) == 0xd61f_0220
            && instruction(entry, 12) as u64 | ((instruction(entry, 16) as u64) << 32) != 0;
        if original || branch || absolute || aligned_absolute {
            if found.is_some() {
                return None;
            }
            found = Some(offset);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patched(words: &[u32]) -> Vec<u8> {
        let mut bytes = MAPPING_NEEDLE.to_vec();
        for (i, word) in words.iter().enumerate() {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn recognizes_original_and_all_skyline_entry_layouts() {
        for entry in [
            MAPPING_NEEDLE.to_vec(),
            patched(&[0x1400_0100]),
            patched(&[0x5800_0051, 0xd61f_0220, 0x1234_5000, 0x71]),
            patched(&[0xd503_201f, 0x5800_0051, 0xd61f_0220, 0x1234_5000, 0x71]),
        ] {
            let mut text = vec![0; 12];
            text.extend(entry);
            assert_eq!(mapper_offset(&text), Some(12));
        }
    }

    #[test]
    fn rejects_ambiguity_damage_null_target_and_unaligned_matches() {
        let mut duplicate = MAPPING_NEEDLE.to_vec();
        duplicate.extend(patched(&[0x1400_0100]));
        assert_eq!(mapper_offset(&duplicate), None);
        let mut damaged = patched(&[0x1400_0100]);
        damaged[24] ^= 1;
        assert_eq!(mapper_offset(&damaged), None);
        assert_eq!(
            mapper_offset(&patched(&[0x5800_0051, 0xd61f_0220, 0, 0])),
            None
        );
        let mut unaligned = vec![0];
        unaligned.extend(MAPPING_NEEDLE);
        assert_eq!(mapper_offset(&unaligned), None);
        assert_eq!(mapper_offset(&[]), None);
    }
}
