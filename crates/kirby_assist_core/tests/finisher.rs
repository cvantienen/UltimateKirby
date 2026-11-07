use kirby_assist_core::{
    button_modes::YMode,
    combat_assist::FairContext,
    finisher::{Context, Finisher},
    native_input::{MAPPER_A, MAPPER_X, MAPPER_ZL},
};
fn ctx(status: i32, motion: u64, frame: f32, grounded: bool) -> Context {
    Context {
        fighter_kind: kirby_assist_core::fighter_kinds::KIRBY,
        target_direction: None,
        fighter: FairContext {
            object: 1,
            kirby: true,
            status,
            motion,
            frame,
            rate: 1.0,
            grounded,
            stopped: false,
            facing: 1.0,
            hit_now: false,
        },
        velocity_y: -0.5,
        can_jump: grounded,
        can_start: grounded,
    }
}

#[test]
fn short_hop_descent_timeout_releases_controls_and_does_not_restart_while_held() {
    for mode in [YMode::FairSmash, YMode::DairSmash] {
        let mut tool = Finisher::default();
        start(&mut tool, mode);
        let motion = if mode == YMode::FairSmash {
            kirby_assist_core::timing::MOTION_ATTACK_AIR_F
        } else {
            kirby_assist_core::timing::MOTION_ATTACK_AIR_LW
        };
        for frame in 0..16 {
            let c = ctx(
                kirby_assist_core::native_states::status::ATTACK_AIR,
                motion,
                frame as f32,
                false,
            );
            assert_ne!(tool.sample(true, true, mode, c).owned, 0);
            for _ in 0..4 {
                tool.sample(true, true, mode, c);
            }
        }
        tool.sample(
            true,
            true,
            mode,
            ctx(
                kirby_assist_core::native_states::status::ATTACK_AIR,
                motion,
                16.0,
                false,
            ),
        );
        let released = tool.sample(
            true,
            true,
            mode,
            ctx(
                kirby_assist_core::native_states::status::ATTACK_AIR,
                motion,
                17.0,
                false,
            ),
        );
        assert_eq!(released.owned, 0);
        assert_eq!(released.stick, None);
        assert!(!tool.active());
        assert_eq!(tool.sample(true, true, mode, ctx(0, 1, 0.0, true)).owned, 0);
    }
}

#[test]
fn airborne_combo_presses_do_nothing_and_cannot_queue_a_takeoff_after_landing() {
    for mode in [YMode::FairSmash, YMode::DairSmash, YMode::YoshiBairSmash] {
        for status in [
            kirby_assist_core::native_states::status::JUMP,
            kirby_assist_core::native_states::status::JUMP_AERIAL,
            kirby_assist_core::native_states::status::FALL,
            kirby_assist_core::native_states::status::FALL_AERIAL,
        ] {
            let mut tool = Finisher::default();
            let mut c = ctx(
                status,
                kirby_assist_core::native_states::status::DASH as u64,
                0.0,
                false,
            );
            c.fighter_kind = if mode == YMode::YoshiBairSmash { 5 } else { 6 };
            assert_eq!(tool.sample(true, true, mode, c).owned, 0);
            assert!(!tool.active());
            let mut landed = ctx(0, 1, 0.0, true);
            landed.fighter_kind = c.fighter_kind;
            assert_eq!(
                tool.sample(true, true, mode, landed).owned,
                0,
                "an airborne press must not be queued until landing"
            );
            tool.sample(true, false, mode, landed);
            assert_ne!(
                tool.sample(true, true, mode, landed).owned,
                0,
                "a fresh grounded press still starts normally"
            );
        }
    }
}

