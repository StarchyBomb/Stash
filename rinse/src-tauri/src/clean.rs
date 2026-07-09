use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Rules {
    /// รวมบรรทัดที่ถูกตัดพัง ๆ (จาก PDF / อีเมล) กลับเป็นย่อหน้า
    pub fix_linebreaks: bool,
    /// smart quotes → ตรง ๆ, em-dash → "-", ellipsis → "..."
    pub normalize_punct: bool,
    /// ตัด tracking param ในลิงก์ (utm_*, fbclid, gclid, ...)
    pub strip_tracking: bool,
    /// เก็บกวาดช่องว่าง/บรรทัดว่างเกิน
    pub tidy_whitespace: bool,
    /// แปลงเป็นรายการหัวข้อย่อย
    pub bullet_list: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            fix_linebreaks: true,
            normalize_punct: true,
            strip_tracking: true,
            tidy_whitespace: true,
            bullet_list: false,
        }
    }
}

pub fn clean(input: &str, rules: &Rules) -> String {
    // normalize line endings + ตัดอักขระล่องหน (ทำเสมอ)
    let mut text: String = input.replace("\r\n", "\n").replace('\r', "\n");
    text = text
        .chars()
        .filter(|c| !matches!(c, '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{00AD}'))
        .collect();

    if rules.normalize_punct {
        text = normalize_punct(&text);
    }
    if rules.fix_linebreaks {
        text = fix_linebreaks(&text);
    }
    if rules.strip_tracking {
        text = strip_tracking(&text);
    }
    if rules.tidy_whitespace {
        text = tidy_whitespace(&text);
    }
    if rules.bullet_list {
        text = format_bullets(&text);
    }
    text
}

fn normalize_punct(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{2032}' => out.push('\''),
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{2033}' => out.push('"'),
            '\u{2013}' | '\u{2014}' | '\u{2015}' => out.push('-'),
            '\u{2026}' => out.push_str("..."),
            '\u{00A0}' | '\u{2007}' | '\u{202F}' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

fn is_thai(c: char) -> bool {
    ('\u{0E00}'..='\u{0E7F}').contains(&c)
}

static LIST_ITEM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:[-•*>‣▪]|\d{1,3}[.)]\s)").unwrap());

fn looks_like_list_item(line: &str) -> bool {
    LIST_ITEM_RE.is_match(line)
}

/// บรรทัดที่โดน hard-wrap มา (เช่นก๊อปจาก PDF) → รวมกลับเป็นย่อหน้าเดียว
fn fix_linebreaks(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let line = raw.trim_end().to_string();
        if let Some(prev) = out.last_mut() {
            if should_join(prev, &line) {
                let next = line.trim_start();
                let prev_last = prev.chars().last().unwrap();
                let next_first = next.chars().next().unwrap();
                if prev.ends_with('-') && next_first.is_ascii_lowercase() {
                    // คำโดนตัดกลางด้วย hyphen: exam-\nple → example
                    prev.pop();
                    prev.push_str(next);
                } else if is_thai(prev_last) && is_thai(next_first) {
                    // ภาษาไทยไม่เว้นวรรคระหว่างคำ
                    prev.push_str(next);
                } else {
                    prev.push(' ');
                    prev.push_str(next);
                }
                continue;
            }
        }
        out.push(line);
    }
    out.join("\n")
}

fn should_join(prev: &str, next: &str) -> bool {
    if prev.is_empty() {
        return false;
    }
    let next_trim = next.trim_start();
    if next_trim.is_empty() || looks_like_list_item(next) {
        return false;
    }
    let last = prev.chars().last().unwrap();
    // จบประโยคชัดเจน → ไม่รวม
    if matches!(last, '.' | '!' | '?' | ':' | ';' | '"' | ')' | ']') {
        return false;
    }
    let first = next_trim.chars().next().unwrap();
    // รวมเฉพาะเมื่อบรรทัดถัดไปดูเป็น "ประโยคต่อเนื่อง" (ตัวเล็ก/ไทย)
    let continuing = first.is_ascii_lowercase() || is_thai(first);
    let prev_ok = last.is_alphanumeric() || is_thai(last) || matches!(last, ',' | '-');
    continuing && prev_ok
}

static URL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"https?://[^\s<>"'\)\]]+"#).unwrap());

fn is_tracking_param(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.starts_with("utm_")
        || matches!(
            n.as_str(),
            "fbclid"
                | "gclid"
                | "gclsrc"
                | "dclid"
                | "wbraid"
                | "gbraid"
                | "msclkid"
                | "yclid"
                | "twclid"
                | "ttclid"
                | "igshid"
                | "igsh"
                | "mc_cid"
                | "mc_eid"
                | "mkt_tok"
                | "si"
                | "spm"
                | "vero_id"
                | "_hsenc"
                | "_hsmi"
                | "oly_anon_id"
                | "oly_enc_id"
                | "ref_src"
                | "ref_url"
                | "share_id"
        )
}

