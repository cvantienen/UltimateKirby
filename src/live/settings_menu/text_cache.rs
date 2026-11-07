//! Explicit owned C-string copies for cached UI. Never reuse a Vec<String>
//! allocation as Vec<CString> or transfer the source strings' byte allocations.
use std::{borrow::Cow, ffi::CString};

pub(super) fn copy_text(text: &str) -> Result<Option<CString>, &'static str> {
    if text.as_bytes().contains(&0) {
        return Ok(None);
    }
    let size = text.len().checked_add(1).ok_or("Menu text is too large")?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| "Not enough memory for menu text")?;
    bytes.extend_from_slice(text.as_bytes());
    bytes.push(0);
    // Safe constructor, including empty strings: every retained entry owns at
    // least the NUL byte. Exact capacity avoids a shrink/reallocation step.
    CString::from_vec_with_nul(bytes)
        .map(Some)
        .map_err(|_| "Invalid menu text")
}
pub(super) fn copy_lines(
    lines: &[String],
    show_hints: bool,
    strip_hints: fn(String) -> Option<String>,
) -> Result<Vec<CString>, &'static str> {
    let mut cached = Vec::new();
    cached
        .try_reserve_exact(lines.len())
        .map_err(|_| "Not enough memory for menu rows")?;
    for source in lines {
        let text = if show_hints {
            Cow::Borrowed(source.as_str())
        } else {
            let Some(filtered) = strip_hints(source.clone()) else {
                continue;
            };
            Cow::Owned(filtered)
        };
        if let Some(line) = copy_text(&text)? {
            cached.push(line);
        }
    }
    Ok(cached)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn strip(s: String) -> Option<String> {
        (!s.starts_with("Y:")).then_some(s)
    }
    #[test]
    fn copies_empty_unicode_error_and_filtered_rows_without_aliasing_sources() {
        let lines = vec![
            "".into(),
            "Save failed: SD unavailable (514)".into(),
            "Saving…".into(),
            "Y: Save".into(),
            "invalid\0text".into(),
        ];
        let result = copy_lines(&lines, false, strip).unwrap();
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
    fn repeated_save_status_cache_replacement_does_not_reuse_source_storage() {
        let mut view = Some(copy_lines(&["Before save".into()], true, strip).unwrap());
        for i in 0..2000 {
            let source = vec![
                format!("Field {i}"),
                String::new(),
                "Saved to sequences.txt (backup created)".into(),
                "Save failed: temporary file exists".into(),
            ];
            let next = copy_lines(&source, true, strip).unwrap();
            let retired = view.replace(next);
            drop(source);
            drop(retired);
            for line in view.as_ref().unwrap() {
                assert_eq!(line.as_bytes_with_nul().last(), Some(&0));
            }
        }
    }
}
