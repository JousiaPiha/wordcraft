//! Language-aware proofing and hyphenation.

use crate::{Issue, IssueKind};

/// Availability is separate from "no errors": missing dictionaries are not a
/// successful proofing result.
#[derive(Clone, Debug)]
pub struct Status {
    pub available: bool,
    pub backend: &'static str,
    pub reason: Option<String>,
}

fn english(language: &str) -> bool {
    crate::ProofLang::from_tag(Some(language)).is_english()
}

pub fn finnish(language: &str) -> bool {
    crate::ProofLang::from_tag(Some(language)) == crate::ProofLang::Fi
}

pub(crate) fn is_correct_finnish(word: &str) -> bool {
    let lower = word.to_lowercase();
    crate::in_user_dictionary(&lower)
        || crate::ignored_words().read().unwrap_or_else(|e| e.into_inner()).contains(&lower)
        || backend::spell(word) != Ok(false)
}

pub(crate) fn finnish_breaks(word: &str) -> Vec<usize> {
    backend::hyphenate(word)
}

pub fn status(language: &str) -> Status {
    if english(language) {
        Status { available: true, backend: "builtin-english", reason: None }
    } else if finnish(language) {
        let reason = backend::status().err();
        Status { available: reason.is_none(), backend: "voikko-rust", reason }
    } else {
        let available = match crate::ProofLang::from_tag(Some(language)) {
            crate::ProofLang::Pl => crate::pl::has_dictionary(),
            crate::ProofLang::Pt => crate::pt::has_dictionary(),
            crate::ProofLang::It => crate::it::has_dictionary(),
            crate::ProofLang::Hr => crate::hr::has_dictionary(),
            crate::ProofLang::Tr => crate::tr::has_dictionary(),
            crate::ProofLang::Ru => crate::ru::has_dictionary(),
            crate::ProofLang::Cs => crate::cs::has_dictionary(),
            crate::ProofLang::Id => crate::id::has_dictionary(),
            _ => return Status { available: false, backend: "none", reason: Some("No proofing dictionary for this language".into()) },
        };
        Status { available, backend: "hunspell", reason: (!available).then(|| "No installed Hunspell dictionary for this language".into()) }
    }
}

pub fn check_spelling(text: &str, language: &str) -> Vec<Issue> {
    check_spelling_with(text, language, &crate::Options::default())
}

pub fn check_spelling_with(text: &str, language: &str, options: &crate::Options) -> Vec<Issue> {
    if text.len() > 100_000 {
        return Vec::new();
    }
    let lang = crate::ProofLang::from_tag(Some(language));
    crate::spelling_issues(text, |_| lang, options)
}

pub fn check_grammar(text: &str, language: &str) -> Vec<Issue> {
    check_grammar_in_context(text, language, true, true)
}

/// A language/exclusion boundary need not be a sentence or paragraph boundary.
pub fn check_grammar_in_context(text: &str, language: &str, sentence_start: bool, paragraph_end: bool) -> Vec<Issue> {
    if finnish(language) {
        backend::grammar(text, sentence_start, paragraph_end)
    } else {
        crate::grammar_in(text, crate::ProofLang::from_tag(Some(language)), sentence_start)
    }
}

pub fn suggest(word: &str, language: &str, max: usize) -> Vec<String> {
    if finnish(language) { backend::suggest(word, max) } else { crate::suggest_in(word, max, crate::ProofLang::from_tag(Some(language))) }
}

/// Break points are character indices, matching the existing English API.
pub fn hyphenate(word: &str, language: &str, limits: &crate::hyphen::Limits) -> Vec<usize> {
    if !finnish(language) {
        let lang = crate::ProofLang::from_tag(Some(language));
        return if lang == crate::ProofLang::Other { Vec::new() } else { crate::hyphen::hyphen_points_in(word, limits, lang) };
    }
    let len = word.chars().count();
    if !finnish(language) || len < limits.min_word || (!limits.capitalized && word.chars().next().is_some_and(char::is_uppercase)) {
        return Vec::new();
    }
    backend::hyphenate(word).into_iter().filter(|&i| i >= limits.after_first && len.saturating_sub(i) >= limits.before_last).collect()
}