#[test]
fn fair_combo_marks_delayed_takeoff_short_then_fast_falls_and_holds_down_until_landing() {
    use kirby_assist_core::native_input::mapped_short_hop;
    let mut tool = Finisher::default();
    let mode = YMode::FairSmash;
    for frame in 0..12 {
        let jump = tool.sample(true, true, mode, ctx(0, 1, frame as f32, true));
        assert_eq!(jump.requested, MAPPER_X);
        assert!(tool.requests_short_hop());
        assert_eq!(mapped_short_hop(0x4, tool.requests_short_hop()), 0x44);
        for _ in 0..8 {
            assert_eq!(
                tool.sample(true, true, mode, ctx(0, 1, frame as f32, true)),
                jump
            );
        }
    }
    tool.sample(true, true, mode, ctx(0xa, 2, 0.0, true));
    assert!(!tool.requests_short_hop());
    let mut apex = ctx(0xb, 3, 0.0, false);
    apex.velocity_y = 0.1;
    let fair = tool.sample(true, true, mode, apex);
    assert_eq!(fair.requested, MAPPER_A);
    assert_eq!(
        fair.stick,
        Some([1.0, 0.0]),
        "select fair before pressing Down"
    );
    assert!(!tool.requests_short_hop());
    for frame in 0..12 {
        let mut airborne = ctx(
            kirby_assist_core::native_states::status::ATTACK_AIR,
            kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
            frame as f32,
            false,
        );
        airborne.fighter.hit_now = frame == 10;
        let down = tool.sample(true, true, mode, airborne);
        assert_eq!(down.stick, Some([0.0, -1.0]));
        assert_eq!(down.requested & MAPPER_X, 0, "no extra jump from held Y");
        for _ in 0..8 {
            assert_eq!(tool.sample(true, true, mode, airborne), down);
        }
        airborne.fighter.stopped = true;
        airborne.velocity_y = 0.0;
        assert_eq!(tool.sample(true, true, mode, airborne), down);
    }
    assert!(tool.confirmed());
    let landing = tool.sample(true, true, mode, ctx(0x18, 4, 0.0, true));
    assert_eq!(landing.requested, MAPPER_A);
    assert_eq!(landing.stick, Some([1.0, 0.5]));
    assert!(!tool.requests_short_hop());
    assert_eq!(
        mapped_short_hop(0x4, false),
        0x4,
        "manual full hops unchanged"
    );
}

#[test]
fn delayed_jump_acceptance_preserves_every_finisher_and_never_repeats_jump() {
    for mode in [YMode::FairSmash, YMode::DairSmash, YMode::YoshiBairSmash] {
        let make = |status, motion, frame, grounded| {
            let mut c = ctx(status, motion, frame, grounded);
            c.fighter_kind = if mode == YMode::YoshiBairSmash { 5 } else { 6 };
            c.fighter.facing = if mode == YMode::YoshiBairSmash {
                -1.0
            } else {
                1.0
            };
            c.target_direction = Some(1.0);
            c.velocity_y = 0.0;
            c
        };
        let mut f = Finisher::default();
        let launch = f.sample(true, true, mode, make(0, 1, 0.0, true));
        assert_eq!(
            launch.requested,
            if mode == YMode::YoshiBairSmash {
                MAPPER_X | MAPPER_ZL
            } else {
                MAPPER_X
            }
        );
        for age in 1..13 {
            let waiting = f.sample(true, false, mode, make(0, 1, age as f32, true));
            assert!(
                f.active(),
                "{mode:?} cancelled before delayed jump acceptance"
            );
            assert_eq!(
                waiting.requested,
                if mode == YMode::YoshiBairSmash {
                    0
                } else {
                    MAPPER_X
                },
                "retain Kirby jump until native squat without creating repeated edges"
            );
        }
        // Test a skipped squat observation and mixed takeoff situation/status.
        let attack = f.sample(true, false, mode, make(0xb, 3, 0.0, true));
        assert_eq!(
            attack.requested, MAPPER_A,
            "{mode:?} must proceed after native Jump"
        );
        let motion = match mode {
            YMode::FairSmash => kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
            YMode::DairSmash => kirby_assist_core::timing::MOTION_ATTACK_AIR_LW,
            YMode::YoshiBairSmash => kirby_assist_core::timing::motion_hash("attack_air_b"),
        };
        let accepted = f.sample(
            true,
            false,
            mode,
            make(
                kirby_assist_core::native_states::status::ATTACK_AIR,
                motion,
                0.0,
                true,
            ),
        );
        assert_eq!(accepted.requested, 0);
        assert!(f.active());
    }
}

