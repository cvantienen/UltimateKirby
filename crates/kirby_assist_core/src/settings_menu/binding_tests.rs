use super::*;
use crate::{
    button_bindings::{Activation, Mode},
    fighter_profiles::FighterProfiles,
    sequence_config::{persist, Catalog},
};
fn setup(text: &str, kind: i32) -> (MenuPresets, MenuSelection) {
    let mut profiles = FighterProfiles::default();
    profiles
        .add_sequence_choices(&Catalog::parse(text).unwrap())
        .unwrap();
    let p = profiles.profile(kind).unwrap().menu_options;
    let mut m = MenuSelection::new(&p);
    m.y_fighter_kind = kind;
    (p, m)
}
fn tap(m: &mut MenuSelection, p: &MenuPresets, b: u32) {
    m.input(0, p);
    m.input(b, p);
}
#[test]
fn idle_gameplay_reuses_button_roles_and_configuration_changes_refresh_them() {
    let (p, mut m) = setup("[bowser.x.hop]\nstart_delay=0\nactions=short_hop\n", 15);
    m.input_with_editing(0, &p, false);
    let first = m.binding_roles_rebuilds;
    for _ in 0..600 {
        m.input_with_editing(0, &p, false);
    }
    assert_eq!(m.binding_roles_rebuilds, first);
    for change in 0..5 {
        match change {
            0 => m.choices[PresetSlot::X as usize] += 1,
            1 => m.recipe_revision += 1,
            2 => m.rule_button_mask = MAPPER_Y,
            3 => m.binding_layout = crate::native_input::ControlLayout::Brother,
            _ => m.binding_safety_jump = true,
        }
        m.refresh_native_bindings(&p);
        assert_eq!(m.binding_roles_rebuilds, first + change + 1);
        m.refresh_native_bindings(&p);
        assert_eq!(m.binding_roles_rebuilds, first + change + 1);
    }
}
#[test]
fn dtilt_facing_is_available_for_yoshi_and_kirby_without_yoshi_idle_crouch() {
    for kind in [5, 6, 0] {
        let (_, mut m) = setup("", kind);
        m.modes_menu = true;
        assert!(m.row_visible(MenuRow::TiltTurn));
        assert!(Mode::TiltTurn.supports(kind));
        assert_eq!(
            m.row_visible(MenuRow::IdleCrouch),
            kind == crate::fighter_kinds::KIRBY
        );
        assert_eq!(
            Mode::IdleCrouch.supports(kind),
            kind == crate::fighter_kinds::KIRBY
        );
    }
}
#[test]
fn dpad_sequences_use_each_authored_slot_and_do_not_repeat_when_held() {
    for (name, slot) in [
        ("dpad_up", PresetSlot::DpadUp),
        ("dpad_down", PresetSlot::DpadDown),
        ("dpad_left", PresetSlot::DpadLeft),
        ("dpad_right", PresetSlot::DpadRight),
    ] {
        let (p, mut m) = setup(
            &format!("[kirby.{name}.custom]\ndefault=true\nstart_delay=2\nactions=hold:a:4\n"),
            6,
        );
        m.input(0, &p);
        let b = slot.button();
        assert_ne!(m.input(b, &p) & b, 0);
        assert_eq!(m.preview_slot, Some(slot));
        m.preview_slot = None;
        for _ in 0..5 {
            assert_ne!(m.input(b, &p) & b, 0);
            assert!(m.preview_slot.is_none());
        }
        tap(&mut m, &p, b);
        assert_eq!(m.preview_slot, Some(slot));
    }
}
#[test]
fn all_legacy_mode_bindings_load_as_normal_buttons_without_changing_modes() {
    for mode in Mode::ALL {
        let kind = if mode == Mode::IdleCrouch { 6 } else { 5 };
        for activation in ["hold", "toggle"] {
            for slot in PresetSlot::BUTTONS {
                let text = format!(
                    "[{}.{name}.old]\ndefault=true\nmode={}\nactivation={activation}\n",
                    if kind == crate::fighter_kinds::KIRBY {
                        "kirby"
                    } else {
                        "yoshi"
                    },
                    mode.id(),
                    name = [
                        "x",
                        "y",
                        "dpad_up",
                        "dpad_down",
                        "dpad_left",
                        "dpad_right",
                        "l",
                        "r",
                        "zl",
                        "zr",
                        "l3",
                        "r3"
                    ][slot as usize - PresetSlot::X as usize]
                );
                let (p, mut m) = setup(&text, kind);
                assert!(p.list(slot).options.is_empty());
                assert_eq!(
                    m.choice(&p, slot, m.choices[slot as usize]).unwrap().id,
                    crate::button_bindings::Builtin::default_for(slot).choice_id()
                );
                m.input(0, &p);
                let consumed = m.input(slot.button(), &p);
                assert_eq!(
                    consumed & slot.button() != 0,
                    crate::button_bindings::Builtin::default_for(slot)
                        != crate::button_bindings::Builtin::Normal
                );
                assert!(m.preview_slot.is_none());
                assert!(!m.trump_mode());
                assert!(m.out_of_shield);
                assert!(m.auto_tech);
                for base in [false, true] {
                    assert_eq!(m.effective_mode(mode, base), base);
                }
            }
        }
    }
}
#[test]
fn legacy_default_mode_keeps_saved_sequences_and_starts_on_normal_button() {
    let text="[yoshi.x.recipe]\ndefault=false\nstart_delay=0\nactions=short_hop_nair:4\n[yoshi.x.old_mode]\ndefault=true\nmode=out_of_shield\nactivation=toggle\n";
    let (p, mut m) = setup(text, 5);
    assert_eq!(p.list(PresetSlot::X).options.len(), 1);
    assert_eq!(p.list(PresetSlot::X).options[0].id, "recipe");
    assert_eq!(
        m.choices[PresetSlot::X as usize],
        PresetSlot::X.default_index(p.list(PresetSlot::X).options.len())
    );
    m.show_quick();
    m.button_menu = true;
    for slot in PresetSlot::BUTTONS {
        for index in 0..p.list(slot).options.len() + slot.templates() + 20 {
            if let Some(choice) = m.choice(&p, slot, index) {
                assert!(choice.sequence.mode.is_none());
            }
        }
    }
}
#[test]
fn mode_and_sequence_save_round_trip_and_nondefault_restart_are_valid() {
    let source = "[kirby.neutral.a]\nstart_delay=0\nactions=jab\n";
    for slot in PresetSlot::BUTTONS {
        let sequence = crate::sequence_config::Sequence {
            mode: Some(crate::button_bindings::ModeBinding {
                mode: Mode::LedgeTrump,
                activation: Activation::Toggle,
            }),
            ..Default::default()
        };
        let saved = persist::update_named(
            source,
            6,
            slot,
            "my_binding",
            sequence,
            true,
            Some("My mode"),
        )
        .unwrap();
        let c = Catalog::parse(&saved).unwrap();
        assert_eq!(c.entries()[1].sequence, sequence);
        assert!(!c.entries()[1].default);
        for diag in [false, true] {
            let mut profiles = FighterProfiles::default();
            profiles
                .add_sequence_choices(&c.clone().for_build(diag))
                .unwrap();
            profiles.validate().unwrap();
        }
        let actions = Catalog::parse("[kirby.x.actions]\nstart_delay=3\nactions=hold:a:4\n")
            .unwrap()
            .entries()[0]
            .sequence;
        let changed = persist::update(&saved, 6, slot, "my_binding", actions, false).unwrap();
        assert!(!changed.contains("mode ="));
        assert!(changed.contains("actions ="));
    }
    for bad in [
        "[kirby.neutral.bad]\nmode=ledge_trump",
        "[yoshi.x.bad]\nmode=idle_crouch",
        "[kirby.x.bad]\nmode=safe_spacing",
        "[kirby.x.bad]\nmode=ledge_trump\nstart_delay=1",
        "[kirby.x.bad]\nmode=ledge_trump\nstart_delay=0\nactions=grab",
    ] {
        assert!(Catalog::parse(bad).is_err(), "{bad}");
    }
}
#[test]
fn button_pages_expose_only_normal_and_input_sequence_editing() {
    let (p, mut m) = setup("", 6);
    tap(&mut m, &p, MAPPER_MINUS);
    assert!(m.quick_menu && !m.button_menu && !m.modes_menu);
    assert!(MenuSelection::LEDGE_ROWS.iter().all(|r| m.row_visible(*r)));
    assert!(!m.row_visible(MenuRow::XSequence));
    m.row = MenuRow::ButtonsPage;
    tap(&mut m, &p, DPAD_RIGHT);
    assert!(m.quick_menu && m.button_menu);
    assert!(MenuSelection::BUTTON_ROWS.iter().all(|r| m.row_visible(*r)));
    assert!(!m.row_visible(MenuRow::Neutral));
    m.row = MenuRow::DpadDown;
    assert_eq!(m.choice_position(&p, PresetSlot::DpadDown).1, 4);
    tap(&mut m, &p, MENU_STICK_CLICK);
    assert!(m.recipe_editing);
    assert!(m
        .recipe_panel(&p)
        .header
        .iter()
        .all(|s| !s.contains("Mode")));
    tap(&mut m, &p, MAPPER_Y);
    assert!(m.save_requested);
    tap(&mut m, &p, MENU_STICK_CLICK);
    assert!(m.button_menu && m.quick_menu);
    m.row = MenuRow::ModesPage;
    tap(&mut m, &p, DPAD_RIGHT);
    assert!(m.modes_menu);
    tap(&mut m, &p, MAPPER_MINUS);
    assert!(m.menu_hidden);
}
#[test]
fn unassigned_dpad_has_no_builtin_mode_or_shortcut_reservation() {
    let (p, mut m) = setup("", 6);
    for slot in PresetSlot::BUTTONS.into_iter().filter(|s| {
        matches!(
            s,
            PresetSlot::DpadUp
                | PresetSlot::DpadDown
                | PresetSlot::DpadLeft
                | PresetSlot::DpadRight
        )
    }) {
        m.input(0, &p);
        assert_eq!(m.input(slot.button(), &p) & slot.button(), 0);
        assert!(m.preview_slot.is_none());
        assert!(!m.trump_mode());
    }
    assert_eq!(
        observation_buttons(
            DPAD_UP | DPAD_DOWN | MAPPER_Y,
            DPAD_UP | DPAD_DOWN | MAPPER_Y
        ),
        0
    );
}

