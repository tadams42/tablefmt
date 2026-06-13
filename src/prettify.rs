pub struct PrettifyMeta {
    pub pre_comment_spaces:  usize,
    pub comment:             Option<String>,
    pub post_comment_spaces: usize,
}

pub fn preprocess(input: &str) -> (Vec<String>, PrettifyMeta) {
    const COMMENT_PREFIXES: &[&str] = &["///", "//", "#", "*"];

    let mut max_pre: usize = 0;
    let mut first_comment: Option<String> = None;
    let mut max_post: usize = 0;
    let mut bare_lines: Vec<String> = Vec::new();

    for line in input.lines() {
        let (pre_spaces, after_ws) = measure_leading_ws(line);

        let mut comment_found: Option<&str> = None;
        for &prefix in COMMENT_PREFIXES {
            if after_ws.starts_with(prefix) {
                comment_found = Some(prefix);
                break;
            }
        }

        let bare = if let Some(c) = comment_found {
            let after_comment = &after_ws[c.len()..];
            let (post_spaces, content) = measure_leading_ws(after_comment);
            max_post = max_post.max(post_spaces);
            if first_comment.is_none() {
                first_comment = Some(c.to_string());
            }
            content.to_string()
        } else {
            after_ws.to_string()
        };

        max_pre = max_pre.max(pre_spaces);
        bare_lines.push(bare);
    }

    let meta = PrettifyMeta {
        pre_comment_spaces:  max_pre,
        comment:             first_comment,
        post_comment_spaces: max_post,
    };

    (bare_lines, meta)
}

/// Strips leading whitespace and a single comment prefix from `line`,
/// returning the bare table content. Used by `locate` for per-line detection.
pub fn bare_line(line: &str) -> &str {
    const COMMENT_PREFIXES: &[&str] = &["///", "//", "#", "*"];
    let (_, after_ws) = measure_leading_ws(line);
    for &prefix in COMMENT_PREFIXES {
        if let Some(after_comment) = after_ws.strip_prefix(prefix) {
            let (_, content) = measure_leading_ws(after_comment);
            return content;
        }
    }
    after_ws
}

fn measure_leading_ws(s: &str) -> (usize, &str) {
    let mut spaces = 0usize;
    let mut rest = s;
    loop {
        if rest.starts_with('\t') {
            spaces += 2;
            rest = &rest[1..];
        } else if rest.starts_with(' ') {
            spaces += 1;
            rest = &rest[1..];
        } else {
            break;
        }
    }
    (spaces, rest)
}

pub fn postprocess(rendered: &str, meta: &PrettifyMeta) -> String {
    let ends_with_newline = rendered.ends_with('\n');
    let pre = " ".repeat(meta.pre_comment_spaces);
    let post_suffix = match &meta.comment {
        None => String::new(),
        Some(c) => format!("{c}{}", " ".repeat(meta.post_comment_spaces.max(1))),
    };
    let prefix = format!("{pre}{post_suffix}");

    let result = rendered
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n");

    if ends_with_newline {
        result + "\n"
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta_of(input: &str) -> PrettifyMeta { preprocess(input).1 }

    fn bare_of(input: &str) -> Vec<String> { preprocess(input).0 }

    #[test]
    fn preprocess_no_indent_no_comment() {
        let m = meta_of("| a | b |\n|---|---|\n| 1 | 2 |");
        assert_eq!(m.pre_comment_spaces, 0);
        assert!(m.comment.is_none());
    }

    #[test]
    fn preprocess_leading_spaces_uses_max() {
        let m = meta_of("  | a |\n    | b |");
        assert_eq!(m.pre_comment_spaces, 4);
    }

    #[test]
    fn preprocess_tabs_count_as_two() {
        let m = meta_of("\t| a |");
        assert_eq!(m.pre_comment_spaces, 2);
    }

    #[test]
    fn preprocess_mixed_indent_uses_max() {
        let m = meta_of("  | a |\n    | b |");
        assert_eq!(m.pre_comment_spaces, 4);
    }

    #[test]
    fn preprocess_comment_hash() {
        let m = meta_of("# | row |");
        assert_eq!(m.comment.as_deref(), Some("#"));
        assert!(m.post_comment_spaces >= 1);
    }

    #[test]
    fn preprocess_comment_double_slash() {
        let m = meta_of("// | row |");
        assert_eq!(m.comment.as_deref(), Some("//"));
    }

    #[test]
    fn preprocess_comment_triple_slash_not_double() {
        let m = meta_of("/// | row |");
        assert_eq!(m.comment.as_deref(), Some("///"));
    }

    #[test]
    fn preprocess_comment_star() {
        let m = meta_of("* | row |");
        assert_eq!(m.comment.as_deref(), Some("*"));
    }

    #[test]
    fn preprocess_partial_comment_all_output_commented() {
        let m = meta_of("| a |\n# | b |");
        assert_eq!(m.comment.as_deref(), Some("#"));
    }

    #[test]
    fn preprocess_post_comment_min_one_space_enforced_in_postprocess() {
        let m = meta_of("#| row |");
        assert_eq!(m.post_comment_spaces, 0);
        // postprocess must clamp to 1
        let out = postprocess("x\n", &m);
        assert!(out.starts_with("#"));
        let after_hash = &out[1..];
        assert!(after_hash.starts_with(' '));
    }

    #[test]
    fn preprocess_indent_before_and_after_comment() {
        let m = meta_of("   #  | row |");
        assert_eq!(m.pre_comment_spaces, 3);
        assert_eq!(m.comment.as_deref(), Some("#"));
        assert_eq!(m.post_comment_spaces, 2);
    }

    #[test]
    fn preprocess_bare_lines_strips_indent_and_comment() {
        let lines = bare_of("  # | a | b |");
        assert_eq!(lines[0], "| a | b |");
    }

    #[test]
    fn postprocess_round_trips_prefix() {
        let m = PrettifyMeta {
            pre_comment_spaces:  2,
            comment:             Some("#".to_string()),
            post_comment_spaces: 1,
        };
        let rendered = "| a | b |\n|---|---|\n| 1 | 2 |\n";
        let out = postprocess(rendered, &m);
        for line in out.lines() {
            assert!(line.starts_with("  # "), "line missing prefix: {line:?}");
        }
    }

    #[test]
    fn postprocess_preserves_trailing_newline() {
        let m = PrettifyMeta {
            pre_comment_spaces:  0,
            comment:             None,
            post_comment_spaces: 0,
        };
        let out = postprocess("a\nb\n", &m);
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn postprocess_no_trailing_newline_when_input_lacks_one() {
        let m = PrettifyMeta {
            pre_comment_spaces:  0,
            comment:             None,
            post_comment_spaces: 0,
        };
        let out = postprocess("a\nb", &m);
        assert!(!out.ends_with('\n'));
    }
}