#[test]
fn rejected_jump_remains_bounded_and_cannot_restart_from_held_y() {
    let mut f = Finisher::default();
    let mode = YMode::DairSmash;
    f.sample(true, true, mode, ctx(0, 1, 0.0, true));
    for frame in 1..=25 {
        f.sample(true, true, mode, ctx(0, 1, frame as f32, true));
    }
    assert!(!f.active());
    assert_eq!(f.sample(true, true, mode, ctx(0, 1, 26.0, true)).owned, 0);
}
fn start(tool: &mut Finisher, mode: YMode) {
    assert_eq!(
        tool.sample(true, true, mode, ctx(0, 1, 0.0, true))
            .requested,
        MAPPER_X
    );
    assert_eq!(
        tool.sample(true, true, mode, ctx(0xa, 2, 0.0, true))
            .requested,
        0
    );
    assert_eq!(
        tool.sample(true, true, mode, ctx(0xb, 3, 0.0, false))
            .requested,
        MAPPER_A
    );
    // If native takeoff has not accepted A, release once and retry.
    assert_eq!(
        tool.sample(true, true, mode, ctx(0xb, 3, 3.0, false))
            .requested,
        0
    );
    let select = tool.sample(true, true, mode, ctx(0xb, 3, 4.0, false));
    assert_eq!(select.requested, MAPPER_A);
    assert_eq!(
        select.stick,
        Some(if mode == YMode::FairSmash {
            [1.0, 0.0]
        } else {
            [0.0, -1.0]
        })
    );
}

#[test]
fn fair_waits_for_apex_then_selects_fair_before_fast_fall_and_requires_hit() {
    let mut tool = Finisher::default();
    let mode = YMode::FairSmash;
    tool.sample(true, true, mode, ctx(0, 1, 0.0, true));
    assert_eq!(
        tool.sample(true, false, mode, ctx(0xa, 2, 0.0, true))
            .requested,
        0
    );
    for (frame, vy) in [(0.0, 1.0), (1.0, 0.8), (2.0, 0.3)] {
        let c = Context {
            velocity_y: vy,
            ..ctx(0xb, 3, frame, false)
        };
        assert_eq!(tool.sample(true, false, mode, c).requested, 0);
    }
    let apex = Context {
        velocity_y: 0.1,
        ..ctx(0xb, 3, 3.0, false)
    };
    let fair = tool.sample(true, false, mode, apex);
    assert_eq!(fair.requested, MAPPER_A);
    assert_eq!(fair.stick, Some([1.0, 0.0]));
    let accepted = ctx(
        kirby_assist_core::native_states::status::ATTACK_AIR,
        kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
        0.0,
        false,
    );
    assert_eq!(tool.sample(true, false, mode, accepted).requested, 0);
    let down = tool.sample(
        true,
        false,
        mode,
        ctx(
            kirby_assist_core::native_states::status::ATTACK_AIR,
            kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
            4.0,
            false,
        ),
    );
    assert_eq!(down.stick, Some([0.0, -1.0]));
    assert!(!tool.confirmed());
    assert_eq!(
        tool.sample(true, false, mode, ctx(0x18, 4, 0.0, true))
            .requested,
        0
    );
}

