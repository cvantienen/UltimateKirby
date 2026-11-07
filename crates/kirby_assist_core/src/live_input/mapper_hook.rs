//! Startup-only recognition of the FIM mapper, including Skyline entry patches.
//! No fixed game address, pointer dereference, or instruction modification here.

pub const MAPPING_NEEDLE: &[u8] = &[
    0xff, 0x03, 0x02, 0xd1, 0xf7, 0x23, 0x00, 0xf9, 0xf6, 0x57, 0x05, 0xa9, 0xf4, 0x4f, 0x06, 0xa9,
    0xfd, 0x7b, 0x07, 0xa9, 0xfd, 0xc3, 0x01, 0x91, 0x3f, 0x04, 0x00, 0x31, 0xe0, 0x77, 0x00, 0x54,
];

const AARCH64_INSTRUCTION_BYTES: usize = 4;
const AARCH64_INSTRUCTION_WORD_BITS: u32 = 32;
const PATCHED_ENTRY_PREFIX_BYTES: usize = 5 * AARCH64_INSTRUCTION_BYTES;
const ENTRY_INSTRUCTION_0_OFFSET_BYTES: usize = 0;
const ENTRY_INSTRUCTION_1_OFFSET_BYTES: usize = AARCH64_INSTRUCTION_BYTES;
const ENTRY_INSTRUCTION_2_OFFSET_BYTES: usize = 2 * AARCH64_INSTRUCTION_BYTES;
const ENTRY_INSTRUCTION_3_OFFSET_BYTES: usize = 3 * AARCH64_INSTRUCTION_BYTES;
const ENTRY_INSTRUCTION_4_OFFSET_BYTES: usize = 4 * AARCH64_INSTRUCTION_BYTES;
const BRANCH_OPCODE_MASK: u32 = 0xfc00_0000;
const BRANCH_OPCODE: u32 = 0x1400_0000;
const LOAD_X17_LITERAL_OPCODE: u32 = 0x5800_0051;
const BRANCH_X17_OPCODE: u32 = 0xd61f_0220;
const NO_OPERATION_OPCODE: u32 = 0xd503_201f;
#[cfg(test)]
const TEST_BRANCH_PATCH_WORD: u32 = 0x1400_0100;
#[cfg(test)]
const TEST_TARGET_LOW_WORD: u32 = 0x1234_5000;
#[cfg(test)]
const TEST_TARGET_HIGH_WORD: u32 = 0x71;

fn instruction(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + AARCH64_INSTRUCTION_BYTES]
            .try_into()
            .unwrap(),
    )
}

/// Accept only the original entry or Skyline's B / LDR+BR entry replacements.
/// The original bytes after the replaced region must still match. Reject two
/// candidates even if one is unpatched: silently choosing one is unsafe.
pub fn mapper_offset(text: &[u8]) -> Option<usize> {
    if text.len() < MAPPING_NEEDLE.len() {
        return None;
    }
    let mut found = None;
    for offset in (0..=text.len() - MAPPING_NEEDLE.len()).step_by(AARCH64_INSTRUCTION_BYTES) {
        let entry = &text[offset..offset + MAPPING_NEEDLE.len()];
        // Skyline overwrites at most five instructions. Filter by the intact
        // suffix first, then validate the exact layout and remaining prologue.
        if entry[PATCHED_ENTRY_PREFIX_BYTES..] != MAPPING_NEEDLE[PATCHED_ENTRY_PREFIX_BYTES..] {
            continue;
        }
        let original = entry == MAPPING_NEEDLE;
        let branch = instruction(entry, ENTRY_INSTRUCTION_0_OFFSET_BYTES) & BRANCH_OPCODE_MASK
            == BRANCH_OPCODE
            && entry[ENTRY_INSTRUCTION_1_OFFSET_BYTES..]
                == MAPPING_NEEDLE[ENTRY_INSTRUCTION_1_OFFSET_BYTES..];
        let absolute = instruction(entry, ENTRY_INSTRUCTION_0_OFFSET_BYTES)
            == LOAD_X17_LITERAL_OPCODE
            && instruction(entry, ENTRY_INSTRUCTION_1_OFFSET_BYTES) == BRANCH_X17_OPCODE
            && instruction(entry, ENTRY_INSTRUCTION_2_OFFSET_BYTES) as u64
                | ((instruction(entry, ENTRY_INSTRUCTION_3_OFFSET_BYTES) as u64)
                    << AARCH64_INSTRUCTION_WORD_BITS)
                != 0
            && entry[ENTRY_INSTRUCTION_4_OFFSET_BYTES..]
                == MAPPING_NEEDLE[ENTRY_INSTRUCTION_4_OFFSET_BYTES..];
        let aligned_absolute = instruction(entry, ENTRY_INSTRUCTION_0_OFFSET_BYTES)
            == NO_OPERATION_OPCODE
            && instruction(entry, ENTRY_INSTRUCTION_1_OFFSET_BYTES) == LOAD_X17_LITERAL_OPCODE
            && instruction(entry, ENTRY_INSTRUCTION_2_OFFSET_BYTES) == BRANCH_X17_OPCODE
            && instruction(entry, ENTRY_INSTRUCTION_3_OFFSET_BYTES) as u64
                | ((instruction(entry, ENTRY_INSTRUCTION_4_OFFSET_BYTES) as u64)
                    << AARCH64_INSTRUCTION_WORD_BITS)
                != 0;
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
            patched(&[TEST_BRANCH_PATCH_WORD]),
            patched(&[
                LOAD_X17_LITERAL_OPCODE,
                BRANCH_X17_OPCODE,
                TEST_TARGET_LOW_WORD,
                TEST_TARGET_HIGH_WORD,
            ]),
            patched(&[
                NO_OPERATION_OPCODE,
                LOAD_X17_LITERAL_OPCODE,
                BRANCH_X17_OPCODE,
                TEST_TARGET_LOW_WORD,
                TEST_TARGET_HIGH_WORD,
            ]),
        ] {
            let mut text = vec![0; 12];
            text.extend(entry);
            assert_eq!(mapper_offset(&text), Some(12));
        }
    }

    #[test]
    fn rejects_ambiguity_damage_null_target_and_unaligned_matches() {
        let mut duplicate = MAPPING_NEEDLE.to_vec();
        duplicate.extend(patched(&[TEST_BRANCH_PATCH_WORD]));
        assert_eq!(mapper_offset(&duplicate), None);
        let mut damaged = patched(&[TEST_BRANCH_PATCH_WORD]);
        damaged[24] ^= 1;
        assert_eq!(mapper_offset(&damaged), None);
        assert_eq!(
            mapper_offset(&patched(&[
                LOAD_X17_LITERAL_OPCODE,
                BRANCH_X17_OPCODE,
                0,
                0,
            ])),
            None
        );
        let mut unaligned = vec![0];
        unaligned.extend(MAPPING_NEEDLE);
        assert_eq!(mapper_offset(&unaligned), None);
        assert_eq!(mapper_offset(&[]), None);
    }
}
