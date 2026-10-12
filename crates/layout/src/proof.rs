//! Shared proofing spans for the rendered page and review commands.

use std::ops::Range;
use wordcraft_doc::{CharProps, Paragraph, styles::StyleSheet};
use wordcraft_proof::{Issue, language};

#[derive(Clone, Debug)]
pub struct Span {
    pub range: Range<usize>,
    pub language: String,
}

/// Coalesce formatting-only boundaries, but never check across a language change
/// or excluded text. Native offsets are thus relative to intact UTF-8 slices.
pub fn spans(p: &Paragraph, styles: &StyleSheet, table: Option<&CharProps>) -> Vec<Span> {
    if p.text.len() > 100_000 {
        return Vec::new();
    }
    let ranges = p.run_ranges().filter_map(|(range, props)| {
        let rc = styles.resolve_char_in(p.props.style.as_deref(), table, props);
        if rc.no_proof || rc.hidden || rc.del.is_some() || rc.link.is_some() {
            return None;
        }
        Some((range, rc.lang.unwrap_or_else(|| "en-US".into())))
    });
    spans_from_ranges(p, ranges)
}

fn spans_from_ranges(p: &Paragraph, ranges: impl Iterator<Item = (Range<usize>, String)>) -> Vec<Span> {
    let mut out: Vec<Span> = Vec::new();
    for (range, lang) in ranges {
        let Some(text) = p.text.get(range.clone()) else { continue };
        for (offset, c) in text.char_indices() {
            if c == wordcraft_doc::para::OBJ || (c.is_control() && !c.is_whitespace()) {
                continue;
            }
            let start = range.start + offset;
            let end = start + c.len_utf8();
            if let Some(last) = out.last_mut()
                && last.range.end == start
                && last.language.eq_ignore_ascii_case(&lang)
            {
                last.range.end = end;
            } else {
                out.push(Span { range: start..end, language: lang.clone() });
            }
        }
    }
    // A boundary inside a token excludes that whole token, rather than checking
    // fragments such as "ello" when the H in Hello is marked no-proof.
    let words = wordcraft_proof::words(&p.text);
    out.into_iter()
        .filter_map(|mut span| {
            let at = words.partition_point(|&(_, end)| end <= span.range.start);
            if let Some(&(start, end)) = words.get(at)
                && start < span.range.start
            {
                span.range.start = end.min(span.range.end);
            }
            let at = words.partition_point(|&(_, end)| end < span.range.end);
            if let Some(&(start, end)) = words.get(at)
                && start < span.range.end
                && span.range.end < end
            {
                span.range.end = start.max(span.range.start);
            }
            (span.range.start < span.range.end).then_some(span)
        })
        .collect()
}

pub fn issues(p: &Paragraph, styles: &StyleSheet, table: Option<&CharProps>) -> Vec<Issue> {
    issues_in_spans(p, spans(p, styles, table), &wordcraft_proof::Options::default(), true)
}

/// Paragraphs containing Finnish use intact language spans, so grammar rules
/// cannot read across Finnish boundaries or hidden/deleted/no-proof runs.
pub(crate) fn issues_for_runs(p: &Paragraph, runs: &[crate::para::ProofRun], options: &wordcraft_proof::Options, grammar: bool) -> Vec<Issue> {
    let ranges = runs
        .iter()
        .filter(|r| !r.skip)
        .map(|r| (r.range.clone(), if r.lang == wordcraft_proof::ProofLang::Other { "x-none" } else { r.lang.code() }.into()));
    issues_in_spans(p, spans_from_ranges(p, ranges), options, grammar)
}

fn issues_in_spans(p: &Paragraph, spans: Vec<Span>, options: &wordcraft_proof::Options, grammar: bool) -> Vec<Issue> {
    let mut out = Vec::new();
    for span in spans {
        let Some(text) = p.text.get(span.range.clone()) else { continue };
        let mut issues = language::check_spelling_with(text, &span.language, options);
        let prefix = p.text.get(..span.range.start).unwrap_or("").trim_end();
        let sentence_start = prefix.is_empty() || prefix.ends_with(['.', '!', '?']);
        if grammar {
            issues.extend(language::check_grammar_in_context(text, &span.language, sentence_start, span.range.end == p.text.len()));
        }
        for mut issue in issues {
            if issue.start < issue.end && text.get(issue.start..issue.end).is_some() {
                issue.start += span.range.start;
                issue.end += span.range.start;
                out.push(issue);
            }
        }
    }
    out.sort_by_key(|i| i.start);
    out
}

