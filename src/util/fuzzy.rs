/// Result of a fuzzy match, including the score and matched indices.
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    /// Lower is better. `None` means no match.
    pub score: f64,
    /// Byte indices in the haystack where pattern characters matched.
    pub indices: Vec<usize>,
}

/// Fuzzy-match `pattern` against `haystack`.
///
/// Returns `Some(FuzzyMatch)` if all characters in the pattern are found
/// (in order) in the haystack, with a score reflecting match quality.
/// Returns `None` if the pattern doesn't match.
///
/// # Scoring
///
/// - **Consecutive bonus**: -5 per consecutive matched character
/// - **Word boundary bonus**: -10 when a match starts at a word boundary
/// - **Gap penalty**: +2 per gap position between matches
/// - **Late match penalty**: +0.1 per column offset of the first match
///
/// Lower scores are better matches.
///
/// # Example
///
/// ```
/// use louie::util::fuzzy::fuzzy_match;
///
/// let m = fuzzy_match("abc", "a_b_c").unwrap();
/// assert_eq!(m.indices, vec![0, 2, 4]);
///
/// // Exact prefix matches score better
/// let prefix = fuzzy_match("he", "hello").unwrap();
/// let mid = fuzzy_match("he", "the help").unwrap();
/// assert!(prefix.score < mid.score);
/// ```
pub fn fuzzy_match(pattern: &str, haystack: &str) -> Option<FuzzyMatch> {
    if pattern.is_empty() {
        return Some(FuzzyMatch {
            score: 0.0,
            indices: vec![],
        });
    }

    let pattern_lower: Vec<char> = pattern.chars().flat_map(|c| c.to_lowercase()).collect();
    let haystack_chars: Vec<char> = haystack.chars().collect();
    let haystack_lower: Vec<char> = haystack.chars().flat_map(|c| c.to_lowercase()).collect();

    // Quick check: are all pattern chars present?
    {
        let mut hi = 0;
        for &pc in &pattern_lower {
            let found = haystack_lower[hi..].iter().position(|&hc| hc == pc);
            match found {
                Some(pos) => hi += pos + 1,
                None => return None,
            }
        }
    }

    // Find best match using a greedy approach with lookahead
    let indices = best_match_indices(&pattern_lower, &haystack_lower)?;

    // Score the match
    let mut score = 0.0;

    // Late match penalty
    if let Some(&first) = indices.first() {
        score += first as f64 * 0.1;
    }

    // Per-character scoring
    for (i, &idx) in indices.iter().enumerate() {
        if i > 0 {
            let prev_idx = indices[i - 1];
            let gap = idx - prev_idx - 1;
            if gap == 0 {
                // Consecutive match bonus
                score -= 5.0;
            } else {
                // Gap penalty
                score += gap as f64 * 2.0;
            }
        }

        // Word boundary bonus
        if idx == 0
            || is_word_boundary(
                haystack_chars.get(idx.wrapping_sub(1)).copied(),
                haystack_chars[idx],
            )
        {
            score -= 10.0;
        }
    }

    Some(FuzzyMatch { score, indices })
}

/// Find the best matching indices using a greedy approach that prefers
/// consecutive runs and word boundaries.
fn best_match_indices(pattern: &[char], haystack: &[char]) -> Option<Vec<usize>> {
    let n = pattern.len();
    let m = haystack.len();

    if n == 0 {
        return Some(vec![]);
    }
    if n > m {
        return None;
    }

    // Simple greedy: for each pattern char, try to find the best position
    // Prefer: (1) consecutive with previous, (2) word boundary, (3) first occurrence
    let mut indices = Vec::with_capacity(n);
    let mut search_start = 0;

    for (pi, &pc) in pattern.iter().enumerate() {
        let mut best_pos = None;
        let mut best_priority = u32::MAX;

        for hi in search_start..m {
            if haystack[hi] == pc {
                let priority = if pi > 0 && hi == indices[pi - 1] + 1 {
                    0 // Consecutive — best
                } else if hi == 0 || is_word_boundary(Some(haystack[hi - 1]), haystack[hi]) {
                    1 // Word boundary — good
                } else {
                    2 // Any other position
                };

                if priority < best_priority {
                    best_priority = priority;
                    best_pos = Some(hi);
                    if priority == 0 {
                        break; // Can't do better than consecutive
                    }
                }

                // Don't search too far ahead
                if hi > search_start + 50 {
                    break;
                }
            }
        }

        let pos = best_pos?;
        indices.push(pos);
        search_start = pos + 1;
    }

    Some(indices)
}

/// Check if a character is at a word boundary.
fn is_word_boundary(prev: Option<char>, current: char) -> bool {
    match prev {
        None => true,
        Some(p) => {
            // Transition from non-alphanumeric to alphanumeric
            (!p.is_alphanumeric() && current.is_alphanumeric())
            // Transition from lowercase to uppercase
            || (p.is_lowercase() && current.is_uppercase())
            // Transition from non-letter to letter
            || (p.is_ascii_digit() && current.is_alphabetic())
        }
    }
}

/// Score and rank a list of candidates against a pattern.
///
/// Returns candidates sorted by score (best first), filtered to only matching items.
pub fn fuzzy_rank<'a>(pattern: &str, candidates: &'a [&str]) -> Vec<(&'a str, FuzzyMatch)> {
    let mut results: Vec<(&str, FuzzyMatch)> = candidates
        .iter()
        .filter_map(|&c| fuzzy_match(pattern, c).map(|m| (c, m)))
        .collect();
    results.sort_by(|a, b| {
        a.1.score
            .partial_cmp(&b.1.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_pattern_matches_everything() {
        let m = fuzzy_match("", "anything").unwrap();
        assert_eq!(m.score, 0.0);
        assert!(m.indices.is_empty());
    }

    #[test]
    fn exact_match() {
        let m = fuzzy_match("hello", "hello").unwrap();
        assert_eq!(m.indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn prefix_match_scores_well() {
        let prefix = fuzzy_match("he", "hello").unwrap();
        let midmatch = fuzzy_match("he", "the").unwrap();
        assert!(prefix.score < midmatch.score);
    }

    #[test]
    fn no_match_returns_none() {
        assert!(fuzzy_match("xyz", "hello").is_none());
    }

    #[test]
    fn case_insensitive() {
        let m = fuzzy_match("abc", "AbC").unwrap();
        assert_eq!(m.indices, vec![0, 1, 2]);
    }

    #[test]
    fn word_boundary_preferred() {
        // "fb" should prefer matching at word boundaries in "foo_bar"
        let m = fuzzy_match("fb", "foo_bar").unwrap();
        assert_eq!(m.indices, vec![0, 4]); // f at 0, b at 4 (word boundary)
    }

    #[test]
    fn fuzzy_rank_sorts_by_score() {
        let candidates = &["hello_world", "help", "he", "abcde"];
        let ranked = fuzzy_rank("he", candidates);
        assert!(ranked.len() >= 2);
        assert!(ranked[0].1.score <= ranked[1].1.score);
    }
}
