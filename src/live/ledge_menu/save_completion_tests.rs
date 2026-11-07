//! Exercise the production completion handler, including maintenance consuming
//! results before the next input poll. No native runtime or SD access.
use super::*;
use kirby_assist_core::{
    fighter_profiles::{presets::PresetSlot, FighterProfiles},
    ledge_menu::{MenuRow, MENU_STICK_CLICK},
    native_input::MAPPER_Y,
    sequence_config::{Catalog, Sequence},
};

fn tap(selection: &mut MenuSelection, profiles: &FighterProfiles, buttons: u32) {
    let presets = &profiles.profile(15).unwrap().menu_options;
    selection.input(0, presets);
    selection.input(buttons, presets);
}

fn prepared() -> (FighterProfiles, LedgeMenu, save::ResultMessage, Sequence) {
    let mut profiles = FighterProfiles::default();
    profiles
        .add_sequence_choices(
            &Catalog::parse("[bowser.roll.saved]\nstart_delay=2\nactions=shield:4,grab\n").unwrap(),
        )
        .unwrap();
    let presets = &profiles.profile(15).unwrap().menu_options;
    let mut selection = MenuSelection::new(presets);
    selection.show();
    selection.row = MenuRow::Roll;
    tap(&mut selection, &profiles, MENU_STICK_CLICK);
    assert!(selection.recipe_editing);
    tap(
        &mut selection,
        &profiles,
        kirby_assist_core::manual_controls::DPAD_RIGHT,
    );
    tap(&mut selection, &profiles, MAPPER_Y);
    assert!(selection.save_requested);
    let sequence = selection
        .recipe_snapshot(presets, PresetSlot::Roll, 0)
        .unwrap();
    assert_eq!(sequence.start_delay, 3);
    let revision = selection.recipe_revision;
    let result = save::ResultMessage {
        kind: kirby_assist_core::fighter_kinds::BOWSER,
        slot: PresetSlot::Roll,
        index: 0,
        revision,
        choice: Some("saved".into()),
        success: true,
        deleted: false,
        message: "Saved to sequences.txt (backup created)".into(),
        snapshot: Some((sequence, None)),
        tuning: None,
    };
    let menu = LedgeMenu {
        selections: [(15, selection)].into(),
        kind: Some(15),
        resolved_recipe_revision: Some(revision),
        rule_stamp: Some((15, revision)),
        view_revision: Some(0),
        ..Default::default()
    };
    (profiles, menu, result, sequence)
}

#[test]
fn maintenance_acknowledgement_commits_the_first_edit_and_invalidates_runtime_caches() {
    let _guard = save_test_guard();
    let (profiles, mut menu, result, sequence) = prepared();
    menu.apply_save_result(result, Some(&profiles));
    assert!(menu.resolved_recipe_revision.is_none());
    assert!(menu.rule_stamp.is_none() && menu.view_revision.is_none());
    let presets = &profiles.profile(15).unwrap().menu_options;
    let selection = menu.selections.get_mut(&15).unwrap();
    assert_eq!(
        selection
            .choice(presets, PresetSlot::Roll, 0)
            .unwrap()
            .sequence,
        sequence
    );
    selection.input_with_editing(0, presets, false);
    selection.input_with_editing(0, presets, true);
    selection.show();
    selection.row = MenuRow::Roll;
    tap(selection, &profiles, MENU_STICK_CLICK);
    assert!(selection.recipe_editing);
    assert_eq!(
        selection.recipe_snapshot(presets, PresetSlot::Roll, 0),
        Some(sequence)
    );
    assert_eq!(
        selection
            .settings(presets, LedgeSettings::default())
            .sequence_overrides[PresetSlot::Roll as usize],
        Some(sequence)
    );
}

