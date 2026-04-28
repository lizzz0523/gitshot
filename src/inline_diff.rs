/// Character-level inline diff using LCS (Longest Common Subsequence).
///
/// Given a pair of `-` / `+` lines, computes which character ranges
/// differ so they can be rendered with a distinct highlight.

/// A half-open range of character indices within a line's content
/// (after stripping the `+`/`-` prefix).
#[derive(Debug, Clone)]
pub struct InlineRange {
    pub start: usize,
    pub end: usize,
}

/// Compute inline highlight ranges for a pair of lines.
///
/// Returns `(deleted_ranges, added_ranges)` where each is a list of
/// non-overlapping, sorted `InlineRange` referring to character positions
/// within the line content (excluding the `+`/`-` prefix).
pub fn inline_diff(old: &str, new: &str) -> (Vec<InlineRange>, Vec<InlineRange>) {
    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();

    let lcs = lcs_table(&old_chars, &new_chars);
    let (old_diff, new_diff) = diff_from_lcs(&old_chars, &new_chars, &lcs);

    let deleted_ranges = consolidate_ranges(&old_diff);
    let added_ranges = consolidate_ranges(&new_diff);

    (deleted_ranges, added_ranges)
}

/// Build the LCS DP table.
fn lcs_table(a: &[char], b: &[char]) -> Vec<Vec<usize>> {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    dp
}

/// Walk the LCS table backwards to determine which characters are different.
///
/// Returns two boolean vectors: `old_diff[i]` is true if `old[i]` is NOT
/// part of the LCS (i.e. it's a deleted character), and similarly for `new_diff`.
fn diff_from_lcs(
    old: &[char],
    new: &[char],
    dp: &[Vec<usize>],
) -> (Vec<bool>, Vec<bool>) {
    let mut old_diff = vec![true; old.len()];
    let mut new_diff = vec![true; new.len()];

    let mut i = old.len();
    let mut j = new.len();

    while i > 0 && j > 0 {
        if old[i - 1] == new[j - 1] {
            old_diff[i - 1] = false;
            new_diff[j - 1] = false;
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    (old_diff, new_diff)
}

/// Merge adjacent true entries in a diff mask into contiguous InlineRanges.
fn consolidate_ranges(diff: &[bool]) -> Vec<InlineRange> {
    let mut ranges = Vec::new();
    let mut i = 0;

    while i < diff.len() {
        if diff[i] {
            let start = i;
            while i < diff.len() && diff[i] {
                i += 1;
            }
            ranges.push(InlineRange { start, end: i });
        } else {
            i += 1;
        }
    }

    ranges
}

/// Pair consecutive `-` and `+` lines and annotate them with inline ranges.
///
/// `lines` is a mutable slice of `(origin, content, inline_ranges)`.
/// The `origin` field is `'+'`, `'-'`, or other. The `content` field
/// is the raw line content (without the `+`/`-` prefix). The `inline_ranges`
/// field will be populated for paired lines.
pub fn annotate_inline_diffs(lines: &mut [(char, String, Vec<InlineRange>)]) {
    let n = lines.len();
    let mut i = 0;

    while i < n {
        if lines[i].0 != '-' {
            i += 1;
            continue;
        }

        // Collect consecutive '-' lines
        let del_start = i;
        while i < n && lines[i].0 == '-' {
            i += 1;
        }
        let del_end = i;

        // Collect consecutive '+' lines
        let add_start = i;
        while i < n && lines[i].0 == '+' {
            i += 1;
        }
        let add_end = i;

        // Pair them up 1-to-1
        let pair_count = (del_end - del_start).min(add_end - add_start);
        for k in 0..pair_count {
            let di = del_start + k;
            let ai = add_start + k;
            let (del_ranges, add_ranges) = inline_diff(&lines[di].1, &lines[ai].1);
            lines[di].2 = del_ranges;
            lines[ai].2 = add_ranges;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_diff_simple() {
        let (del, add) = inline_diff("hello_world", "hello_rust");
        assert!(!del.is_empty());
        assert!(!add.is_empty());
        // Common prefix "hello_" (6 chars) must not be in any range
        for r in &del {
            assert!(r.start >= 6);
        }
        for r in &add {
            assert!(r.start >= 6);
        }
    }

    #[test]
    fn test_inline_diff_identical() {
        let (del, add) = inline_diff("same", "same");
        assert!(del.is_empty());
        assert!(add.is_empty());
    }

    #[test]
    fn test_inline_diff_completely_different() {
        let (del, add) = inline_diff("abc", "xyz");
        assert_eq!(del.len(), 1);
        assert_eq!(del[0].start, 0);
        assert_eq!(del[0].end, 3);
        assert_eq!(add.len(), 1);
        assert_eq!(add[0].start, 0);
        assert_eq!(add[0].end, 3);
    }
}
