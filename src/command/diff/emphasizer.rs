use std::ops::Range;

use super::tokenizer::{self, Token};

/// Similarity threshold: if two lines are less similar than this, skip inline highlighting.
const SIMILARITY_THRESHOLD: f64 = 0.3;

/// Lines longer than this many characters are skipped to avoid O(m·n) LCS degradation.
const MAX_LINE_LENGTH: usize = 2000;

/// Compute inline highlight ranges for a pair of lines using token-level LCS.
///
/// Returns `(del_ranges, add_ranges)` — character-index ranges within each line
/// that should be emphasized as the changed portions.
pub fn emphasis(old: &str, new: &str) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    // Strip trailing newline — git2 includes it, but BuddyViewer's split-based parsing does not.
    let old = old.trim_end_matches('\n');
    let new = new.trim_end_matches('\n');

    // Length protection (character count, matching BuddyViewer's String.count)
    if old.chars().count() > MAX_LINE_LENGTH || new.chars().count() > MAX_LINE_LENGTH {
        return (Vec::new(), Vec::new());
    }

    let old_tokens = tokenizer::tokens(old);
    let new_tokens = tokenizer::tokens(new);

    if old_tokens.is_empty() || new_tokens.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let dp = lcs_table(&old_tokens, &new_tokens);
    let (old_matched, new_matched) = lcs_match_mask(&old_tokens, &new_tokens, &dp);

    // Compute similarity: matched_non_ws / (old_non_ws + new_non_ws)
    let old_non_ws = old_tokens.iter().filter(|t| !t.is_whitespace).count();
    let new_non_ws = new_tokens.iter().filter(|t| !t.is_whitespace).count();
    let denom = old_non_ws + new_non_ws;

    if denom == 0 {
        return (Vec::new(), Vec::new());
    }

    let matched_non_ws = old_matched
        .iter()
        .zip(old_tokens.iter())
        .filter(|(m, t)| **m && !t.is_whitespace)
        .count()
        + new_matched
            .iter()
            .zip(new_tokens.iter())
            .filter(|(m, t)| **m && !t.is_whitespace)
            .count();

    let similarity = matched_non_ws as f64 / denom as f64;
    if similarity < SIMILARITY_THRESHOLD {
        return (Vec::new(), Vec::new());
    }

    let del_ranges = merged_ranges(&old_tokens, &old_matched);
    let add_ranges = merged_ranges(&new_tokens, &new_matched);

    (del_ranges, add_ranges)
}

/// Build the LCS DP table over token text.
fn lcs_table(a: &[Token], b: &[Token]) -> Vec<Vec<usize>> {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1].text == b[j - 1].text {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    dp
}

/// Walk the LCS table backwards to determine which tokens are matched (in LCS).
fn lcs_match_mask(a: &[Token], b: &[Token], dp: &[Vec<usize>]) -> (Vec<bool>, Vec<bool>) {
    let mut a_matched = vec![false; a.len()];
    let mut b_matched = vec![false; b.len()];

    let mut i = a.len();
    let mut j = b.len();

    while i > 0 && j > 0 {
        if a[i - 1].text == b[j - 1].text && dp[i][j] == dp[i - 1][j - 1] + 1 {
            a_matched[i - 1] = true;
            b_matched[j - 1] = true;
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    (a_matched, b_matched)
}

/// Merge unmatched non-whitespace tokens into contiguous Ranges.
/// Unmatched whitespace between unmatched non-whitespace tokens keeps ranges visually continuous.
fn merged_ranges(tokens: &[Token], matched: &[bool]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        // Skip matched tokens
        if matched[i] {
            i += 1;
            continue;
        }
        // Unmatched but pure whitespace: don't start a range here
        if tokens[i].is_whitespace {
            i += 1;
            continue;
        }

        let start = i;
        let mut end = i;
        let mut j = i + 1;
        while j < tokens.len() {
            if matched[j] {
                break;
            }
            if !tokens[j].is_whitespace {
                end = j;
            }
            j += 1;
        }

        // [start..=end] contains at least one unmatched non-whitespace token
        ranges.push(tokens[start].start..tokens[end].end);

        i = j;
    }

    ranges
}
