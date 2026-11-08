// Included in the native menu and host tests so both compile this exact publication path.
pub(super) fn publish(
    profile: &FighterProfile,
    selection: &MenuSelection,
    auto_mash: bool,
) -> bool {
    #[cfg(feature = "diagnostic-logging")]
    crate::performance::health::event(crate::performance::health::Event::MenuPublish);
    LIGHTWEIGHT_REQUESTED.store(selection.lightweight, Ordering::Release);
    REQUESTED_FIGHTER.store(profile.fighter_kind, Ordering::Release);
    REQUESTED_REVISION.store(selection.revision, Ordering::Release);
    let was_visible = MENU_VISIBLE.swap(!selection.menu_hidden, Ordering::AcqRel);
    if selection.menu_hidden {
        // Hidden binding/preview changes do not need snapshot/save locks once
        // the last visible view has been released. Keep deferred cleanup when
        // a save worker or renderer temporarily owns the relevant lock.
        if was_visible {
            DEFERRED_CLEAR.store(true, Ordering::Release);
        }
        finish_deferred_clear();
        return !DEFERRED_CLEAR.load(Ordering::Acquire);
    }
    let auto_mash = selection.effective_mode(
        kirby_assist_core::button_bindings::Mode::AutoMash,
        auto_mash,
    );
    let generation = VIEW_GENERATION.load(Ordering::Acquire);
    // Cursor refreshes do not allocate or retire text, so they remain safe
    // while a save worker blocks replacement of the cached strings.
    if selection.lightweight {
        if let Ok(mut cached) = VIEW.try_lock() {
            if cached
                .as_mut()
                .is_some_and(|v| v.refresh_cursor(profile, selection, auto_mash, generation))
            {
                DEFERRED_CLEAR.store(false, Ordering::Release);
                return true;
            }
        }
    }
    let Some(_save_gate) = super::save::try_ui_edit() else {
        #[cfg(feature = "diagnostic-logging")]
        crate::performance::health::event(crate::performance::health::Event::MenuRetry);
        return false;
    };
    let Ok(mut view) = VIEW.try_lock() else {
        #[cfg(feature = "diagnostic-logging")]
        crate::performance::health::event(crate::performance::health::Event::MenuRetry);
        return false;
    };
    DEFERRED_CLEAR.store(false, Ordering::Release);
    #[cfg(feature = "diagnostic-logging")]
    let _build_time =
        crate::performance::health::Trace::new(crate::performance::health::Lane::MenuBuild);
    #[cfg(feature = "diagnostic-logging")]
    crate::performance::health::event(crate::performance::health::Event::MenuBuild);
    let Ok(next) = view::build(profile, selection, auto_mash, generation) else {
        return false;
    };
    // Publish a complete replacement before retiring the old snapshot.
    let retired = view.replace(next);
    #[cfg(feature = "diagnostic-logging")]
    if let Some(v) = view.as_ref() {
        let bytes = std::mem::size_of::<Snapshot>()
            + v.navigation
                .as_ref()
                .map_or(0, |s| s.as_bytes_with_nul().len())
            + v.values.capacity() * std::mem::size_of::<SavedChoice>()
            + v.values
                .iter()
                .map(|r| r.line.as_bytes_with_nul().len())
                .sum::<usize>()
            + [&v.header, &v.body, &v.footer]
                .into_iter()
                .map(|lines| {
                    lines.capacity() * std::mem::size_of::<CString>()
                        + lines
                            .iter()
                            .map(|s| s.as_bytes_with_nul().len())
                            .sum::<usize>()
                })
                .sum::<usize>();
        crate::performance::health::view_bytes(bytes);
    }
    drop(view);
    drop(retired);
    true
}
