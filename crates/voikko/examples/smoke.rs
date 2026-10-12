//! Offline proofing smoke test; also runnable as a wasm32-unknown-unknown binary.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut checker = wordcraft_voikko::Voikko::new()?;
    if !checker.spell("talossanikinko")? || checker.spell("koirra")? {
        return Err("Finnish spelling failed".into());
    }
    if !checker.suggest("koirra", 6)?.iter().any(|s| s == "koira") {
        return Err("Finnish suggestions failed".into());
    }
    if checker.hyphenate("kissa")? != [3] {
        return Err("Finnish hyphenation failed".into());
    }
    if !checker.grammar("Äiti on on kotona.")?.iter().any(|e| e.code == 8 && e.start == 6 && e.end == 11) {
        return Err("Finnish grammar offsets failed".into());
    }
    checker.status()?;
    Ok(())
}
