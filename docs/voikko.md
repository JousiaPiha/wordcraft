# Finnish proofing with Voikko

> **Last reviewed:** 2026-10-22 · **Last updated:** 2026-10-22 · **Change:** minor (upstream 0.10.0 sync)

WordCraft includes Finnish spelling, correction suggestions, grammar checking and
hyphenation through the Rust implementation of Voikko. The engine and Finnish
dictionary are built into the application, including WebAssembly builds. There
is no Voikko package, DLL, Homebrew installation or dictionary download to manage.
No document text leaves the machine.

## Using Finnish

Select text and choose **Review → Language → Finnish**, or choose Finnish as the
document's default language in that dialog. The status-bar language menu also
includes Finnish. **Spelling & Grammar (F7)**, spelling underlines, correction
suggestions, and automatic/manual hyphenation use Voikko for Finnish text.
Language can vary within a paragraph and is inherited through styles and table
formatting. English, Polish, Portuguese, Italian, Croatian and Bulgarian keep their existing tools.

Ignore All, the user dictionary, spelling preferences, hidden/deleted text and
no-proof formatting are respected. Grammar checking does not treat a formatting
or language boundary as a new sentence. Language tags persist in saved documents.

`review.proofingStatus={"lang":"fi-FI"}` reports `backend: "voikko-rust"` and
`available: true` when initialization succeeds. Omit `lang` for the caret's
language. Review, suggestions, correction and manual hyphenation commands use
the same implementation as the editor. Unsupported languages remain unavailable.

## Implementation

