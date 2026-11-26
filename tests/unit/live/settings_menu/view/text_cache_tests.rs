use super::*;
#[test]
/// Checks empty, Unicode, error, and embedded-NUL line handling and ownership.
fn copies_empty_unicode_and_error_rows_without_aliasing_sources() {
    let lines = vec![
        "".into(),
        "Save failed: SD unavailable (514)".into(),
        "Saving…".into(),
        "invalid\0text".into(),
    ];
    let result = copy_lines(&lines).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].as_bytes_with_nul(), b"\0");
    assert_eq!(result[1].to_str().unwrap(), lines[1]);
    assert_eq!(result[2].to_str().unwrap(), "Saving…");
    for (copy, source) in result.iter().zip(&lines) {
        assert_ne!(copy.as_ptr().cast::<u8>(), source.as_ptr());
    }
    drop(lines);
    assert_eq!(
        result[1].to_str().unwrap(),
        "Save failed: SD unavailable (514)"
    );
}
#[test]
/// Repeatedly replaces cached save-status lines without source-buffer reuse.
fn repeated_save_status_cache_replacement_does_not_reuse_source_storage() {
    let mut view = Some(copy_lines(&["Before save".into()]).unwrap());
    for i in 0..2000 {
        let source = vec![
            format!("Field {i}"),
            String::new(),
            "Saved to sequences.txt (backup created)".into(),
            "Save failed: temporary file exists".into(),
        ];
        let next = copy_lines(&source).unwrap();
        let retired = view.replace(next);
        drop(source);
        drop(retired);
        for line in view.as_ref().unwrap() {
            assert_eq!(line.as_bytes_with_nul().last(), Some(&0));
        }
    }
}
