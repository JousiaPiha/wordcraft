//! Finnish proofing in Rust, with a pinned dictionary embedded on every target.
//! No system library, filesystem lookup, network request, or C interface is used.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use voikko_fi::handle::VoikkoHandle;

const MORPHOLOGY: &[u8] = include_bytes!("../../../assets/voikko/mor.vfst");
const AUTOCORRECT: &[u8] = include_bytes!("../../../assets/voikko/autocorr.vfst");
const MAX_WORD_CHARS: usize = 255;
const MAX_PARAGRAPH_BYTES: usize = 100_000;
const MAX_SUGGESTIONS: usize = 32;
const MAX_ISSUES: usize = 500;

/// Backend initialization or input validation failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}

/// Grammar result with checked UTF-8 byte offsets, ready for editor selections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrammarIssue {
    pub code: i32,
    pub start: usize,
    pub end: usize,
    pub message: String,
    pub suggestions: Vec<String>,
}

/// One thread's Finnish checker. Callers share neither its cache nor its handle.
///
/// The only dictionary input is our trusted, pinned embedded data. Recoverable
/// dependency panics disable the handle; this guard is a last resort, not a
/// substitute for bounded inputs and regression tests. Targets with aborting
/// panics (including wasm32-unknown-unknown) cannot recover from a panic.
pub struct Voikko {
    handle: Option<VoikkoHandle>,
}

impl Voikko {
    pub fn new() -> Result<Self, Error> {
        let handle = catch_unwind(|| VoikkoHandle::from_bytes(MORPHOLOGY, Some(AUTOCORRECT), "fi"))
            .map_err(|_| Error("Finnish dictionary initialization failed".into()))?
            .map_err(|e| Error(format!("Finnish dictionary initialization failed: {e}")))?;
        Ok(Self { handle: Some(handle) })
    }

    fn run<T>(&mut self, f: impl FnOnce(&mut VoikkoHandle) -> T) -> Result<T, Error> {
        let handle = self.handle.as_mut().ok_or_else(|| Error("Finnish checker was disabled after an internal error".into()))?;
        match catch_unwind(AssertUnwindSafe(|| f(handle))) {
            Ok(value) => Ok(value),
            Err(_) => {
                self.handle = None;
                Err(Error("Finnish checker encountered an internal error".into()))
            }
        }
    }

    /// A disabled handle must not be reported as a successful check.
    pub fn status(&self) -> Result<(), Error> {
        if self.handle.is_some() { Ok(()) } else { Err(Error("Finnish checker was disabled after an internal error".into())) }
    }

    pub fn spell(&mut self, word: &str) -> Result<bool, Error> {
        validate_word(word)?;
        self.run(|v| v.spell(word))
    }

    pub fn suggest(&mut self, word: &str, max: usize) -> Result<Vec<String>, Error> {
        validate_word(word)?;
        if max == 0 {
            return Ok(Vec::new());
        }
        self.run(|v| {
            v.set_max_suggestions(max.min(MAX_SUGGESTIONS));
            v.suggest(word)
        })
    }

    /// Character indices before which an insertion-only hyphen is valid.
    /// Replacement markers (`=`) are omitted: layout cannot replace a glyph.
    pub fn hyphenate(&mut self, word: &str) -> Result<Vec<usize>, Error> {
        validate_word(word)?;
        let pattern = self.run(|v| v.hyphenate(word))?;
        if pattern.len() != word.chars().count() || !pattern.bytes().all(|b| matches!(b, b' ' | b'-' | b'=')) {
            return Err(Error("Voikko returned an invalid hyphenation pattern".into()));
        }
        Ok(pattern.bytes().enumerate().filter_map(|(i, c)| (c == b'-' && i > 0).then_some(i)).collect())
    }