#[test]
fn aerial_delivery_buffers_in_squat_retries_missed_inputs_and_stops_on_native_acceptance() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput},
        native_input::{ButtonTrace, MAPPER_Y},
    };
    for mode in [YMode::FairSmash, YMode::DairSmash, YMode::YoshiBairSmash] {
        for facing in [-1.0, 1.0] {
            let mut tool = Finisher::default();
            let mut overlay = ManualOverlay::default();
            let make = |status, motion, frame, grounded| {
                let mut c = ctx(status, motion, frame, grounded);
                c.fighter.facing = facing;
                if mode == YMode::YoshiBairSmash {
                    c.fighter_kind = 5;
                    c.fighter.kirby = false;
                    c.target_direction = Some(facing);
                    c.fighter.facing = -facing; // Already turned away from the target.
                }
                c
            };
            let jump = tool.sample(true, true, mode, make(0, 1, 0.0, true));
            overlay.apply_controls(
                ButtonTrace::default(),
                [0.0; 2],
                TurboOutput {
                    consume_dpad: false,
                    stick: jump.stick,
                },
                MAPPER_Y,
                jump.requested,
                jump.owned,
            );
            // Native squat can overlap the situation change at takeoff. Even
            // then it must release Jump and prepare the correct directional A.
            let squat = tool.sample(true, false, mode, make(0xa, 2, 0.0, false));
            let raw = overlay.apply_controls(
                ButtonTrace::default(),
                [0.0; 2],
                TurboOutput {
                    consume_dpad: false,
                    stick: squat.stick,
                },
                MAPPER_Y,
                squat.requested,
                squat.owned,
            );
            assert_eq!(raw.buttons.pressed & MAPPER_A, 0);
            assert_eq!(raw.buttons.held & (MAPPER_X | MAPPER_ZL), 0);
            assert_eq!(
                raw.stick,
                if mode == YMode::YoshiBairSmash {
                    [-facing, 0.0]
                } else {
                    [0.0, 0.0]
                }
            );
            // Duplicate polls and hitstop cannot consume the hold/retry budget.
            let mut stopped = make(0xa, 2, 0.0, false);
            stopped.fighter.stopped = true;
            assert_eq!(tool.sample(true, false, mode, stopped), squat);
            assert_eq!(
                tool.sample(true, false, mode, make(0xa, 2, 0.0, false)),
                squat
            );
            // Simulate native rejection: stay in jump instead of ATTACK_AIR.
            let mut retry_edge = false;
            for frame in 0..7 {
                let proposal = tool.sample(true, false, mode, make(0xb, 3, frame as f32, false));
                let raw = overlay.apply_controls(
                    ButtonTrace::default(),
                    [0.0; 2],
                    TurboOutput {
                        consume_dpad: false,
                        stick: proposal.stick,
                    },
                    MAPPER_Y,
                    proposal.requested,
                    proposal.owned,
                );
                retry_edge |= frame > 1 && raw.buttons.pressed & MAPPER_A != 0;
                assert_eq!(raw.buttons.held & (MAPPER_X | MAPPER_ZL), 0);
            }
            assert!(retry_edge, "an ignored A must get a fresh press edge");
            let motion = match mode {
                YMode::DairSmash => kirby_assist_core::timing::MOTION_ATTACK_AIR_LW,
                YMode::FairSmash => kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
                YMode::YoshiBairSmash => kirby_assist_core::timing::motion_hash("attack_air_b"),
            };
            let accepted = tool.sample(
                true,
                false,
                mode,
                make(
                    kirby_assist_core::native_states::status::ATTACK_AIR,
                    motion,
                    0.0,
                    false,
                ),
            );
            assert_eq!(accepted.requested, 0);
            assert!(tool.active());
        }
    }
}
#[test]
fn both_routes_confirm_real_hits_then_buffer_landing_smashes_without_charging_or_repeating() {
    for mode in [YMode::FairSmash, YMode::DairSmash] {
        let mut tool = Finisher::default();
        start(&mut tool, mode);
        let motion = if mode == YMode::FairSmash {
            kirby_assist_core::timing::MOTION_ATTACK_AIR_F
        } else {
            kirby_assist_core::timing::MOTION_ATTACK_AIR_LW
        };
        let first = if mode == YMode::FairSmash { 10 } else { 18 };
        for frame in 0..=first {
            let mut c = ctx(
                kirby_assist_core::native_states::status::ATTACK_AIR,
                motion,
                frame as f32,
                false,
            );
            c.velocity_y = if frame < first { 0.5 } else { -0.5 };
            c.fighter.hit_now = frame == first;
            c.fighter.stopped = frame == first;
            let out = tool.sample(true, true, mode, c);
            assert_eq!(out.requested, 0);
        }
        let land = ctx(0x18, 4, 0.0, true);
        let fire = tool.sample(true, true, mode, land);
        assert_eq!(fire.requested, MAPPER_A);
        assert_eq!(
            fire.stick,
            Some(if mode == YMode::FairSmash {
                [1.0, 0.5]
            } else {
                [0.0, -1.0]
            })
        );
        assert_eq!(tool.sample(true, true, mode, land), fire);
        assert_eq!(
            tool.sample(true, true, mode, ctx(0x18, 4, 1.0, true))
                .requested,
            0
        );
        assert_eq!(
            tool.sample(true, true, mode, ctx(0x18, 4, 2.0, true))
                .requested,
            MAPPER_A
        );
        let status = if mode == YMode::FairSmash { 0x2d } else { 0x30 };
        assert_eq!(
            tool.sample(
                true,
                true,
                mode,
                ctx(
                    status,
                    kirby_assist_core::native_states::status::RUN_BRAKE as u64,
                    0.0,
                    true
                )
            )
            .requested,
            0
        );
        tool.sample(true, true, mode, ctx(status + 2, 6, 0.0, true));
        assert!(!tool.active());
        assert_eq!(
            tool.sample(true, true, mode, ctx(0, 1, 0.0, true)).owned,
            0,
            "holding Y cannot loop"
        );
    }
}
#[test]
fn whiffs_shields_damage_and_mode_changes_never_issue_a_finisher() {
    for mode in [YMode::FairSmash, YMode::DairSmash] {
        let mut tool = Finisher::default();
        start(&mut tool, mode);
        let motion = if mode == YMode::FairSmash {
            kirby_assist_core::timing::MOTION_ATTACK_AIR_F
        } else {
            kirby_assist_core::timing::MOTION_ATTACK_AIR_LW
        };
        for frame in 0..=11 {
            tool.sample(
                true,
                false,
                mode,
                ctx(
                    kirby_assist_core::native_states::status::ATTACK_AIR,
                    motion,
                    frame as f32,
                    false,
                ),
            );
        }
        assert_eq!(
            tool.sample(true, false, mode, ctx(0x18, 4, 0.0, true))
                .owned,
            0
        );
        let mut tool = Finisher::default();
        start(&mut tool, mode);
        assert_eq!(
            tool.sample(true, false, mode, ctx(0x49, 7, 0.0, false))
                .owned,
            0
        );
        let mut tool = Finisher::default();
        start(&mut tool, mode);
        assert_eq!(
            tool.sample(true, false, mode.cycle(true, true), ctx(0xb, 3, 3.0, false))
                .owned,
            0
        );
    }
}

