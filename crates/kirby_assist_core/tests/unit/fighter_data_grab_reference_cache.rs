use super::*;

/// Regression test for `prepared_references_match_uncached_data_for_every_kind_and_status`.
#[test]
fn prepared_references_match_uncached_data_for_every_kind_and_status() {
    let expected = all()
        .iter()
        .flat_map(|fighter| {
            [
                STATUS_GRAB,
                STATUS_DASH_GRAB,
                STATUS_PIVOT_GRAB,
                crate::native_states::ESCAPE,
            ]
            .map(|status| {
                (
                    fighter.kind,
                    status,
                    ordinary_grab_uncached(fighter.kind, status),
                )
            })
        })
        .collect::<Vec<_>>();
    prepare_grab_references().unwrap();
    for (kind, status, expected) in expected {
        assert_eq!(ordinary_grab(kind, status), expected);
    }
    for kind in [-1, 0x77, i32::MAX] {
        assert_eq!(
            ordinary_grab(kind, crate::native_states::CATCH),
            ordinary_grab_uncached(kind, crate::native_states::CATCH)
        );
    }
}