    pub fn grammar(&mut self, text: &str) -> Result<Vec<GrammarIssue>, Error> {
        if text.len() > MAX_PARAGRAPH_BYTES || text.contains('\0') {
            return Err(Error("Paragraph exceeds Voikko's input limit or contains NUL".into()));
        }
        // Do not send enormous individual tokens into morphology through grammar.
        // Splitting would invent sentence boundaries, so leave that paragraph unchecked.
        if text.split_whitespace().any(|word| word.chars().count() > MAX_WORD_CHARS) {
            return Err(Error("Paragraph contains a word exceeding Voikko's input limit".into()));
        }
        let errors = self.run(|v| v.grammar_errors(text))?;
        let offsets: Vec<usize> = text.char_indices().map(|(i, _)| i).chain(std::iter::once(text.len())).collect();
        Ok(errors
            .into_iter()
            .take(MAX_ISSUES)
            .filter_map(|e| {
                let end_index = e.start_pos.checked_add(e.error_len)?;
                let start = *offsets.get(e.start_pos)?;
                let end = *offsets.get(end_index)?;
                if start >= end {
                    return None;
                }
                let description = voikko_core::grammar_error::error_code_description_en(e.error_code);
                Some(GrammarIssue {
                    code: e.error_code,
                    start,
                    end,
                    message: if description.is_empty() { "Finnish grammar issue".into() } else { description.into() },
                    suggestions: e.suggestions.into_iter().take(MAX_SUGGESTIONS).collect(),
                })
            })
            .collect())
    }
}

fn validate_word(word: &str) -> Result<(), Error> {
    if word.is_empty() || word.len() > MAX_WORD_CHARS * 4 || word.chars().count() > MAX_WORD_CHARS || word.contains('\0') {
        Err(Error("Word is empty, contains NUL or exceeds Voikko's input limit".into()))
    } else {
        Ok(())
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn real_finnish_operations() {
        let mut v = Voikko::new().unwrap();
        assert!(v.spell("talossanikinko").unwrap());
        assert!(!v.spell("koirra").unwrap());
        assert!(v.suggest("koirra", 6).unwrap().contains(&"koira".to_string()));
        assert_eq!(v.hyphenate("kissa").unwrap(), vec![3]);
        let text = "Äiti on on kotona.";
        let issues = v.grammar(text).unwrap();
        let duplicate = issues.iter().find(|e| e.code == 8).unwrap();
        assert_eq!((duplicate.start, duplicate.end), (6, 11));
        assert_eq!(duplicate.suggestions, vec!["on"]);
        for issue in issues {
            assert!(text.get(issue.start..issue.end).is_some());
            assert!(!issue.message.is_empty());
        }
    }

    #[test]
    fn bounded_inputs_and_suggestion_count() {
        let mut v = Voikko::new().unwrap();
        for word in [String::new(), "bad\0word".into(), "ä".repeat(256), "😀".repeat(256)] {
            assert!(v.spell(&word).is_err());
            assert!(v.suggest(&word, 6).is_err());
            assert!(v.hyphenate(&word).is_err());
        }
        assert!(v.grammar("a\0b").is_err());
        assert!(v.grammar(&"ä".repeat(50_001)).is_err());
        assert!(v.grammar(&"x".repeat(256)).is_err());
        assert!(v.grammar("").unwrap().is_empty());
        assert!(v.suggest("koirra", 0).unwrap().is_empty());
        assert!(v.suggest("koirra", usize::MAX).unwrap().len() <= 32);
        assert!(v.spell("koira").unwrap()); // invalid requests do not poison the checker
    }

    #[test]
    fn unicode_ranges_and_sentence_boundaries() {
        let mut v = Voikko::new().unwrap();
        for text in ["😀 Äiti on on kotona.", "Äiti on on kotona. Isä on on töissä.", "äiti on kotona."] {
            let errors = v.grammar(text).unwrap();
            assert!(!errors.is_empty(), "{text}");
            for e in errors {
                assert!(text.get(e.start..e.end).is_some(), "{e:?}");
            }
        }
        assert!(v.grammar("Äiti on kotona.").unwrap().is_empty());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        fn arbitrary_unicode_never_panics(text in ".{0,64}") {
            let mut v = Voikko::new().unwrap();
            // Errors are allowed; a dependency panic recovered by our last-resort
            // guard still fails this test by disabling the handle.
            let _ = v.spell(&text);
            let _ = v.hyphenate(&text);
            let _ = v.suggest(&text, 3);
            if let Ok(issues) = v.grammar(&text) {
                for e in issues {
                    prop_assert!(text.get(e.start..e.end).is_some());
                }
            }
            prop_assert!(v.status().is_ok());
        }
    }
}
