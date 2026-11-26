use super::*;

#[test]
/// Ensures toggle labels correspond to their selected indices.
fn toggle_rows_keep_matching_labels_and_positions() {
    assert_eq!(toggle_value(false), ("OFF".to_owned(), 0, 2));
    assert_eq!(toggle_value(true), ("ON".to_owned(), 1, 2));
}