fn clean_url(url: &str) -> String {
    let (base, rest) = match url.split_once('?') {
        Some((b, r)) => (b, r),
        None => return url.to_string(),
    };
    let (query, frag) = match rest.split_once('#') {
        Some((q, f)) => (q, Some(f)),
        None => (rest, None),
    };
    let kept: Vec<&str> = query
        .split('&')
        .filter(|p| {
            let name = p.split('=').next().unwrap_or("");
            !name.is_empty() && !is_tracking_param(name)
        })
        .collect();
    let mut out = base.to_string();
    if !kept.is_empty() {
        out.push('?');
        out.push_str(&kept.join("&"));
    }
    if let Some(f) = frag {
        out.push('#');
        out.push_str(f);
    }
    out
}

fn strip_tracking(text: &str) -> String {
    URL_RE
        .replace_all(text, |caps: &regex::Captures| {
            clean_url(caps.get(0).unwrap().as_str())
        })
        .into_owned()
}

static MULTI_SPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t]{2,}").unwrap());
static MULTI_NL: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").unwrap());

fn tidy_whitespace(text: &str) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .map(|line| {
            // คงย่อหน้า/indent ข้างหน้าไว้ (เผื่อเป็นโค้ด) แต่ยุบช่องว่างซ้ำข้างใน
            let trimmed_start = line.trim_start();
            let indent = &line[..line.len() - trimmed_start.len()];
            format!(
                "{}{}",
                indent,
                MULTI_SPACE.replace_all(trimmed_start.trim_end(), " ")
            )
        })
        .collect();
    let joined = lines.join("\n");
    MULTI_NL.replace_all(&joined, "\n\n").trim().to_string()
}

fn format_bullets(text: &str) -> String {
    let mut out = Vec::new();
    for line in text.split('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let cleaned = LIST_ITEM_RE.replace(trimmed, "");
        let cleaned_trim = cleaned.trim();
        if !cleaned_trim.is_empty() {
            out.push(format!("• {}", cleaned_trim));
        }
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> Rules {
        Rules::default()
    }

    #[test]
    fn joins_pdf_hard_wrapped_lines() {
        let input = "This is a sentence that was\nwrapped by a pdf viewer and\nshould become one line.";
        let out = clean(input, &all());
        assert_eq!(
            out,
            "This is a sentence that was wrapped by a pdf viewer and should become one line."
        );
    }

    #[test]
    fn joins_hyphenated_word() {
        let input = "this word was hyphen-\nated across lines";
        assert_eq!(clean(input, &all()), "this word was hyphenated across lines");
    }

    #[test]
    fn joins_thai_without_space() {
        let input = "ข้อความภาษาไทยที่โดนตัด\nบรรทัดกลางประโยค";
        assert_eq!(clean(input, &all()), "ข้อความภาษาไทยที่โดนตัดบรรทัดกลางประโยค");
    }

    #[test]
    fn keeps_paragraph_breaks_and_lists() {
        let input = "First paragraph ends here.\n\nSecond paragraph.\n- item one\n- item two";
        let out = clean(input, &all());
        assert!(out.contains("First paragraph ends here.\n\nSecond paragraph."));
        assert!(out.contains("- item one\n- item two"));
    }

    #[test]
    fn strips_tracking_params() {
        let input = "see https://example.com/page?id=42&utm_source=x&fbclid=abc123&q=hello ok";
        let out = clean(input, &all());
        assert!(out.contains("https://example.com/page?id=42&q=hello"));
        assert!(!out.contains("utm_source"));
        assert!(!out.contains("fbclid"));
    }

    #[test]
    fn strips_all_params_leaves_clean_url() {
        let input = "https://example.com/page?utm_source=x&utm_medium=y";
        assert_eq!(clean(input, &all()), "https://example.com/page");
    }

    #[test]
    fn normalizes_smart_punct() {
        let input = "\u{201C}hello\u{201D} \u{2014} it\u{2019}s fine\u{2026}";
        assert_eq!(clean(input, &all()), "\"hello\" - it's fine...");
    }

    #[test]
    fn removes_zero_width_and_nbsp() {
        let input = "a\u{200B}b\u{00A0}c";
        assert_eq!(clean(input, &all()), "ab c");
    }

    #[test]
    fn tidies_whitespace() {
        let input = "too   many    spaces\n\n\n\nand blank lines   ";
        assert_eq!(clean(input, &all()), "too many spaces\n\nand blank lines");
    }

    #[test]
    fn rules_can_be_disabled() {
        let rules = Rules {
            strip_tracking: false,
            ..Rules::default()
        };
        let input = "https://example.com/?utm_source=x";
        assert!(clean(input, &rules).contains("utm_source"));
    }

    #[test]
    fn formats_bullet_list() {
        let rules = Rules {
            bullet_list: true,
            ..Rules::default()
        };
        let input = "First item\n- Second item\n* Third item\n1. Fourth item\n\n\nFifth item";
        let out = clean(input, &rules);
        assert_eq!(
            out,
            "• First item\n• Second item\n• Third item\n• Fourth item\n• Fifth item"
        );
    }
}
