# Finnish interface

> **Last reviewed:** 2026-10-22 · **Last updated:** 2026-10-22 · **Change:** minor (Finnish interface integrated with embedded Voikko) · **Target:** WordCraft 0.10.0

Choose **File › Options › Interface language › Suomi**, or follow a Finnish system display
language (`fi-FI`, `fi_FI.UTF-8`). The catalog code is `fi`; `ui.language` with `{"value":"fi"}`
selects it explicitly, and `WORDCRAFT_LANGUAGE=fi-FI` does the same for a headless screenshot or a
launch while the preference is `auto`. The choice is saved with the other interface settings.
Document text, command ids, CLI/MCP/control responses and file paths do not change.

`crates/ui-egui/src/i18n/fi.tsv` has 2,750 entries: every key of every other catalog, plus four
regional German proofing strings (`German (Austria)`, `German (Switzerland)`, the proofing-language
hint and the missing-dictionary message) that the other catalogs do not carry yet. It is written
from the meaning of the English labels; no other product's Finnish interface, string table or
terminology database was consulted, and no other catalog was used as a source. The contribution is
an original translation by @jernbole-dot (#2740), pending native-speaker review. Placeholders, `\n` breaks in ribbon labels
and dialog ellipses are kept, and strings added later fall back to English until they are
translated.

The source is [PR #2807](https://github.com/storytold/wordcraft/pull/2807) at
`f0368c4181fcd27292256a902006ff60de3caab6` (`maxm11:i18n/finnish-2740`). It remained
open and separate from upstream main when fetched on 2026-10-22. This fork merges
that branch and adds 72 original translations for newer 0.10.0 labels.

## Conventions

- Commands are imperatives (`Tallenna`, `Lisää taulukko`); messages address the reader as `sinä`
  (`Voit vaihtaa sen…`). Labels use sentence case; only the first word is capitalised.
- Counts use count-neutral wording: `Sanoja: {words}` reads right for `0`, `1`, `21` and every
  other number, so the catalog needs no plural forms.
- Decimal comma in fixed labels (`Riviväli 1,5`) and Finnish quotation marks (`”…”`).
- Product, format and technology names stay as they are: WordCraft, PDF, DOCX, Markdown, MCP,
  Zotero, SmartArt, WordArt, the Craft theme, paper sizes (Letter, Legal …) and citation styles
  (APA, MLA …).

## Terminology

| Source | Finnish |
|---|---|
| File / Home / Insert / Draw / Design / Layout | Tiedosto / Aloitus / Lisää / Piirrä / Rakenne / Asettelu |
| References / Mailings / Review / View / Help | Viittaukset / Postitukset / Tarkista / Näytä / Ohje |
| Font / Paragraph / Styles | Fontti / Kappale / Tyylit |
| Header / Footer / Heading | Ylätunniste / Alatunniste / Otsikko |
| Table of Contents / Index / Bibliography | Sisällysluettelo / Hakemisto / Lähdeluettelo |
| Comment / Track Changes / Accept / Reject | Kommentti / Muutosten seuranta / Hyväksy / Hylkää |
| Mail Merge / Recipients / Merge Field | Joukkokirje / Vastaanottajat / Yhdistämiskenttä |
| Columns / Section / Section Break | Sarakkeet / Osa / Osanvaihto |
| Undo / Redo / Repeat | Kumoa / Tee uudelleen / Toista |
| Ribbon / Pane / Quick Access Toolbar | Valintanauha / Ruutu / Pikatyökalurivi |
| Proofing / Word Count | Oikoluku / Sanamäärä |
| Page {page} of {pages} | Sivu {page}/{pages} |
| {words} words | Sanoja: {words} |

## Verification and remaining scope

`cargo test -p wordcraft-ui-egui --lib i18n::tests` covers Finnish locale tags (`fi`, `fi-FI`,
`fi_FI.UTF-8`, `FI-fi`) and preference order, saving and restoring `fi`, unchanged document content,
every key of every other catalog, no catalog errors, the command-search example, page and count
wording, English fallback, and the letters `ÄÖÅäöåŠšŽž` plus every other accented letter the catalog
uses in the bundled Inter (Regular, Medium, SemiBold) and JetBrains Mono. The shared catalog checks
(validation, every tab and command translated) include Finnish automatically, and the on-demand
interface audit (`untranslated_interface_text`) checks Finnish alongside the other complete
catalogs.

This build already includes the [Voikko contribution](voikko.md). Finnish UI sets
`editing_lang` to `fi-FI`: an untouched blank document and subsequent new documents
use Finnish spelling, suggestions, grammar and hyphenation without a separate
installation. Existing documents retain their language, and a saved default chosen
in Review › Language takes precedence over the UI language. Tests exercise an actual
Finnish misspelling and Voikko status after the UI logic has applied the setting.
A native-speaker review of the catalog and the 72 new translations remains welcome.

The combined 0.10.0 build passed `cargo +1.95.0 xtask ci` on Linux x86_64:
3,932 tests passed, none failed, four upstream tests remained ignored. Formatting,
strict Clippy, asset attribution, dependency layering and WebAssembly checks all
passed. This includes the rendered-interface translation audit.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-22 | minor | Merged PR #2807 into the 0.10.0 Voikko build; 72 current-label translations, 2,750 entries, blank-document proofing and preference-precedence checks |
| 2026-10-18 | minor | Finnish catalog (2,620 entries: every key of every other catalog plus the regional German proofing strings), conventions, terminology, scope and regression checks (#2740) |
