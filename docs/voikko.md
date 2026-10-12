# Finnish proofing with Voikko

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
formatting. English, Polish, Portuguese and Bulgarian keep their existing tools.

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
mean those remote machines have already passed. User testing on Windows and
Apple Silicon remains necessary.

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
commit. Windows x86_64 and Apple Silicon execution remains pending.