pub fn language_at(p: &Paragraph, styles: &StyleSheet, table: Option<&CharProps>, offset: usize) -> String {
    styles.resolve_char_in(p.props.style.as_deref(), table, p.props_of_char(offset)).lang.unwrap_or_else(|| "en-US".into())
}

pub fn table_props(doc: &wordcraft_doc::Document, story: wordcraft_doc::StoryRef, path: &wordcraft_doc::Path) -> Option<CharProps> {
    let n = path.0.len().checked_sub(3)?;
    let table_path = wordcraft_doc::Path(path.0.get(..n)?.to_vec());
    let table = doc.table(story, &table_path)?;
    crate::table::proof_char_props(doc, table, *path.0.get(n)? as usize, *path.0.get(n + 1)? as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_doc::{CharProps, Paragraph, styles::StyleSheet};

    #[test]
    fn voikko_boundaries_preserve_polish_and_portuguese_sentence_context() {
        for (tag, word, capital) in [("pl-PL", "kot", "Kot"), ("pt-BR", "casa", "Casa")] {
            for (suffix, expected) in [(format!("{word}."), false), (format!("{word}. {word}."), true)] {
                let text = format!("Kissa {suffix}");
                let mut doc = wordcraft_doc::Document::from_text(&text);
                doc.styles.default_chr.lang = Some(tag.into());
                doc.format_range(&wordcraft_doc::Pos::body(0, 0), &wordcraft_doc::Pos::body(0, 5), &|c| c.lang = Some("fi-FI".into())).unwrap();
                let p = doc.para_at(&wordcraft_doc::Pos::body(0, 0)).unwrap();
                let runs = crate::para::proof_runs(&doc, p, None);
                let issues = crate::para::paragraph_issues(p, &runs, &wordcraft_proof::Options::default(), true);
                assert!(!issues.iter().any(|i| i.start == 6 && i.suggestions.contains(&capital.into())), "{issues:?}");
                assert_eq!(issues.iter().any(|i| i.start > 6 && i.suggestions.contains(&capital.into())), expected, "{issues:?}");
            }
        }
    }

    #[test]
    fn voikko_boundaries_do_not_feed_other_languages_grammar() {
        for (text, split) in [("On on.", 3), ("A äiti.", 2)] {
            let mut doc = wordcraft_doc::Document::from_text(text);
            doc.format_range(&wordcraft_doc::Pos::body(0, split), &wordcraft_doc::Pos::body(0, text.len()), &|c| c.lang = Some("fi-FI".into()))
                .unwrap();
            let p = doc.para_at(&wordcraft_doc::Pos::body(0, 0)).unwrap();
            let runs = crate::para::proof_runs(&doc, p, None);
            let issues = crate::para::paragraph_issues(p, &runs, &wordcraft_proof::Options::default(), true);
            assert!(!issues.iter().any(|i| i.kind == wordcraft_proof::IssueKind::Grammar && i.lang == wordcraft_proof::ProofLang::En), "{issues:?}");
        }
    }

    #[test]
    fn voikko_uses_upstream_preferences_and_exclusions() {
        let mut doc = wordcraft_doc::Document::from_text("KOIRRA koirra. Äiti on on kotona.");
        doc.styles.default_chr.lang = Some("fi-FI".into());
        let p = doc.para_at(&wordcraft_doc::Pos::body(0, 0)).unwrap();
        let runs = crate::para::proof_runs(&doc, p, None);
        let options = wordcraft_proof::Options::default();
        let issues = crate::para::paragraph_issues(p, &runs, &options, false);
        assert_eq!(issues.len(), 1);
        assert_eq!(&p.text[issues[0].start..issues[0].end], "koirra");
        assert_eq!(issues[0].lang, wordcraft_proof::ProofLang::Fi);
        let options = wordcraft_proof::Options { ignore_uppercase: false, ..options };
        assert_eq!(crate::para::paragraph_issues(p, &runs, &options, false).len(), 2);
        let mut runs = runs;
        runs[0].skip = true;
        assert!(crate::para::paragraph_issues(p, &runs, &options, true).is_empty());
    }

    #[test]
    fn same_language_runs_merge_and_excluded_text_is_skipped() {
        let mut styles = StyleSheet::default();
        styles.default_chr.lang = Some("fi-FI".into());
        let mut p = Paragraph::with_text("Äiti koirra", CharProps::default());
        p.format(0, 2, &|c| c.bold = Some(true)).unwrap();
        let v = spans(&p, &styles, None);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].language, "fi-FI");
        assert_eq!(v[0].range, 0..12);
        p.format(6, 12, &|c| c.no_proof = Some(true)).unwrap();
        assert_eq!(spans(&p, &styles, None)[0].range, 0..6);
    }

    #[test]
    fn unsupported_language_does_not_get_english_squiggles() {
        let styles = StyleSheet::default();
        let p = Paragraph::with_text("koirra", CharProps { lang: Some("sv-SE".into()), ..Default::default() });
        assert!(issues(&p, &styles, None).is_empty());
    }

    #[test]
    fn whitespace_keeps_grammar_context() {
        let styles = StyleSheet::default();
        let p = Paragraph::with_text("Hello\tworld.", CharProps::default());
        assert_eq!(issues(&p, &styles, None), wordcraft_proof::check_grammar(&p.text));
        let p = Paragraph::with_text("Hello\thello.", CharProps::default());
        assert!(issues(&p, &styles, None).iter().any(|i| i.kind == wordcraft_proof::IssueKind::Grammar));
    }

    #[test]
    fn excluded_text_does_not_create_word_fragments_or_sentence_starts() {
        let styles = StyleSheet::default();
        for end in [1, 5] {
            let mut p = Paragraph::with_text("Hello world.", CharProps::default());
            p.format(0, end, &|c| c.no_proof = Some(true)).unwrap();
            assert!(issues(&p, &styles, None).is_empty());
        }
        let mut p = Paragraph::with_text("Bonjour world.", CharProps::default());
        p.format(0, 7, &|c| c.lang = Some("fr-FR".into())).unwrap();
        assert!(issues(&p, &styles, None).is_empty());

        let mut p = Paragraph::with_text("Skip hello hello. world.", CharProps::default());
        p.format(0, 4, &|c| c.no_proof = Some(true)).unwrap();
        let v = issues(&p, &styles, None);
        assert!(v.iter().any(|i| i.suggestions.contains(&"hello".into())));
        assert!(v.iter().any(|i| i.suggestions.contains(&"World".into())));
    }

    #[test]
    fn voikko_exclusions_do_not_invent_sentence_boundary_errors() {
        let styles = StyleSheet::default();
        let mut p = Paragraph::with_text("Äiti on kotona.", CharProps { lang: Some("fi".into()), ..Default::default() });
        p.format(6, 8, &|c| c.no_proof = Some(true)).unwrap();
        assert!(issues(&p, &styles, None).is_empty());

        let mut p = Paragraph::with_text("Skip äiti on on kotona. äiti on kotona.", CharProps { lang: Some("fi".into()), ..Default::default() });
        p.format(0, 4, &|c| c.no_proof = Some(true)).unwrap();
        let v = issues(&p, &styles, None);
        assert!(v.iter().any(|i| i.suggestions.contains(&"on".into())));
        assert!(v.iter().any(|i| i.start == p.text.rfind("äiti").unwrap() && i.suggestions.contains(&"Äiti".into())));
    }

    #[test]
    fn voikko_mixed_language_spelling_and_grammar_offsets() {
        let styles = StyleSheet::default();
        let mut p = Paragraph::with_text("Hello. Äiti on on kotona. koirra", CharProps::default());
        p.format(7, p.text.len(), &|c| c.lang = Some("fi-FI".into())).unwrap();
        p.format(9, 11, &|c| c.bold = Some(true)).unwrap();
        let v = issues(&p, &styles, None);
        let duplicate = v.iter().find(|i| i.kind == wordcraft_proof::IssueKind::Grammar && i.suggestions.contains(&"on".into())).unwrap();
        assert_eq!(p.text.get(duplicate.start..duplicate.end), Some("on on"));
        assert_eq!(duplicate.start, 13);
        let spelling: Vec<_> = v.iter().filter(|i| i.kind == wordcraft_proof::IssueKind::Spelling).collect();
        assert_eq!(spelling.len(), 1);
        assert_eq!(p.text.get(spelling[0].start..spelling[0].end), Some("koirra"));
    }
}