#[test]
fn dair_landing_hit_and_later_fair_hits_confirm_their_finishers() {
    let mut tool = Finisher::default();
    start(&mut tool, YMode::DairSmash);
    tool.sample(
        true,
        false,
        YMode::DairSmash,
        ctx(
            kirby_assist_core::native_states::status::ATTACK_AIR,
            kirby_assist_core::timing::MOTION_ATTACK_AIR_LW,
            0.0,
            false,
        ),
    );
    let mut landing = ctx(
        0x18,
        kirby_assist_core::timing::motion_hash("landing_air_lw"),
        0.0,
        true,
    );
    landing.fighter.hit_now = true;
    assert_eq!(
        tool.sample(true, false, YMode::DairSmash, landing)
            .requested,
        MAPPER_A
    );
    let mut tool = Finisher::default();
    start(&mut tool, YMode::FairSmash);
    for frame in 0..=17 {
        let mut c = ctx(
            kirby_assist_core::native_states::status::ATTACK_AIR,
            kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
            frame as f32,
            false,
        );
        c.velocity_y = if frame < 10 { 0.5 } else { -0.5 };
        c.fighter.hit_now = frame == 10 || frame == 17;
        tool.sample(true, false, YMode::FairSmash, c);
    }
    assert!(
        tool.active(),
        "a later fair frame must not discard a confirmed hit"
    );
    assert_eq!(
        tool.sample(true, false, YMode::FairSmash, ctx(0x18, 4, 0.0, true))
            .requested,
        MAPPER_A
    );
}

