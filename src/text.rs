// Ported from MahaBodi (github.com/mahabodi/mahabodi, crates/mahabodi-core/src/text.rs, MIT).
//! Text normalisation shared by ingestion, the index and search.
//!
//! Everything here is deterministic and allocation-light: identifiers such as
//! `Validate_Token` or `sessionUUID` are split into their word parts so a query
//! for "token" or "session" reaches them.

use std::collections::HashSet;

const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "been", "but", "by", "can", "could", "did", "do",
    "does", "for", "from", "had", "has", "have", "he", "her", "his", "how", "i", "if", "in", "into",
    "is", "it", "its", "me", "my", "no", "not", "of", "on", "or", "our", "she", "so", "such",
    "than", "that", "the", "their", "them", "then", "there", "these", "they", "this", "those",
    "to", "too", "us", "was", "we", "were", "what", "when", "where", "which", "while", "who",
    "whom", "why", "will", "with", "would", "you", "your", "all", "any", "each", "also", "just",
    "only", "own", "same", "very", "s", "t", "about", "after", "before", "over", "under", "up",
    "down", "out", "off", "again", "more", "most", "other", "some", "should", "must", "may",
];

pub fn is_stopword(w: &str) -> bool {
    STOPWORDS.contains(&w)
}

/// Scripts written without spaces between words: Han, Hiragana, Katakana, Thai.
/// There is no segmenter here, so runs of these are indexed as character bigrams
/// (the standard CJK IR fallback) instead of one term per clause.
pub fn is_unspaced_script(c: char) -> bool {
    matches!(c as u32,
        0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x0E00..=0x0E7F | 0x20000..=0x2FA1F)
}

/// Splits `Validate_Token`, `sessionUUID`, `auth-module v2` into lowercase word parts.
/// Unspaced-script runs become overlapping character bigrams (a lone char stays a unigram).
pub fn split_words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut run: Vec<char> = Vec::new();
    let flush_run = |run: &mut Vec<char>, out: &mut Vec<String>| {
        match run.len() {
            0 => {}
            1 => out.push(run[0].to_string()),
            _ => out.extend(run.windows(2).map(|w| w.iter().collect::<String>())),
        }
        run.clear();
    };
    for (i, &c) in chars.iter().enumerate() {
        if is_unspaced_script(c) {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur).to_lowercase());
            }
            run.push(c);
            continue;
        }
        flush_run(&mut run, &mut out);
        if c.is_alphanumeric() {
            // camelCase boundary: lower->Upper, or UPPER->Upper+lower (UUIDToken -> UUID, Token)
            let boundary = !cur.is_empty()
                && c.is_uppercase()
                && (chars[i - 1].is_lowercase()
                    || (chars[i - 1].is_uppercase() && chars.get(i + 1).is_some_and(|n| n.is_lowercase())));
            let digit_boundary = !cur.is_empty() && (c.is_ascii_digit() != chars[i - 1].is_ascii_digit());
            if boundary || digit_boundary {
                out.push(std::mem::take(&mut cur).to_lowercase());
            }
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur).to_lowercase());
        }
    }
    flush_run(&mut run, &mut out);
    if !cur.is_empty() {
        out.push(cur.to_lowercase());
    }
    out
}

/// Content terms: split words minus stopwords and 1-char noise (digits kept).
pub fn terms(s: &str) -> Vec<String> {
    split_words(s)
        .into_iter()
        .filter(|w| {
            !is_stopword(w)
                && (w.chars().count() > 1 || w.chars().all(|c| c.is_ascii_digit() || is_unspaced_script(c)))
        })
        .collect()
}

