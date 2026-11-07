# Embedded Finnish dictionary

These are unmodified dictionary files from yongsk0066/corevoikko, revision
`beba2cd7c0b3839b63f54c18948c4e2add318bc1`, `libvoikko/js/dict/`.
They are embedded by `wordcraft-voikko` on desktop and WebAssembly. No runtime
installation, filesystem search or network request is needed.

| File | SHA-256 |
| --- | --- |
| `mor.vfst` | `95b816605ca670e813cff20c9bc9767b4a52928ab662d6336373cd387a51446a` |
| `autocorr.vfst` | `e06326e84b72e240d1b7047f0e6a899b5e98e8a0a3bb05aa803a66dbf7377ce1` |

Dictionary authors: the Voikko project contributors; see `CONTRIBUTORS.txt`.
Dictionary license: GPL-2.0-or-later, separate from WordCraft's own source license;
see `COPYING-dictionary.txt`. The Rust dependency declares
`MPL-1.1 OR GPL-2.0-or-later OR LGPL-2.1-or-later`; this integration selects MPL-1.1.
See `LICENSE-libvoikko.txt`. No library implementation source is copied here.

Corresponding source, notices and build instructions:
https://github.com/yongsk0066/corevoikko/tree/beba2cd7c0b3839b63f54c18948c4e2add318bc1

Source archive (library, dictionary sources, data and build scripts):
https://github.com/yongsk0066/corevoikko/archive/beba2cd7c0b3839b63f54c18948c4e2add318bc1.tar.gz

Archive SHA-256: `c376240b4457b6e05d259b4a6f1342e58404f194ebcc4037a2f2bad3cbedcedb`.
Before publishing binaries, make the corresponding source archive available
alongside them and review the library and dictionary licensing with maintainers.
This file is provenance documentation, not a change to upstream license terms.

To update: choose a reviewed upstream revision, update both Cargo git revisions,
replace both dictionaries from that revision, update hashes/attribution/notices,
and run the complete Finnish regression suite before packaging.