#[test]
fn fair_confirm_persists_through_later_hits_and_aims_up_fsmash_at_current_enemy() {
    for hit_frame in [10, 13, 17, 25] {
        let mut tool = Finisher::default();
        let mode = YMode::FairSmash;
        start(&mut tool, mode);
        for frame in 0..=26 {
            let mut c = ctx(
                kirby_assist_core::native_states::status::ATTACK_AIR,
                kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
                frame as f32,
                false,
            );
            c.velocity_y = if frame < 17 { 0.5 } else { -0.5 };
            c.fighter.hit_now = frame == hit_frame;
            tool.sample(true, false, mode, c);
        }
        assert!(tool.confirmed());
        let falling = tool.sample(true, false, mode, ctx(0xe, 77, 0.0, false));
        assert_eq!(falling.requested, 0, "do not throw a second aerial");
        let landing = Context {
            target_direction: Some(-1.0),
            ..ctx(0x18, 4, 0.0, true)
        };
        let smash = tool.sample(true, false, mode, landing);
        assert_eq!(smash.requested, MAPPER_A);
        assert_eq!(smash.stick, Some([-1.0, 0.5]));
        let startup = tool.sample(
            true,
            false,
            mode,
            Context {
                target_direction: Some(1.0),
                ..ctx(0x2d, 5, 0.0, true)
            },
        );
        assert_eq!(startup.requested, 0);
        assert_eq!(
            startup.stick,
            Some([-1.0, 0.5]),
            "retain chosen angle/direction after native acceptance"
        );
    }
}

#[test]
fn fair_contact_on_landing_or_grounded_air_status_still_arms_fsmash() {
    for landing_motion in [
        kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
        kirby_assist_core::timing::motion_hash("landing_air_f"),
    ] {
        let mut tool = Finisher::default();
        let mode = YMode::FairSmash;
        start(&mut tool, mode);
        tool.sample(
            true,
            false,
            mode,
            ctx(
                kirby_assist_core::native_states::status::ATTACK_AIR,
                kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
                0.0,
                false,
            ),
        );
        let mut contact = ctx(
            if landing_motion == kirby_assist_core::timing::MOTION_ATTACK_AIR_F {
                kirby_assist_core::native_states::status::ATTACK_AIR
            } else {
                0x18
            },
            landing_motion,
            1.0,
            true,
        );
        contact.fighter.hit_now = true;
        let first = tool.sample(true, false, mode, contact);
        assert!(tool.confirmed());
        if contact.fighter.status == kirby_assist_core::native_states::status::LANDING_ATTACK_AIR {
            assert_eq!(first.requested, MAPPER_A);
        } else {
            assert_eq!(
                tool.sample(true, false, mode, ctx(0x18, 4, 0.0, true))
                    .requested,
                MAPPER_A
            );
        }
    }
}

#[test]
fn disabled_and_identity_changes_cancel_even_a_confirmed_route() {
    for disable in [false, true] {
        let mut tool = Finisher::default();
        start(&mut tool, YMode::FairSmash);
        for frame in 0..=10 {
            let mut c = ctx(
                kirby_assist_core::native_states::status::ATTACK_AIR,
                kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
                frame as f32,
                false,
            );
            c.fighter.hit_now = frame == 10;
            tool.sample(true, false, YMode::FairSmash, c);
        }
        let mut land = ctx(0x18, 4, 0.0, true);
        if !disable {
            land.fighter.object = 2;
        }
        assert_eq!(
            tool.sample(!disable, false, YMode::FairSmash, land).owned,
            0
        );
        assert!(!tool.active());
    }
}

