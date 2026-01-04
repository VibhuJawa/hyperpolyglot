//! Python bindings for hyperpolyglot language detection.

use std::io::BufReader;
use pyo3::prelude::*;

const MAX_CONTENT_SIZE_BYTES: usize = 51200;

// function stolen from from https://doc.rust-lang.org/nightly/src/core/str/mod.rs.html
fn truncate_to_char_boundary(s: &str, mut max: usize) -> &str {
    if max >= s.len() {
        s
    } else {
        while !s.is_char_boundary(max) {
            max -= 1;
        }
        &s[..max]
    }
}

fn truncate(s: &str) -> &str {
    truncate_to_char_boundary(s, MAX_CONTENT_SIZE_BYTES)
}

fn language(
    filename: Option<&str>,
    content: &str,
) -> Result<Option<hyperpolyglot::Detection>, std::io::Error> {
    let candidate = filename
        .and_then(|filename| hyperpolyglot::detectors::get_language_from_filename(filename));
    if let Some(candidate) = candidate {
        return Ok(Some(hyperpolyglot::Detection::Filename(candidate)));
    };

    let extension = filename.and_then(|filename| hyperpolyglot::detectors::get_extension(filename));

    let candidates = extension
        .map(|ext| hyperpolyglot::detectors::get_languages_from_extension(ext))
        .unwrap_or_else(Vec::new);

    if candidates.len() == 1 {
        return Ok(Some(hyperpolyglot::Detection::Extension(candidates[0])));
    };

    let mut reader = BufReader::new(content.as_bytes());

    let candidates = hyperpolyglot::filter_candidates(
        candidates,
        hyperpolyglot::detectors::get_languages_from_shebang(&mut reader)?,
    );
    if candidates.len() == 1 {
        return Ok(Some(hyperpolyglot::Detection::Shebang(candidates[0])));
    };

    let content = truncate(content);

    let candidates = if candidates.len() > 1 {
        if let Some(extension) = extension {
            let languages = hyperpolyglot::detectors::get_languages_from_heuristics(
                &extension[..],
                &candidates,
                &content,
            );
            hyperpolyglot::filter_candidates(candidates, languages)
        } else {
            candidates
        }
    } else {
        candidates
    };

    match candidates.len() {
        1 => Ok(Some(hyperpolyglot::Detection::Heuristics(candidates[0]))),
        _ => Ok(Some(hyperpolyglot::Detection::Classifier(
            hyperpolyglot::detectors::classify(&content, &candidates),
        ))),
    }
}

/// Detect languages for a batch of files.
#[pyfunction]
fn detect_languages(
    contents: Vec<Option<String>>,
    filenames: Vec<Option<String>>,
) -> (Vec<Option<String>>, Vec<Option<String>>) {
    contents
        .iter()
        .zip(filenames.iter())
        .map(|(content_opt, filename_opt)| {
            if let Some(content) = content_opt {
                match language(filename_opt.as_deref(), content) {
                    Ok(Some(detection)) => (
                        Some(detection.language().to_string()),
                        Some(detection.variant().to_string()),
                    ),
                    _ => (None, None),
                }
            } else {
                (None, None)
            }
        })
        .unzip()
}

#[pymodule]
fn _py_hyperpolyglot(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(pyo3::wrap_pyfunction!(detect_languages, m)?)?;
    Ok(())
}