#[test]
fn stale_completion_never_overwrites_a_newer_edit_or_a_different_recipe() {
    let _guard = save_test_guard();
    for wrong_identity in [false, true] {
        let (profiles, mut menu, mut result, sequence) = prepared();
        let presets = &profiles.profile(15).unwrap().menu_options;
        let selection = menu.selections.get_mut(&15).unwrap();
        tap(
            selection,
            &profiles,
            kirby_assist_core::manual_controls::DPAD_RIGHT,
        );
        let newer = selection
            .recipe_snapshot(presets, PresetSlot::Roll, 0)
            .unwrap();
        assert_eq!(newer.start_delay, 4);
        // Revisions can coincide after rebuilding a session; payload identity
        // must still prevent a late result from marking this newer edit clean.
        result.revision = selection.recipe_revision;
        if wrong_identity {
            result.choice = Some("another_recipe".into());
        }
        menu.apply_save_result(result, Some(&profiles));
        let selection = menu.selections.get(&15).unwrap();
        assert_eq!(
            selection.recipe_snapshot(presets, PresetSlot::Roll, 0),
            Some(newer)
        );
        assert_eq!(
            selection
                .choice(presets, PresetSlot::Roll, 0)
                .unwrap()
                .sequence
                .start_delay,
            if wrong_identity {
                2
            } else {
                sequence.start_delay
            }
        );
        if !wrong_identity {
            assert!(selection
                .save_status
                .as_deref()
                .unwrap()
                .contains("newer edits are unsaved"));
        }
    }
}

#[test]
fn confirmed_match_end_releases_session_drafts_and_resolved_menu_metadata() {
    let _guard = save_test_guard();
    let (profiles, mut menu, result, _) = prepared();
    menu.apply_save_result(result, Some(&profiles));
    menu.resolved_preview = Some(PresetSlot::Roll);
    // This fixture's match guard is outside a battle; respawn deliberately
    // remains a separate deferred-cleanup path in production.
    assert!(!super::super::MASTER_IN_MATCH.load(std::sync::atomic::Ordering::Acquire));
    menu.end_match();
    assert!(menu.selections.is_empty());
    assert!(menu.kind.is_none() && menu.auto_mash.is_none());
    assert!(menu.rule_cache.is_none() && menu.rule_stamp.is_none());
    assert!(menu.resolved.is_none() && menu.resolved_choices.is_none());
    assert!(menu.resolved_recipe_revision.is_none() && menu.resolved_preview.is_none());
    assert!(menu.view_revision.is_none());
}