#[test]
fn yoshi_sprint_instant_turnaround_bair_confirms_then_faces_target_for_fsmash() {
    let mode = YMode::YoshiBairSmash;
    let make = |status, motion, frame, grounded, facing| {
        let mut c = ctx(status, motion, frame, grounded);
        c.fighter_kind = 5;
        c.fighter.kirby = false;
        c.fighter.facing = facing;
        c.target_direction = Some(1.0);
        c
    };
    let mut tool = Finisher::default();
    let turn = tool.sample(true, true, mode, make(6, 1, 0.0, true, 1.0));
    assert_eq!(turn.requested, MAPPER_X | MAPPER_ZL);
    assert_eq!(turn.stick, Some([-1.0, 0.0]));
    assert!(tool.requests_short_hop());
    let squat = tool.sample(true, false, mode, make(0xa, 3, 0.0, true, -1.0));
    assert_eq!(squat.requested, 0);
    assert_eq!(squat.stick, Some([-1.0, 0.0]));
    assert_eq!(
        tool.sample(true, false, mode, make(0xb, 4, 0.0, false, -1.0))
            .stick,
        Some([1.0, 0.0])
    );
    for frame in 0..=11 {
        let mut c = make(
            kirby_assist_core::native_states::status::ATTACK_AIR,
            kirby_assist_core::timing::motion_hash("attack_air_b"),
            frame as f32,
            false,
            -1.0,
        );
        c.fighter.hit_now = frame == 11;
        let output = tool.sample(true, false, mode, c);
        assert_eq!(
            output.stick,
            Some([0.0, -1.0]),
            "Yoshi should hold Down from bair's apex through frame {frame}"
        );
    }
    assert!(tool.confirmed());
    let face = tool.sample(true, false, mode, make(0x18, 5, 0.0, true, -1.0));
    assert_eq!(face.requested, 0);
    assert_eq!(face.stick, Some([0.3, 0.0]));
    assert_eq!(
        tool.sample(true, false, mode, make(0x18, 5, 1.0, true, 1.0))
            .requested,
        0
    );
    let smash = tool.sample(true, false, mode, make(0x18, 5, 2.0, true, 1.0));
    assert_eq!(smash.requested, MAPPER_A);
    assert_eq!(smash.stick, Some([1.0, 0.0]));
}

#[test]
fn kirby_starters_accept_all_grounded_movement_states_and_queue_transient_jump_readiness() {
    use kirby_assist_core::finisher::grounded_start_allowed;
    for status in (kirby_assist_core::native_states::status::WAIT
        ..=kirby_assist_core::native_states::status::TURN_RUN_BRAKE)
        .chain(
            kirby_assist_core::native_states::status::SQUAT
                ..=kirby_assist_core::native_states::status::SQUAT_RV,
        )
    {
        for mode in [YMode::FairSmash, YMode::DairSmash] {
            let mut tool = Finisher::default();
            let mut moving = ctx(
                status,
                kirby_assist_core::native_states::status::WALK as u64,
                0.0,
                true,
            );
            moving.can_start =
                grounded_start_allowed(status, kirby_assist_core::native_states::status::WAIT);
            moving.can_jump = false;
            let waiting = tool.sample(true, true, mode, moving);
            assert_ne!(waiting.owned, 0, "status {status:#x} lost the starter");
            assert_eq!(waiting.requested, MAPPER_X);
            for _ in 0..100 {
                assert_eq!(tool.sample(true, true, mode, moving), waiting);
            }
            moving.fighter.frame = 1.0;
            moving.can_jump = true;
            assert_eq!(tool.sample(true, true, mode, moving).requested, MAPPER_X);
            moving.fighter.frame = 2.0;
            assert_eq!(
                tool.sample(true, true, mode, moving).requested,
                MAPPER_X,
                "retain the same native jump request until squat"
            );
            let mut immediate = Finisher::default();
            assert_eq!(
                immediate.sample(true, true, mode, moving).requested,
                MAPPER_X
            );
        }
    }
    for status in [
        kirby_assist_core::native_states::status::JUMP_SQUAT,
        kirby_assist_core::native_states::status::LANDING_ATTACK_AIR,
        kirby_assist_core::native_states::GUARD_DAMAGE,
        kirby_assist_core::native_states::status::ATTACK,
        kirby_assist_core::native_states::status::ATTACK_LW4,
        kirby_assist_core::native_states::status::DAMAGE,
    ] {
        assert!(!grounded_start_allowed(
            status,
            kirby_assist_core::native_states::status::WAIT
        ));
        let mut tool = Finisher::default();
        assert_eq!(
            tool.sample(
                true,
                true,
                YMode::FairSmash,
                ctx(
                    status,
                    kirby_assist_core::native_states::status::WALK as u64,
                    0.0,
                    true
                )
            )
            .owned,
            0
        );
    }
    assert!(!grounded_start_allowed(4, 2));
}

