mod tests {
    use super::*;

    /// Sets the upper bound for recorder size bytes.
    const MAX_RECORDER_SIZE_BYTES: usize = 1024;
    /// Sets the upper bound for report size bytes.
    const MAX_REPORT_SIZE_BYTES: usize = 768;

    /// Regression test for `copied_labels_survive_menu_teardown_and_truncate_on_utf8_boundaries`.
    #[test]
    fn copied_labels_survive_menu_teardown_and_truncate_on_utf8_boundaries() {
        let name = "é".repeat(40);
        let label = Label::new(&name);
        drop(name);
        assert_eq!(label.as_str(), "é".repeat(32));
        assert_eq!(label.to_string(), label.as_str());
        assert!(Label::new("").is_empty());
        let menu_name = String::from("Saved custom option");
        let configuration =
            Configuration::new("Bowser", [menu_name.as_str(); CONFIGURATION_ROW_COUNT]);
        drop(menu_name);
        assert_eq!(configuration.rows[0].as_str(), "Saved custom option");
    }
    /// Captures the current values into a snapshot.
    fn snapshot() -> Snapshot {
        Snapshot {
            winner: Some(Owner::Ledge),
            present: bit(Owner::Ledge) | bit(Owner::Posture),
            committed: false,
            requested: 0,
            stick: None,
            status: crate::native_states::status::WAIT,
            motion: 1,
            native_frame: 0,
            stopped: false,
            can_act: true,
            hit: Some(false),
            valid: true,
            safety: 0,
        }
    }
    /// Regression test for `snapshots_explain_every_owner_and_preserve_intermediate_native_startup`.
    #[test]
    fn snapshots_explain_every_owner_and_preserve_intermediate_native_startup() {
        for owner in OWNERS {
            let mut r = Recorder::new();
            let s = Snapshot {
                winner: Some(owner),
                present: bit(owner),
                ..snapshot()
            };
            let report = r.sample(s, 0).unwrap();
            assert!(report.format().contains(purpose(owner)));
            assert!(report.card_text().contains(name(owner)));
        }
        let mut r = Recorder::new();
        r.sample(snapshot(), 0);
        r.sample(
            Snapshot {
                status: crate::native_states::status::ATTACK_AIR,
                motion: crate::timing::motion_hash("attack_air_f"),
                native_frame: 10,
                ..snapshot()
            },
            10,
        );
        let report = r.sample(snapshot(), 250).unwrap();
        assert!(report.format().contains("aerial attack"));
        assert!(report.format().contains("not proof"));
        assert!(report.card_text().contains("Hit: not confirmed"));
        assert_eq!(report.at_ms, 250);
        assert_eq!(
            motion_name(
                crate::native_states::status::ATTACK_AIR,
                crate::timing::motion_hash("attack_air_lw"),
            ),
            "down air"
        );
    }
    /// Regression test for `animation_frame_only_changes_do_not_spam_stable_modes`.
    #[test]
    fn animation_frame_only_changes_do_not_spam_stable_modes() {
        let mut r = Recorder::new();
        r.sample(snapshot(), 0);
        for frame in 1..1000 {
            assert!(r
                .sample(
                    Snapshot {
                        native_frame: frame,
                        ..snapshot()
                    },
                    u64::from(frame)
                )
                .is_none());
        }
    }
    /// Regression test for `duplicate_polls_are_free_and_batches_have_a_hard_rate_limit`.
    #[test]
    fn duplicate_polls_are_free_and_batches_have_a_hard_rate_limit() {
        let mut recorder = Recorder::new();
        assert!(recorder.sample(snapshot(), 0).is_some());
        for ms in 1..250 {
            assert!(recorder.sample(snapshot(), ms).is_none());
        }
        let mut count = 1;
        for ms in 250u64..=1250 {
            let s = Snapshot {
                requested: if ms.is_multiple_of(2) { MAPPER_A } else { 0 },
                ..snapshot()
            };
            count += usize::from(recorder.sample(s, ms).is_some());
        }
        assert_eq!(count, 6);
        assert!(recorder.sample(snapshot(), 0).is_some());
    }
    /// Regression test for `buffered_press_survives_release_in_the_same_batch_without_claiming_a_hit`.
    #[test]
    fn buffered_press_survives_release_in_the_same_batch_without_claiming_a_hit() {
        let mut r = Recorder::new();
        r.sample(snapshot(), 0);
        r.sample(
            Snapshot {
                requested: MAPPER_X | MAPPER_A,
                committed: true,
                ..snapshot()
            },
            1,
        );
        r.sample(
            Snapshot {
                status: crate::native_states::status::JUMP_SQUAT,
                can_act: false,
                ..snapshot()
            },
            16,
        );
        let report = r.sample(snapshot(), 250).unwrap().format();
        assert!(report.contains("Jump, Attack"));
        assert!(report.contains("never applied"));
        assert!(report.contains("do not prove"));
        assert!(report.contains("no hit flag"));
    }
    /// Regression test for `every_owner_and_configured_row_has_a_readable_label`.
    #[test]
    fn every_owner_and_configured_row_has_a_readable_label() {
        assert_eq!(
            OWNERS.iter().fold(0, |mask, owner| mask | bit(*owner)),
            (1u32 << OWNERS.len()) - 1
        );
        for owner in OWNERS {
            assert!(!name(owner).is_empty());
        }
        let text = Configuration::new("yoshi", ["OFF"; CONFIGURATION_ROW_COUNT]).format();
        assert!(text.contains("Neutral getup: OFF"));
    }
    /// Regression test for `configuration_rows_remain_aligned_after_removing_jab_grab`.
    #[test]
    fn configuration_rows_remain_aligned_after_removing_jab_grab() {
        let rows = [
            "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
            "16", "17", "18", "19", "20", "21", "22", "23", "24",
        ];
        let text = Configuration::new("yoshi", rows).format();
        for expected in [
            "Turnaround tilt: 9",
            "Spot-dodge punish: 11",
            "X button recipe: 16",
            "D-pad Right binding: 21",
            "Ledge trap mode: 22",
            "Out of shield: 23",
            "Shield pressure: 24",
        ] {
            assert!(text.contains(expected), "missing {expected}");
        }
    }
    /// Regression test for `left_result_stacks_below_menu_or_hides_instead_of_overlapping`.
    #[test]
    fn left_result_stacks_below_menu_or_hides_instead_of_overlapping() {
        for height in [480.0_f32, 720.0, 1080.0, 2160.0] {
            let scale = (height / 720.0).clamp(0.65, 3.0);
            let menu = 330.0 * scale;
            let (y, available) = left_result_space(height, scale, menu).unwrap();
            assert!(y > 8.0 * scale + menu);
            assert!(y + available <= height);
            assert!(left_result_space(height, scale, height).is_none());
        }
        assert!(left_result_space(f32::NAN, 1.0, 100.0).is_none());
    }
    /// Regression test for `disabled_modes_do_not_log_manual_play_and_evidence_storage_is_bounded`.
    #[test]
    fn disabled_modes_do_not_log_manual_play_and_evidence_storage_is_bounded() {
        let mut r = Recorder::new();
        for ms in 0..1000 {
            let idle = Snapshot {
                winner: None,
                present: 0,
                status: ms as i32,
                ..snapshot()
            };
            assert!(r.sample(idle, ms).is_none());
        }
        assert!(std::mem::size_of::<Recorder>() <= MAX_RECORDER_SIZE_BYTES);
        assert!(std::mem::size_of::<Report>() <= MAX_REPORT_SIZE_BYTES);
    }
    /// Regression test for `unknown_hit_evidence_and_invalid_scope_are_explained_without_inventing_a_blocker`.
    #[test]
    fn unknown_hit_evidence_and_invalid_scope_are_explained_without_inventing_a_blocker() {
        let mut r = Recorder::new();
        let report = r
            .sample(
                Snapshot {
                    winner: None,
                    valid: false,
                    hit: None,
                    ..snapshot()
                },
                0,
            )
            .unwrap()
            .format();
        assert!(report.contains("scope was unavailable"));
        assert!(report.contains("hit evidence was not sampled"));
        assert!(!report.contains("another mode retained priority"));
    }
}