#[test]
fn every_training_editor_dispatches_y_save_without_gameplay_input_permission() {
    use kirby_assist_core::native_input::ControlLayout;
    static JOBS: std::sync::Mutex<Vec<(String, u16, bool)>> = std::sync::Mutex::new(Vec::new());
    fn record(job: &save::Job) -> Result<bool, String> {
        JOBS.lock().unwrap().push((
            job.choice.clone(),
            job.sequence.start_delay,
            job.tuning.is_some(),
        ));
        Ok(true)
    }
    struct Hook;
    impl Drop for Hook {
        fn drop(&mut self) {
            *save::TEST_OPERATION.lock().unwrap() = None;
        }
    }
    let _guard = save_test_guard();
    // A union of the host adapters' startup fixtures keeps their OnceLock
    // independent of test order. This instance loads its own editor records.
    let profiles=PROFILES.get_or_init(||{
        let c=Catalog::parse("[kirby.jump.first]\nstart_delay=0\nactions=short_hop\n[kirby.jump.second]\nstart_delay=0\nactions=nair\n[bowser.roll.roll]\nstart_delay=0\nreaction=auto_roll:30:12\nactions=up_b:4,drift:opponent:1:69\n[bowser.roll.spot]\nstart_delay=0\nreaction=auto_spot_dodge:25:10\nactions=up_b:4\n[bowser.roll.air]\nstart_delay=0\nreaction=auto_air_dodge:35:19\nactions=up_b:4\n[bowser.roll.dodge_grab]\nstart_delay=0\nreaction=auto_grab_dodged:25:10\nactions=up_b:4\n[yoshi.roll.custom_spot]\nstart_delay=0\nreaction=auto_spot_dodge:10:3\nactions=down_smash:1\n").unwrap();
        let mut p=FighterProfiles::default();p.add_sequence_choices(&c).unwrap();p
    });
    let presets = &profiles.profile(15).unwrap().menu_options;
    let rows = [
        MenuRow::Neutral,
        MenuRow::Jump,
        MenuRow::Roll,
        MenuRow::Attack,
        MenuRow::Drop,
        MenuRow::XSequence,
        MenuRow::YSequence,
        MenuRow::DpadUp,
        MenuRow::DpadDown,
        MenuRow::DpadLeft,
        MenuRow::DpadRight,
        MenuRow::L3Sequence,
        MenuRow::R3Sequence,
    ];
    let mut source = String::new();
    for slot in [
        "neutral",
        "jump",
        "roll",
        "attack",
        "drop",
        "x",
        "y",
        "dpad_up",
        "dpad_down",
        "dpad_left",
        "dpad_right",
        "l3",
        "r3",
    ] {
        source.push_str(&format!(
            "[bowser.{slot}.editor_save]\nstart_delay=2\nactions=shield:4,grab\n"
        ));
    }
    let events = [
        "auto_roll",
        "auto_spot_dodge",
        "auto_air_dodge",
        "auto_grab",
        "auto_shield_grab",
        "auto_grab_dodged",
        "auto_shield_backup",
    ];
    for event in events {
        source.push_str(&format!(
            "[bowser.roll.{event}]\nstart_delay=0\nreaction={event}:10:12\nactions=up_b:4\n"
        ));
    }
    source.push_str("[bowser.x.builtin_normal]\nbuiltin=normal\n");
    let catalog = Catalog::parse(&source).unwrap();
    JOBS.lock().unwrap().clear();
    *save::TEST_OPERATION.lock().unwrap() = Some(record);
    let _hook = Hook;
    let sample = |menu: &mut LedgeMenu, buttons| {
        menu.input(
            true,
            false,
            true,
            false,
            false,
            Some(15),
            buttons,
            buttons,
            ControlLayout::Default,
            &LedgeSettings::default(),
            true,
            true,
            true,
            true,
        );
    };
    for (row, choice) in rows
        .into_iter()
        .map(|r| (r, "editor_save"))
        .chain(events.into_iter().map(|e| (MenuRow::Roll, e)))
        .chain([(MenuRow::XSequence, "builtin_normal")])
    {
        let mut selection = MenuSelection::new(presets);
        selection.load_saved_recipes(presets, &catalog, 15);
        selection.input_with_editing(0, presets, true);
        selection.show();
        selection.row = row;
        let slot = row.slot().unwrap();
        let index = (0..128)
            .find(|&i| {
                selection
                    .choice(presets, slot, i)
                    .is_some_and(|c| c.id == choice)
            })
            .unwrap();
        selection.choices[slot as usize] = index;
        selection.input_with_editing(MENU_STICK_CLICK, presets, true);
        selection.input_with_editing(0, presets, true);
        assert!(selection.recipe_editing, "{row:?} / {choice}");
        let mut menu = LedgeMenu {
            kind: Some(15),
            selections: [(15, selection)].into(),
            ..Default::default()
        };
        let before = JOBS.lock().unwrap().len();
        sample(&mut menu, MAPPER_Y);
        assert!(
            menu.selections[&15]
                .save_status
                .as_deref()
                .unwrap()
                .contains("Saving now"),
            "{row:?} / {choice}"
        );
        for _ in 0..2000 {
            menu.poll_save_result();
            if !save::busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(JOBS.lock().unwrap().len(), before + 1, "{row:?} / {choice}");
        assert!(!save::busy());
        assert_eq!(JOBS.lock().unwrap().last().unwrap().0, choice);
        // A held Y must not submit a second write after completion.
        sample(&mut menu, MAPPER_Y);
        assert_eq!(JOBS.lock().unwrap().len(), before + 1);
    }
    let mut selection = MenuSelection::new(presets);
    selection.input_with_editing(0, presets, true);
    selection.show();
    selection.row = MenuRow::FighterSettings;
    selection.tuning_page = true;
    selection.tuning.set("walk_strength", "0.4").unwrap();
    let mut menu = LedgeMenu {
        kind: Some(15),
        selections: [(15, selection)].into(),
        ..Default::default()
    };
    sample(&mut menu, MAPPER_Y);
    assert!(menu.selections[&15]
        .save_status
        .as_deref()
        .unwrap()
        .contains("Saving now"));
    for _ in 0..2000 {
        menu.poll_save_result();
        if !save::busy() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(JOBS.lock().unwrap().last().unwrap().2);
    assert!(!save::busy());
}