#[test]
fn descending_dair_rearms_down_after_delayed_acceptance_then_keeps_it_held() {
    let mut tool = Finisher::default();
    let mode = YMode::DairSmash;
    start(&mut tool, mode);
    let accepted = tool.sample(
        true,
        false,
        mode,
        ctx(
            kirby_assist_core::native_states::status::ATTACK_AIR,
            kirby_assist_core::timing::MOTION_ATTACK_AIR_LW,
            0.0,
            false,
        ),
    );
    assert_eq!(accepted.stick, Some([0.0; 2]));
    let down = tool.sample(
        true,
        false,
        mode,
        ctx(
            kirby_assist_core::native_states::status::ATTACK_AIR,
            kirby_assist_core::timing::MOTION_ATTACK_AIR_LW,
            1.0,
            false,
        ),
    );
    assert_eq!(down.stick, Some([0.0, -1.0]));
    let mut rising_read = ctx(
        kirby_assist_core::native_states::status::ATTACK_AIR,
        kirby_assist_core::timing::MOTION_ATTACK_AIR_LW,
        2.0,
        false,
    );
    rising_read.velocity_y = 0.1;
    assert_eq!(
        tool.sample(true, false, mode, rising_read).stick,
        down.stick
    );
}

#[test]
fn automatic_fair_followup_holds_down_across_frozen_hit_frames() {
    use kirby_assist_core::combat_assist::FairFollowup;
    let mut tool = FairFollowup::default();
    let mut hit = ctx(
        kirby_assist_core::native_states::status::ATTACK_AIR,
        kirby_assist_core::timing::MOTION_ATTACK_AIR_F,
        10.0,
        false,
    )
    .fighter;
    hit.hit_now = true;
    assert_eq!(
        tool.sample_with_velocity(true, hit, -0.5).stick,
        Some([0.0, -1.0])
    );
    hit.stopped = true;
    assert_eq!(
        tool.sample_with_velocity(true, hit, 0.0).stick,
        Some([0.0, -1.0])
    );
    let landing = ctx(0x18, 4, 0.0, true).fighter;
    assert_eq!(
        tool.sample_with_velocity(true, landing, 0.0).requested,
        MAPPER_A
    );
}

#[test]
fn kirby_combo_jump_is_immediate_x_only_and_keeps_physical_takeoff_direction() {
    use kirby_assist_core::manual_controls::{ManualOverlay, TurboOutput};
    use kirby_assist_core::native_input::ButtonTrace;
    for mode in [YMode::FairSmash, YMode::DairSmash] {
        let mut tool = Finisher::default();
        let mut c = ctx(4, 1, 0.0, true);
        c.can_jump = false; // Transient native jump term must not delay the input.
        c.fighter.stopped = true;
        let jump = tool.sample(true, true, mode, c);
        assert_eq!(jump.requested, MAPPER_X);
        let mapped = ManualOverlay::default().apply_controls(
            ButtonTrace::default(),
            [1.0, 0.0],
            TurboOutput {
                consume_dpad: false,
                stick: jump.stick,
            },
            0,
            jump.requested,
            jump.owned,
        );
        assert_eq!(mapped.buttons.pressed & (MAPPER_X | MAPPER_ZL), MAPPER_X);
        assert_eq!(mapped.stick, [1.0, 0.0]);
        for _ in 0..100 {
            assert_eq!(tool.sample(true, false, mode, c), jump);
        }
        c.fighter.status = kirby_assist_core::native_states::status::JUMP_SQUAT;
        c.fighter.motion = 2;
        let release = tool.sample(true, false, mode, c);
        assert_eq!(release.requested & (MAPPER_X | MAPPER_ZL), 0);
        assert!(tool.active());
    }
}
