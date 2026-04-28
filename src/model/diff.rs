use std::ops::Range;

use anyhow::{Context, Result};
use git2::{DiffFormat, DiffOptions, Repository};
use tiny_skia::Color;

use crate::config::DiffStyle;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    FileHeader, // 'F' 且 content 以 "diff --git" 开头
    Hunk,       // 'H'
    Added,      // '+'
    Deleted,    // '-'
    Context,    // ' '
    Other,      // 其它 'F' 行（index / --- / +++）
    Separator,  // 人造的文件间分隔行
}

impl LineKind {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Added => "+",
            Self::Deleted => "-",
            Self::Context => " ",
            _ => "",
        }
    }

    pub fn color(self, s: &DiffStyle) -> Color {
        match self {
            Self::Added => s.added_fg,
            Self::Deleted => s.deleted_fg,
            Self::Hunk => s.hunk_fg,
            Self::FileHeader => s.file_fg,
            _ => s.default_fg,
        }
    }
}

pub struct DiffLine {
    pub kind: LineKind,
    pub content: String,
    pub inline_ranges: Vec<Range<usize>>,
}

impl DiffLine {
    pub fn from_repo(
        repo: &Repository,
        pathspecs: &[String],
        whitespace: bool,
    ) -> Result<Vec<Self>> {
        let mut opts = DiffOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .show_untracked_content(true)
            .ignore_whitespace(!whitespace);
        for ps in pathspecs {
            opts.pathspec(ps);
        }

        let diff = repo
            .diff_index_to_workdir(None, Some(&mut opts))
            .context("failed to get diff")?;

        let mut lines: Vec<Self> = Vec::new();
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            let kind = classify_origin(line.origin(), line.content());
            let content = String::from_utf8_lossy(line.content())
                .trim_end_matches('\n')
                .to_owned();

            // 非首个文件前插入分隔行，视觉区分多文件 diff
            if kind == LineKind::FileHeader && !lines.is_empty() {
                lines.push(Self {
                    kind: LineKind::Separator,
                    content: String::new(),
                    inline_ranges: Vec::new(),
                });
            }
            lines.push(Self {
                kind,
                content,
                inline_ranges: Vec::new(),
            });
            true
        })
        .context("failed to collect diff")?;

        annotate_inline_diffs(&mut lines);
        Ok(lines)
    }
}

fn classify_origin(origin: char, content: &[u8]) -> LineKind {
    match origin {
        '+' => LineKind::Added,
        '-' => LineKind::Deleted,
        ' ' => LineKind::Context,
        'H' => LineKind::Hunk,
        'F' if content.starts_with(b"diff --git") => LineKind::FileHeader,
        _ => LineKind::Other,
    }
}

// 连续 `-` 段与紧随的连续 `+` 段按索引一一配对；长度不等时尾部多余行不做高亮
fn annotate_inline_diffs(lines: &mut [DiffLine]) {
    let n = lines.len();
    let mut i = 0;

    while i < n {
        if lines[i].kind != LineKind::Deleted {
            i += 1;
            continue;
        }

        let del_start = i;
        while i < n && lines[i].kind == LineKind::Deleted {
            i += 1;
        }
        let del_end = i;

        let add_start = i;
        while i < n && lines[i].kind == LineKind::Added {
            i += 1;
        }
        let add_end = i;

        let pair_count = (del_end - del_start).min(add_end - add_start);
        for k in 0..pair_count {
            let di = del_start + k;
            let ai = add_start + k;
            let (del_ranges, add_ranges) =
                inline_diff_ranges(&lines[di].content, &lines[ai].content);
            lines[di].inline_ranges = del_ranges;
            lines[ai].inline_ranges = add_ranges;
        }
    }
}

// 低于此相似度认为两行差异过大，不做 inline 高亮（避免满屏红绿块）
const SIMILARITY_THRESHOLD: f64 = 0.3;

// 字符数超过此值的行跳过 LCS，避免 O(m·n) 退化
const MAX_LINE_LENGTH: usize = 2000;