#[test]
fn yoshi_oos_is_only_an_on_off_mode() {
    let (p, mut m) = setup(
        "[yoshi.dpad_up.oos]\ndefault=true\nmode=out_of_shield\nactivation=toggle\n",
        5,
    );
    m.modes_menu = true;
    assert!(m.row_visible(MenuRow::OutOfShield));
    m.modes_menu = false;
    m.menu_hidden = true;
    tap(&mut m, &p, DPAD_UP);
    assert!(m.out_of_shield);
    assert_eq!(
        p.list(PresetSlot::DpadUp).selected,
        p.list(PresetSlot::DpadUp).options.len()
    );
}

#[test]
fn reaction_buttons_toggle_once_and_never_start_their_manual_recipe() {
    use crate::reaction_rules::Control;
    for control in Control::ALL.into_iter().skip(1) {
        let text = format!(
            "[yoshi.roll.rule]\nstart_delay=0\nreaction=roll:any:0:20:12:any:1:{}\nactions=jab:4\n",
            control.token()
        );
        let (p, mut m) = setup(&text, 5);
        m.input(0, &p);
        assert_eq!(m.rule_toggles, 0);
        let consumed = m.input(control.button(), &p);
        assert_ne!(consumed & control.button(), 0);
        assert_eq!(m.rule_toggles, control.bit());
        assert!(m.roll_rules);
        assert!(m.preview_slot.is_none());
        for _ in 0..12 {
            m.input(control.button(), &p);
            assert_eq!(m.rule_toggles, control.bit());
        }
        m.input(0, &p);
        m.input(control.button(), &p);
        assert_eq!(m.rule_toggles, 0);
    }
}

