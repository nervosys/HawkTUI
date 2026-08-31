//! Inline grapheme storage for terminal cells.
//!
//! A terminal cell holds one grapheme cluster. Storing that as a heap-backed
//! string costs an allocation per cell and makes [`Cell`](super::cell::Cell)
//! non-`Copy`, which in turn makes buffer allocation, reset, and diffing pay
//! for clones and destructors on every one of the ~10,000 cells in a frame.
//!
//! [`Symbol`] stores clusters of up to 7 UTF-8 bytes inline in 8 bytes total,
//! which covers every single scalar value (max 4 bytes) plus most combining
//! sequences. Longer clusters — ZWJ emoji, flags, skin-tone modifiers — are
//! interned in a process-wide table and referenced by index, so `Symbol` stays
//! `Copy` and 8 bytes wide without ever losing content.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

/// Maximum cluster length stored inline.
const INLINE_CAP: usize = 7;

/// `len` sentinel marking an interned (out-of-line) cluster.
const INTERNED: u8 = 0xFF;

/// A grapheme cluster stored in 8 bytes.
///
/// Clusters of up to 7 UTF-8 bytes live inline; longer ones are interned.
/// `Symbol` is `Copy`, never allocates for the common case, and compares
/// bytewise.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol {
    /// Number of inline bytes, or [`INTERNED`].
    len: u8,
    /// Inline UTF-8 bytes, or a little-endian intern index in `bytes[0..4]`.
    bytes: [u8; INLINE_CAP],
}

impl Symbol {
    /// The empty symbol, used for the trailing half of a wide grapheme.
    pub const EMPTY: Self = Self {
        len: 0,
        bytes: [0; INLINE_CAP],
    };

    /// A single space — the contents of a reset cell.
    pub const SPACE: Self = Self {
        len: 1,
        bytes: [b' ', 0, 0, 0, 0, 0, 0],
    };

    /// Store a grapheme cluster, interning it if it exceeds the inline capacity.
    pub fn new(s: &str) -> Self {
        let b = s.as_bytes();
        if b.len() <= INLINE_CAP {
            let mut bytes = [0u8; INLINE_CAP];
            bytes[..b.len()].copy_from_slice(b);
            Self {
                len: b.len() as u8,
                bytes,
            }
        } else {
            let idx = intern(s);
            let mut bytes = [0u8; INLINE_CAP];
            bytes[..4].copy_from_slice(&idx.to_le_bytes());
            Self {
                len: INTERNED,
                bytes,
            }
        }
    }

    /// Store a single ASCII byte. Branch-free and always inline.
    ///
    /// The caller must pass a byte below `0x80`; anything else would not be
    /// valid UTF-8 on its own and is replaced with a space.
    #[inline]
    pub const fn from_ascii(byte: u8) -> Self {
        let byte = if byte < 0x80 { byte } else { b' ' };
        Self {
            len: 1,
            bytes: [byte, 0, 0, 0, 0, 0, 0],
        }
    }

    /// Store a single `char` — always inline, never allocates.
    pub fn from_char(ch: char) -> Self {
        let mut buf = [0u8; 4];
        Self::new(ch.encode_utf8(&mut buf))
    }

    /// The cluster as a string slice.
    pub fn as_str(&self) -> &str {
        if self.len == INTERNED {
            let idx =
                u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]]);
            resolve(idx)
        } else {
            // Safety-free path: the bytes were copied from a `&str`, so the
            // prefix of `len` bytes is always valid UTF-8.
            std::str::from_utf8(&self.bytes[..self.len as usize]).unwrap_or("")
        }
    }

    /// Whether this symbol holds no content (a wide-grapheme continuation cell).
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Length of the cluster in bytes.
    pub fn len(&self) -> usize {
        if self.len == INTERNED {
            self.as_str().len()
        } else {
            self.len as usize
        }
    }
}

impl Default for Symbol {
    fn default() -> Self {
        Self::SPACE
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<char> for Symbol {
    fn from(ch: char) -> Self {
        Self::from_char(ch)
    }
}

impl PartialEq<str> for Symbol {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Symbol {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// ─────────────────────────────── interning ────────────────────────────────

type Interner = Mutex<(HashMap<&'static str, u32>, Vec<&'static str>)>;

fn interner() -> &'static Interner {
    static INTERNER: OnceLock<Interner> = OnceLock::new();
    INTERNER.get_or_init(|| Mutex::new((HashMap::new(), Vec::new())))
}

/// Intern a long cluster, returning its stable index.
///
/// Interned clusters live for the life of the process. The table only ever
/// receives grapheme clusters longer than 7 bytes, so it stays small in
/// practice (one entry per distinct exotic emoji actually rendered).
fn intern(s: &str) -> u32 {
    let mut guard = interner().lock().unwrap_or_else(|e| e.into_inner());
    let (map, list) = &mut *guard;
    if let Some(&idx) = map.get(s) {
        return idx;
    }
    let leaked: &'static str = Box::leak(s.to_string().into_boxed_str());
    let idx = list.len() as u32;
    list.push(leaked);
    map.insert(leaked, idx);
    idx
}

/// Look up an interned cluster by index.
fn resolve(idx: u32) -> &'static str {
    let guard = interner().lock().unwrap_or_else(|e| e.into_inner());
    guard.1.get(idx as usize).copied().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_is_eight_bytes_and_copy() {
        assert_eq!(std::mem::size_of::<Symbol>(), 8);
        fn assert_copy<T: Copy>() {}
        assert_copy::<Symbol>();
    }

    #[test]
    fn inline_round_trip() {
        for s in ["a", " ", "é", "→", "🦀", "e\u{0301}", ""] {
            assert_eq!(Symbol::new(s).as_str(), s, "round trip failed for {s:?}");
        }
    }

    #[test]
    fn long_cluster_interns_and_round_trips() {
        // Family emoji: far longer than the inline capacity.
        let family = "👨‍👩‍👧‍👦";
        assert!(family.len() > INLINE_CAP);
        let sym = Symbol::new(family);
        assert_eq!(sym.as_str(), family);
        assert_eq!(sym.len(), family.len());
        // Interning is stable: equal clusters compare equal.
        assert_eq!(sym, Symbol::new(family));
    }

    #[test]
    fn skin_tone_emoji_round_trips() {
        let thumbs = "👍🏽"; // 8 bytes — one past the inline capacity
        assert_eq!(Symbol::new(thumbs).as_str(), thumbs);
    }

    #[test]
    fn empty_is_distinct_from_space() {
        assert!(Symbol::EMPTY.is_empty());
        assert!(!Symbol::SPACE.is_empty());
        assert_ne!(Symbol::EMPTY, Symbol::SPACE);
        assert_eq!(Symbol::SPACE.as_str(), " ");
    }

    #[test]
    fn char_constructor_matches_str() {
        assert_eq!(Symbol::from_char('x'), Symbol::new("x"));
        assert_eq!(Symbol::from_char('🦀'), Symbol::new("🦀"));
    }
}