fn inline_diff_ranges(old: &str, new: &str) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    let old = old.trim_end_matches('\n');
    let new = new.trim_end_matches('\n');

    if old.chars().count() > MAX_LINE_LENGTH || new.chars().count() > MAX_LINE_LENGTH {
        return (Vec::new(), Vec::new());
    }

    let old_tokens = tokenize(old);
    let new_tokens = tokenize(new);

    if old_tokens.is_empty() || new_tokens.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let (dp, stride) = lcs_table(&old_tokens, &new_tokens);
    let (old_matched, new_matched) = lcs_match_mask(&old_tokens, &new_tokens, &dp, stride);

    let (old_non_ws, old_matched_ws) = count_non_ws(&old_tokens, &old_matched);
    let (new_non_ws, new_matched_ws) = count_non_ws(&new_tokens, &new_matched);
    let denom = old_non_ws + new_non_ws;

    if denom == 0 {
        return (Vec::new(), Vec::new());
    }

    let similarity = (old_matched_ws + new_matched_ws) as f64 / denom as f64;
    if similarity < SIMILARITY_THRESHOLD {
        return (Vec::new(), Vec::new());
    }

    let del_ranges = merged_ranges(&old_tokens, &old_matched);
    let add_ranges = merged_ranges(&new_tokens, &new_matched);

    (del_ranges, add_ranges)
}

// DP 表扁平化为一维 Vec，用 i*stride+j 索引，避免 Vec<Vec<_>> 两层堆分配
fn lcs_table(a: &[Token<'_>], b: &[Token<'_>]) -> (Vec<usize>, usize) {
    let m = a.len();
    let n = b.len();
    let stride = n + 1;
    let mut dp = vec![0usize; (m + 1) * stride];

    for i in 1..=m {
        for j in 1..=n {
            dp[i * stride + j] = if a[i - 1].text == b[j - 1].text {
                dp[(i - 1) * stride + (j - 1)] + 1
            } else {
                dp[(i - 1) * stride + j].max(dp[i * stride + (j - 1)])
            };
        }
    }

    (dp, stride)
}

fn lcs_match_mask(
    a: &[Token<'_>],
    b: &[Token<'_>],
    dp: &[usize],
    stride: usize,
) -> (Vec<bool>, Vec<bool>) {
    let mut a_matched = vec![false; a.len()];
    let mut b_matched = vec![false; b.len()];

    let mut i = a.len();
    let mut j = b.len();

    while i > 0 && j > 0 {
        if a[i - 1].text == b[j - 1].text
            && dp[i * stride + j] == dp[(i - 1) * stride + (j - 1)] + 1
        {
            a_matched[i - 1] = true;
            b_matched[j - 1] = true;
            i -= 1;
            j -= 1;
        } else if dp[(i - 1) * stride + j] >= dp[i * stride + (j - 1)] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    (a_matched, b_matched)
}

fn count_non_ws(tokens: &[Token<'_>], matched: &[bool]) -> (usize, usize) {
    let mut total = 0usize;
    let mut matched_count = 0usize;
    for (t, m) in tokens.iter().zip(matched) {
        if !t.is_whitespace {
            total += 1;
            if *m {
                matched_count += 1;
            }
        }
    }
    (total, matched_count)
}

// 相邻未匹配 token 合并成连续 range；夹在中间的未匹配空白不单独起新 range，
// 但会被纳入已开启的 range 里，视觉上保持段落连续
fn merged_ranges(tokens: &[Token<'_>], matched: &[bool]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if matched[i] {
            i += 1;
            continue;
        }
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

        ranges.push(tokens[start].start..tokens[end].end);

        i = j;
    }

    ranges
}

// start/end 为字节偏移（非字符数），配合 &str 切片零拷贝使用；
// char_indices 保证 token 边界始终在 UTF-8 字符边界上
struct Token<'a> {
    start: usize,
    end: usize,
    text: &'a str,
    is_whitespace: bool,
}

fn tokenize(s: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    let mut chars = s.char_indices().peekable();

    while let Some(&(start, c)) = chars.peek() {
        if is_identifier_char(c) {
            let end = consume_run(&mut chars, s.len(), is_identifier_char);
            out.push(Token {
                start,
                end,
                text: &s[start..end],
                is_whitespace: false,
            });
        } else if is_plain_whitespace(c) {
            let end = consume_run(&mut chars, s.len(), is_plain_whitespace);
            out.push(Token {
                start,
                end,
                text: &s[start..end],
                is_whitespace: true,
            });
        } else {
            // 标点单字符成 token
            let (_, ch) = chars.next().expect("peeked");
            let end = start + ch.len_utf8();
            out.push(Token {
                start,
                end,
                text: &s[start..end],
                is_whitespace: false,
            });
        }
    }

    out
}

fn consume_run<I>(
    chars: &mut std::iter::Peekable<I>,
    source_len: usize,
    predicate: fn(char) -> bool,
) -> usize
where
    I: Iterator<Item = (usize, char)>,
{
    let mut end = source_len;
    while let Some(&(i, c)) = chars.peek() {
        if predicate(c) {
            chars.next();
        } else {
            end = i;
            break;
        }
    }
    end
}

fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_plain_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}