`wordcraft-voikko` wraps the external `voikko-fi` and `voikko-core` Rust crates at
revision `beba2cd7c0b3839b63f54c18948c4e2add318bc1` of
[yongsk0066/corevoikko](https://github.com/yongsk0066/corevoikko).
The matching `mor.vfst` and `autocorr.vfst` files are embedded from `assets/voikko/`.
Both desktop and browser builds use the same API and data. A handle and cache are
initialized once per proofing thread. All WordCraft crates forbid unsafe code;
the previous C adapter and its local exception have been removed.

Inputs reject NUL, empty words, words over 255 Unicode scalars and paragraphs over
100,000 UTF-8 bytes. Grammar paragraphs containing oversized tokens are skipped
rather than split into artificial sentences. Suggestions are capped at 32 and
returned grammar issues at 500. Grammar character offsets become checked UTF-8
byte ranges. Hyphenation uses insertion markers only; replacement markers are
omitted because WordCraft cannot replace a glyph while laying out a line.

A last-resort panic guard disables a failed handle on targets with unwinding;
status then reports it unavailable. Aborting targets such as browser WASM cannot
recover from a dependency panic. Bounds, trusted dictionary data and Unicode
regressions are the primary safeguards, not this guard.

## Tests and platform checklist

No system Voikko installation is needed for any of these tests. Finnish tests
run in the normal suite rather than being ignored:

```sh
cargo test --locked -p wordcraft-voikko -p wordcraft-proof -p wordcraft-layout -p wordcraft-engine
cargo run --locked -p wordcraft-voikko --example smoke
cargo xtask ci
```

The `Finnish proofing` workflow runs the same tests on Linux x86_64, Windows
x86_64 and macOS Apple Silicon. It is prepared here; adding a workflow does not
mean those remote machines have already passed. The owner has confirmed that the
Apple Silicon Mac version works; Windows user testing remains pending.

Build the desktop and CLI on each machine with Rust 1.95.0 or newer. On Windows,
use Rust's MSVC toolchain with Visual Studio C++ Build Tools and the Windows SDK.
On Apple Silicon, install Xcode Command Line Tools and the native aarch64 Rust
toolchain. The [Rust installer](https://www.rust-lang.org/tools/install) explains
these prerequisites. Linux build packages are listed in the existing
`.github/workflows/printing.yml` workflow. No Voikko development tools are needed:

```sh
cargo build --release --locked -p wordcraft -p wordcraft-cli
```

Run `target/release/wordcraft` (`wordcraft.exe` on Windows). Installer scripts
remain `packaging/linux/package.sh`, `packaging/windows/package.ps1 -Arch x64`,
and `packaging/macos/package.sh --arch aarch64`; see their headers for packaging
tools and signing requirements. The Linux script needs nfpm for .deb generation.

For a short manual test on **each** target:

1. Set Finnish as the default language in a new document.
2. Type `Äiti ja koirra.`. Only `koirra` should have a spelling underline;
   choose `koira`, then undo and verify the original text returns.
3. Type `Äiti on on kotona.`. Grammar review should select `on on`, not part
   of `Äiti`; accept the proposed single `on`.
4. Type `Äiti on kotona.`. It should have no Finnish proofing marks.
5. Enable automatic hyphenation in a narrow column with repeated
   `kansantalous tietokone talossanikinko`. Check manual hyphenation too.
6. Mix an English sentence and a Finnish sentence, assigning their languages;
   bold part of a Finnish word and verify it is still checked as one word.
7. Save as DOCX, reopen, and verify language and correction behavior persist.
8. Repeat after disconnecting the network, with no system Voikko installed.

## Attribution and distribution

WordCraft's original integration retains MIT/Apache-2.0 licensing. The external
Rust library declares MPL-1.1 / GPL-2.0-or-later / LGPL-2.1-or-later alternatives;
this integration uses its MPL-1.1 option. The Finnish dictionary is separate
GPL-2.0-or-later data. See `assets/voikko/README.md`, its license files, `NOTICE`
and `ATTRIBUTION.md` for provenance, hashes and corresponding source links.
Packaging includes the notices in Linux, Windows, macOS and web outputs.

Before public distribution, provide the pinned corresponding source archive
alongside the binaries and have maintainers review the dependency and dictionary
licensing. No upstream acceptance, relicensing of WordCraft, or ongoing maintainer
role for this contribution is assumed.

## Local verification (2026-10-13)

Linux x86_64, Rust 1.95.0: 1,945 workspace tests passed, 0 failed, 2 unrelated
tests ignored. The new adapter passes strict Clippy; all 17 WASM-safe workspace
crates build. The `smoke` example was also compiled to wasm32-unknown-unknown and
executed in Node's WebAssembly runtime with no host imports: spelling, suggestions,
grammar UTF-8 ranges and hyphenation passed. This is a backend runtime test, not
a browser UI test. Asset attribution (59 files) and layering (21 crates) pass.

The full `cargo +1.95.0 xtask ci` command passes all six gates: formatting,
workspace Clippy, tests, asset attribution, dependency layering and WASM builds.
Small, behavior-preserving cleanups for Rust 1.95 Clippy are included in a separate
commit. Windows x86_64 execution and user acceptance remain pending. Apple Silicon
automated, visual and owner acceptance results follow.

### Apple Silicon macOS (2026-10-13)

Tested source and packaged binaries: `663f3a07d7392eefde2a80895ce0f88ab0910225`
on `feat/voikko-finnish`, initially a clean checkout. Machine: Apple M4 Pro,
macOS 26.6.2 (25G83), native `arm64`, `sysctl.proc_translated=0` (no Rosetta).
Rust: `rustc 1.95.0 (59807616e 2026-04-14)`, host
`aarch64-apple-darwin`; the compiler itself is ARM64. Installed the minimal
1.95.0 toolchain, rustfmt, Clippy and `wasm32-unknown-unknown`, and set a local
rustup directory override. Existing Xcode Command Line Tools were sufficient.
No system Voikko package or dictionary was installed.

The local `plan/` files and referenced `../../craftrules` checkout were absent.
The public-roadmap fallback was read. Attempts to read the shared craftrules
instructions through the public URL and authenticated GitHub API returned 404;
the repository's own instructions remained applicable.

**Automated checks:** the requested locked test command passed 800 tests;
the native Voikko `smoke` example passed. `cargo xtask ci` passed all six gates:
formatting, strict workspace Clippy, tests (1,944 passed, 0 failed, 2 ignored),
asset attribution (59 files), layering (21 crates), and WASM (17 crates).
WASM checking emitted existing unused-mutability/dead-code warnings; it completed
successfully. This is a WASM compile check, not a Mac browser runtime test.
No macOS-specific source or packaging fixes were required, and the dependency
pin and lockfile are unchanged.

Built with `MACOS_SIGN_IDENTITY=- bash packaging/macos/package.sh --arch aarch64`.
The app and CLI contain only `arm64`; strict code-signature verification passed
for the app, CLI and DMG, and `hdiutil verify` passed. Signing is ad hoc, with no
notarization. Dynamic dependencies contain no libvoikko or Homebrew library.
The mounted DMG and CLI ZIP both contain byte-identical copies of `NOTICE`,
`ATTRIBUTION.md`, and `voikko/{README.md,CONTRIBUTORS.txt,COPYING-dictionary.txt,
LICENSE-libvoikko.txt}` from the source checkout.

**Packaged-app checks:** ran the executable inside the read-only mounted DMG,
with `WORDCRAFT_NO_PREFS=1`. An additional per-process `sandbox-exec` policy
denied external networking while allowing the loopback control channel. A curl
probe under that policy failed to connect externally; the app initialized
`voikko-rust` and completed the checks below. The Mac's network and security
settings were not changed. All inputs used the documented control channel,
without global keystrokes. Screenshots captured only WordCraft's window and were
opened for visual inspection.

| Checklist item | Automated packaged-app result | Visual result |
| --- | --- | --- |
| 2026-10-22 | minor | Synced upstream 0.10.0 at `9130181a`; retained embedded Finnish proofing, English regional spelling, Indonesian availability and list-item grammar context |
| 2026-10-18 | minor | Synced upstream 0.8.0; preserved Finnish alongside new Czech, Russian and Turkish proofing and inherited Language dialog choices |
| 2026-10-14 | minor | Synced upstream 0.7.0; retained Finnish and integrated Italian/Croatian routing and upstream spelling optimization |
| Finnish default and availability | `fi-FI`, `voikko-rust`, available | Status bar displays Finnish |
| `Äiti ja koirra.` | Only spelling issue is `koirra`; `koira` offered; correction and undo pass, including clicking the visible context-menu suggestion | Red underline only on `koirra`; suggestion menu displays `koira` |
| `Äiti on on kotona.` | Grammar range is UTF-8 bytes 6–11; selection is exactly `on on`; correction gives `Äiti on kotona.` | Selection and grammar underline cover `on on`, not `Äiti` |
| `Äiti on kotona.` | No proofing issues | No spelling or grammar marks |
| Automatic hyphenation | Enabled in a narrow text area with repeated `kansantalous tietokone talossanikinko` | Turning it on adds Finnish line-end breaks, including `kansan-talous` and `tieto-kone` |
| Manual hyphenation | `kansantalous` points `[3,6,8]`; clicking Yes at 6 inserts U+00AD; survives DOCX reopen | Dialog shows `kan-san-ta-lous`; reopened page breaks at `kansan-talous` |
| Mixed languages and formatting | English sentence + Finnish sentence; bold `irr` inside `koirra`; whole word remains one Finnish spelling issue | English sentence unmarked; whole Finnish misspelling underlined |
| DOCX save/reopen | `en-US`/`fi-FI` run languages, bold formatting, issue range, suggestions, correction and undo preserved | Reopened text and spelling mark displayed correctly |
| Embedded dictionaries offline | All preceding app checks passed with external networking denied | Same proofing marks and dialogs displayed offline |

After correcting `Äiti ja koirra.` to `Äiti ja koira.`, the spelling mark clears,
but Voikko reports a missing-main-verb grammar warning on that sentence fragment.
That is distinct from the clean complete-sentence test `Äiti on kotona.`.

**Owner acceptance (2026-10-13):** after receiving the local DMG, the owner
reported: “Mac version works.” Mac personal acceptance is therefore confirmed.
This is an overall user report, not an item-by-item manual test record: the
Finder installation path, Language-menu selection, physical typing/F7, Finnish
linguistic quality on personal documents and physically disconnected-network use
were not individually reported. The automated and visual evidence above remains
separate. Windows x86_64 runtime, packaged-app testing and user acceptance remain
pending; broader Word parity is not established by these checks.

Local outputs, relative to the checkout:

- App: `target/macos-package/WordCraft.app`
- DMG: `dist/release/wordcraft-0.6.1-macos-aarch64.dmg`
- CLI ZIP: `dist/release/wordcraft-cli-0.6.1-macos-aarch64.zip`
- Logs, control replies, synthetic DOCX files and window screenshots:
  `target/voikko-macos-validation/` (gitignored).

The test instance was closed and its read-only DMG unmounted after verification.
No release or PR was published; no ongoing maintainer role is assumed.

## Upstream sync (2026-10-14)

Merged upstream `a124d58` (0.7.0, 70 commits after the previous base). Finnish
remains built in. The shared adapter now reports Italian and Croatian Hunspell
availability, checks their grammar without inventing sentence boundaries, and
uses the upstream spelling-token optimization. The updated language menu keeps
Finnish and the full language dialog. Regression tests cover the new backends
and Finnish internet-token handling.

Linux x86_64, Rust 1.95.0: the full `cargo +1.95.0 xtask ci` passes all six
gates after the merge, with 2,162 tests passed, no failures, one ignored test,
and all 17 WASM-safe crates checked. Three equivalent boolean-expression
cleanups in upstream code are included for Rust 1.95 Clippy. A focused review
found no important proofing merge regressions.

The owner also confirmed manual acceptance of the previous Linux x86_64 `.deb`.
The Mac acceptance above applies to the earlier tested commit; the merged version
has not been retested on Mac or Windows.

## Upstream sync (2026-10-18)

Merged upstream `f2e15cf` (0.8.0, 241 commits after the previous base). Finnish
remains embedded. The adapter includes upstream Czech, Russian and Turkish
availability and grammar. Russian and Turkish checks preserve sentence context
when a paragraph mixes these languages with Finnish. Regression tests cover
backend availability, false sentence-start capitals and UTF-8 issue offsets in
mixed-language paragraphs. The Language dialog keeps inherited proofing choices
and the Finnish integration's table-aware language resolution.

Linux x86_64, Rust 1.95.0: all six `cargo +1.95.0 xtask ci` gates pass,
with 2,803 tests passed, no failures, two ignored tests, and all 17 WASM-safe
crates checked. A sequential proofing run also passes all 211 tests. The merge
includes equivalent match-guard/boolean rewrites for strict Clippy checks.
Independent review found no remaining merge-specific issues after removing a
partial Turkish test dictionary that could interfere with upstream tests.
Mac and Windows runtime checks have not been repeated for this revision.

## Upstream sync (2026-10-22)

Merged upstream `9130181a` (0.10.0, 843 commits after the previous base).
Finnish remains embedded. Mixed Finnish paragraphs preserve British and Canadian
spelling variants and the upstream list-item capitalization rule. The adapter
reports Indonesian dictionary availability and delegates non-Finnish grammar to
the new upstream context-aware dispatcher, removing the older per-language
wrappers. Table-language inheritance also follows upstream's skipped grid columns.
Regression tests cover regional spelling, availability, list context and table
proofing. The new text-box pagination fixture uses explicit filler spacing so its
page counts do not depend on host font metrics; its original assertions remain.
The native workspace suite passed 3,926 tests (four upstream tests ignored).
The corrected fixture also passed independently after review.

## Revision history

| Date | Change | Summary |
| --- | --- | --- |
| 2026-10-14 | minor | Synced upstream 0.7.0; retained Finnish and integrated Italian/Croatian routing and upstream spelling optimization |
| 2026-10-13 | minor | Owner confirmed the Mac version works; recorded overall Mac acceptance separately from automated checks and unreported manual details; Windows testing remains pending |
| 2026-10-13 | minor | Recorded native Apple Silicon build, ad-hoc package, offline app automation and visual results; personal acceptance and Windows testing pending |