#[test]
fn reaction_toggle_requires_release_and_menu_navigation_cannot_activate_it() {
    use crate::reaction_rules::Control;
    let (p,mut m)=setup("[yoshi.roll.rule]\nstart_delay=0\nreaction=roll:any:0:20:12:any:1:toggle_up\nactions=jab:4\n",5);
    m.input(DPAD_UP, &p);
    m.input(DPAD_UP, &p);
    assert_eq!(m.rule_toggles, 0);
    m.input(0, &p);
    m.input(DPAD_UP, &p);
    assert_eq!(m.rule_toggles, Control::ToggleUp.bit());
    m.input(0, &p);
    m.input(MAPPER_MINUS, &p);
    assert!(!m.menu_hidden);
    tap(&mut m, &p, DPAD_UP);
    assert_eq!(m.rule_toggles, Control::ToggleUp.bit());
    tap(&mut m, &p, MAPPER_MINUS);
    tap(&mut m, &p, DPAD_UP);
    assert_eq!(m.rule_toggles, 0);
}

#[test]
fn shared_toggle_groups_do_not_double_toggle_and_other_buttons_still_run_sequences() {
    use crate::native_input::*;
    use crate::reaction_rules::Control;
    let (p,mut m)=setup("[yoshi.roll.a]\nstart_delay=0\nreaction=roll:any:0:10:12:any:1:toggle_x\nactions=jab:4\n[yoshi.roll.b]\nstart_delay=0\nreaction=roll:any:10:20:12:any:2:toggle_x\nactions=down_smash:4\n[yoshi.y.manual]\nstart_delay=0\ndefault=true\nactions=nair:4\n",5);
    tap(&mut m, &p, MAPPER_X);
    assert_eq!(m.rule_toggles, Control::ToggleX.bit());
    assert!(m.preview_slot.is_none());
    tap(&mut m, &p, MAPPER_Y);
    assert_eq!(m.preview_slot, Some(PresetSlot::Y));
    assert_eq!(m.rule_toggles, Control::ToggleX.bit());
    let (_, fresh) = setup("", 5);
    assert_eq!(fresh.rule_toggles, 0);
}

#[test]
fn removing_a_reaction_binding_clears_its_toggle_and_keeps_a_held_button_latched() {
    use crate::{native_input::MAPPER_X, reaction_rules::Control};
    let (p,mut m)=setup("[yoshi.roll.rule]\nstart_delay=0\nreaction=roll:any:0:20:12:any:1:toggle_x\nactions=jab:4\n",5);
    tap(&mut m, &p, MAPPER_X);
    assert_eq!(m.rule_toggles, Control::ToggleX.bit());
    // Reload an empty saved catalog as a fresh profile edit: the active binding
    // disappears, but held X must not leak through as an unexpected jump.
    m.load_saved_recipes(&p, &Catalog::default(), 5);
    assert_ne!(m.input(MAPPER_X, &p) & MAPPER_X, 0);
    assert_eq!(m.rule_toggles, 0);
    assert_eq!(m.rule_button_mask, 0);
    m.input(0, &p);
    assert_ne!(m.input(MAPPER_X, &p) & MAPPER_X, 0);
}