/// Light suffix-stripping stemmer. Only has to make inflections of one word meet
/// ("rules"/"rule", "validating"/"validate", "policies"/"policy") without merging
/// distinct words ("apply" vs "app", "speed", "family", "news"). Every strip keeps
/// a stem of at least 4 characters; "-ly" is never stripped.
pub fn stem(w: &str) -> String {
    let w = w.to_lowercase();
    if w.len() <= 3 || !w.is_ascii() || STEM_EXCEPTIONS.contains(&w.as_str()) {
        return w;
    }
    let strip = |suf: &str, rep: &str, min_stem: usize| -> Option<String> {
        let base = w.strip_suffix(suf)?;
        (base.len() + rep.len() >= min_stem).then(|| format!("{base}{rep}"))
    };
    let sibilant_es = w.ends_with("es")
        && ["s", "x", "z", "ch", "sh"].iter().any(|e| w[..w.len() - 2].ends_with(e));
    let stripped = strip("ization", "ize", 4)
        .or_else(|| strip("ational", "ate", 4))
        .or_else(|| strip("ations", "ate", 4))
        .or_else(|| strip("ation", "ate", 4))
        .or_else(|| strip("ments", "", 5))
        .or_else(|| strip("ment", "", 5))
        .or_else(|| strip("ies", "y", 4))
        .or_else(|| strip("ied", "y", 4))
        .or_else(|| strip("sses", "ss", 4))
        .or_else(|| if sibilant_es { strip("es", "", 3) } else { None })
        .or_else(|| strip("ing", "", 4))
        .or_else(|| strip("ed", "", 4))
        .or_else(|| {
            let ends_plain_s = w.ends_with('s') && !["ss", "us", "is"].iter().any(|e| w.ends_with(e));
            if ends_plain_s { strip("s", "", 4) } else { None }
        })
        .unwrap_or_else(|| w.clone());
    // "validate"/"validating" -> "validat"; "rule" stays "rule" (len 4)
    if stripped.len() > 4 && stripped.ends_with('e') {
        stripped[..stripped.len() - 1].to_string()
    } else {
        stripped
    }
}

const STEM_EXCEPTIONS: &[&str] = &["news", "series", "species", "lens", "gas", "bus", "yes", "always", "perhaps", "thus", "its"];

/// Character trigrams of `#word#` (padded), used for typo-tolerant term lookup.
pub fn trigrams(w: &str) -> HashSet<String> {
    let padded: Vec<char> = format!("#{w}#").chars().collect();
    let mut out = HashSet::new();
    if padded.len() < 3 {
        out.insert(padded.iter().collect());
        return out;
    }
    for win in padded.windows(3) {
        out.insert(win.iter().collect());
    }
    out
}

pub fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count();
    inter as f64 / (a.len() + b.len() - inter) as f64
}

/// Sentence split on terminal punctuation / blank lines; keeps sentences non-empty.
pub fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        cur.push(c);
        let end = matches!(c, '。' | '！' | '？')
            || (matches!(c, '.' | '!' | '?' | '\n') && chars.get(i + 1).map_or(true, |n| n.is_whitespace()));
        if end {
            let s = cur.trim();
            if s.chars().any(|c| c.is_alphanumeric()) {
                out.push(s.to_string());
            }
            cur.clear();
        }
    }
    let s = cur.trim();
    if s.chars().any(|c| c.is_alphanumeric()) {
        out.push(s.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_identifiers() {
        assert_eq!(split_words("Validate_Token"), vec!["validate", "token"]);
        assert_eq!(split_words("sessionUUIDToken"), vec!["session", "uuid", "token"]);
        assert_eq!(split_words("auth-module v2"), vec!["auth", "module", "v", "2"]);
    }

    #[test]
    fn stems_meet() {
        for (a, b) in [
            ("tokens", "token"), ("validating", "validate"), ("policies", "policy"),
            ("rules", "rule"), ("files", "file"), ("roles", "role"), ("notes", "note"),
            ("boxes", "box"), ("matches", "match"), ("applied", "apply"), ("approved", "approve"),
            ("reimbursements", "reimbursement"), ("families", "family"),
        ] {
            assert_eq!(stem(a), stem(b), "{a} vs {b}");
        }
        assert_eq!(stem("class"), "class");
    }

    #[test]
    fn stems_do_not_merge_distinct_words() {
        assert_ne!(stem("apply"), stem("app"));
        assert_ne!(stem("reply"), stem("rep"));
        assert_eq!(stem("speed"), "speed");
        assert_eq!(stem("family"), "family");
        assert_eq!(stem("news"), "news");
        assert_ne!(stem("notes"), stem("not"));
    }

    #[test]
    fn unspaced_scripts_become_bigrams() {
        assert_eq!(terms("报销需要收据"), vec!["报销", "销需", "需要", "要收", "收据"]);
        // a query term for a sub-phrase now hits the document's term set
        let doc = terms("报销需要收据。差旅必须事先批准");
        assert!(terms("收据").iter().all(|t| doc.contains(t)));
        assert_eq!(terms("領収書 receipt"), vec!["領収", "収書", "receipt"]);
        assert_eq!(terms("字"), vec!["字"]);
    }

    #[test]
    fn trigram_similarity_tolerates_typos() {
        let a = trigrams("reimbursement");
        let b = trigrams("reimbursment");
        assert!(jaccard(&a, &b) > 0.6, "{}", jaccard(&a, &b));
        assert!(jaccard(&a, &trigrams("banana")) < 0.1);
    }

    #[test]
    fn sentences_split() {
        let s = sentences("Travel must be approved. Receipts are required!\nDone");
        assert_eq!(s.len(), 3);
    }
}
