mod tests {
    use super::*;

    /// Builds the patched instruction fixture used by tests.
    fn patched(words: &[u32]) -> Vec<u8> {
        let mut bytes = MAPPING_NEEDLE.to_vec();
        for (i, word) in words.iter().enumerate() {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    /// Regression test for `recognizes_original_and_all_skyline_entry_layouts`.
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

    /// Regression test for `rejects_ambiguity_damage_null_target_and_unaligned_matches`.
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