mod backend {
    use super::*;
    use std::cell::RefCell;
    use wordcraft_voikko::{Error, Voikko};

    thread_local! {
        static VOIKKO: RefCell<Result<Voikko, Error>> = RefCell::new(Voikko::new());
    }

    fn with<T>(f: impl FnOnce(&mut Voikko) -> Result<T, Error>) -> Result<T, Error> {
        VOIKKO.with(|cell| {
            let mut value = cell.try_borrow_mut().map_err(|_| Error("Voikko is busy".into()))?;
            match value.as_mut() {
                Ok(v) => f(v),
                Err(e) => Err(e.clone()),
            }
        })
    }
    pub fn status() -> Result<(), String> {
        with(|v| v.status()).map_err(|e| e.to_string())
    }
    pub fn spell(word: &str) -> Result<bool, Error> {
        with(|v| v.spell(word))
    }
    pub fn suggest(word: &str, max: usize) -> Vec<String> {
        with(|v| v.suggest(word, max)).unwrap_or_default()
    }
    pub fn hyphenate(word: &str) -> Vec<usize> {
        with(|v| v.hyphenate(word)).unwrap_or_default()
    }
    pub fn grammar(text: &str, sentence_start: bool, paragraph_end: bool) -> Vec<Issue> {
        let first_word = crate::words(text).first().map(|&(a, _)| a);
        with(|v| v.grammar(text))
            .unwrap_or_default()
            .into_iter()
            // Voikko's stable Finnish diagnostics: 7 = initial lowercase,
            // 9 = missing terminating punctuation. Do not turn a formatting or
            // language boundary into a new sentence, but keep internal errors.
            .filter(|i| !(i.code == 7 && !sentence_start && Some(i.start) == first_word) && (i.code != 9 || paragraph_end))
            .map(|i| Issue {
                start: i.start,
                end: i.end,
                kind: IssueKind::Grammar,
                message: i.message,
                suggestions: i.suggestions,
                lang: crate::ProofLang::Fi,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_english_variants_and_indonesian_remain_available() {
        for tag in ["en-US", "en-GB", "en-CA", "en-AU"] {
            assert!(status(tag).available, "{tag}");
        }
        crate::id::test_dict::install();
        let indonesian = status("id-ID");
        assert!(indonesian.available);
        assert_eq!(indonesian.backend, "hunspell");
    }

    #[test]
    fn upstream_hunspell_languages_remain_available() {
        crate::it::test_dict::install();
        crate::hr::test_dict::install();
        crate::ru::test_dict::install();
        crate::cs::test_dict::install();
        for tag in ["it-IT", "hr-HR", "ru-RU", "cs-CZ"] {
            let result = status(tag);
            assert!(result.available, "{tag}: {:?}", result.reason);
            assert_eq!(result.backend, "hunspell");
        }
        // Turkish tests also exercise the real installed dictionary when present.
        // Do not replace that shared dictionary with a synthetic partial word list.
        let turkish = status("tr-TR");
        assert_eq!(turkish.backend, "hunspell");
        assert_eq!(turkish.available, crate::tr::has_dictionary());
    }

    #[test]
    fn upstream_grammar_respects_language_boundaries() {
        for (tag, text) in [("it-IT", "questo questo."), ("hr-HR", "ovo ovo."), ("tr-TR", "ev ev."), ("ru-RU", "дом дом.")] {
            let issues = check_grammar_in_context(text, tag, false, true);
            assert!(
                issues.iter().any(|i| i.start == if tag == "tr-TR" { 3 } else { 0 } && i.end == text.len() - 1),
                "{tag}: repeated word must be checked: {issues:?}"
            );
            assert!(
                !issues.iter().any(|i| i.start == 0 && i.end == text.find(' ').unwrap()),
                "{tag}: mid-sentence span must not require a capital: {issues:?}"
            );
            let at_start = check_grammar(text, tag);
            assert!(
                at_start.iter().any(|i| i.start == 0 && i.end == text.find(' ').unwrap()),
                "{tag}: actual sentence start needs a capital: {at_start:?}"
            );
        }
    }

    #[test]
    fn czech_grammar_remains_available_in_mixed_paragraphs() {
        let text = "město město.";
        let issues = check_grammar_in_context(text, "cs-CZ", false, true);
        assert!(issues.iter().any(|i| i.start == 0 && i.end == text.len() - 1 && i.lang == crate::ProofLang::Cs), "{issues:?}");
    }

    #[test]
    fn finnish_spelling_keeps_upstream_internet_token_handling() {
        let text = "koirra@example.com,koirra koirra";
        let issues = check_spelling(text, "fi-FI");
        assert_eq!(issues.len(), 1);
        assert_eq!(text.get(issues[0].start..issues[0].end), Some("koirra"));
        assert_eq!(issues[0].start, 26);
        let options = crate::Options { ignore_internet: false, ..Default::default() };
        assert!(check_spelling_with(text, "fi-FI", &options).iter().any(|i| i.start == 0 && i.end == 6));
    }

    #[test]
    fn finnish_backend_is_built_in_and_ready() {
        let status = status("fi-FI");
        assert!(status.available, "{:?}", status.reason);
        assert_eq!(status.backend, "voikko-rust");
        assert!(suggest("koirra", "fi-FI", 6).contains(&"koira".into()));
    }

    #[test]
    fn finnish_uses_the_upstream_language_api() {
        let lang = crate::ProofLang::from_tag(Some("fi-FI"));
        assert_eq!(lang, crate::ProofLang::Fi);
        assert!(!crate::is_correct_in("koirra", lang));
        assert!(crate::suggest_in("koirra", 6, lang).contains(&"koira".into()));
        assert!(!crate::check_grammar_in("Äiti on on kotona.", lang).is_empty());
        assert_eq!(crate::hyphen::hyphen_points_in("kissa", &crate::hyphen::Limits::default(), lang), vec![3]);
        assert!(!crate::check_grammar_in("Wiem że to jest dobre.", crate::ProofLang::Pl).is_empty());
    }

    #[test]
    fn dispatch_does_not_apply_english_to_other_languages() {
        assert!(check_spelling("koirra", "sv-SE").is_empty());
        assert!(!status("sv-SE").available);
        let british = check_spelling("helo", "EN-gb");
        assert_eq!(british.len(), 1);
        assert_eq!((british[0].start, british[0].end, british[0].lang), (0, 4, crate::ProofLang::EnGb));
        assert!(check_grammar("a apple", "sv-SE").is_empty());
        assert!(hyphenate("university", "sv-SE", &crate::hyphen::Limits::default()).is_empty());
        assert_eq!(suggest("wrold", "", 3), crate::suggest("wrold", 3));
    }

    #[test]
    fn finnish_proofing_and_limits() {
        assert!(status("fi-FI").available);
        assert!(check_spelling("Äiti asuu talossa.", "fi").is_empty());
        let issues = check_spelling("Äiti ja koirra.", "fi-FI");
        assert_eq!(issues.len(), 1);
        assert_eq!((issues[0].start, issues[0].end), (9, 15));
        assert!(suggest("koirra", "fi-FI", 6).contains(&"koira".into()));
        assert!(suggest("koirra", "fi-FI", 0).is_empty());
        assert!(!check_grammar("Äiti on on kotona.", "fi").is_empty());
        assert_eq!(hyphenate("kissa", "fi", &crate::hyphen::Limits::default()), vec![3]);
        let limits = crate::hyphen::Limits { before_last: 3, ..Default::default() };
        assert!(hyphenate("kissa", "fi", &limits).is_empty());
        crate::add_word("voikkotestisanaqwerty");
        assert!(check_spelling("voikkotestisanaqwerty", "fi").is_empty());
    }
}
