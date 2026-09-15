use std::collections::BTreeMap;
use std::sync::LazyLock;

use linkify::{LinkFinder, LinkKind};
use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

use crate::models::exp::minecraft::Language;
use crate::models::projects::Project;

pub(super) use super::language::{
    has_sufficient_english_blocks, is_confidently_non_english_short_text,
    is_likely_english_summary,
};

static WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{L}\p{M}\p{N}]+").unwrap());
static SPAM_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"[\p{L}\p{M}\p{N}]+(?:['_\u{2019}.:+/-][\p{L}\p{M}\p{N}]+)*"#)
        .unwrap()
});
static SUMMARY_LINK_FINDER: LazyLock<LinkFinder> = LazyLock::new(|| {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]).url_must_have_scheme(false);
    finder
});
static MARKDOWN_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!?\[([^\]]*)\]\([^)]+\)").unwrap());
static HTML_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<!--.*?-->|</?[a-z][^>]*>").unwrap());
static HTML_ENTITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"&(?:#[0-9]+|#[xX][0-9a-fA-F]+|[a-zA-Z][a-zA-Z0-9]+);").unwrap()
});
static HTML_OPEN_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<([a-z][\w:-]*)\b[^>]*>").unwrap());
static HTML_CLOSE_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)</([a-z][\w:-]*)\s*>").unwrap());
static CODE_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)```.*?```").unwrap());
static INLINE_CODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`([^`]*)`").unwrap());
static MARKDOWN_IMAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\([^)]+\)").unwrap());
static HTML_IMAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<img\b[^>]*>").unwrap());
static ALT_ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\balt\s*=\s*(?:"([^"]*)"|'([^']*)')"#).unwrap()
});
static DESCRIPTION_LINK_FINDER: LazyLock<LinkFinder> = LazyLock::new(|| {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]).url_must_have_scheme(false);
    finder
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProfanityKind {
    Profanity,
    Slur,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProfanityMatch {
    pub(super) kind: ProfanityKind,
    pub(super) raw_text: String,
}

const SLUR_TERMS: &[&str] = &[
    "beaner",
    "cameljockey",
    "chankoro",
    "chink",
    "chingchong",
    "coon",
    "cottonpic",
    "cottonpik",
    "darkie",
    "downie",
    "dyke",
    "fag",
    "gook",
    "jap",
    "jigabo",
    "junglebunny",
    "kike",
    "koon",
    "niqa",
    "nigga",
    "niqqa",
    "niggu",
    "niqqu",
    "niggr",
    "nigger",
    "niglet",
    "nignog",
    "paki",
    "raghead",
    "retard",
    "trannie",
    "tranny",
    "wetback",
];

const PROFANITY_TERMS: &[&str] = &[
    "asshole",
    "bastard",
    "bitch",
    "bullshit",
    "cum",
    "cunt",
    "douchebag",
    "fck",
    "fuck",
    "incest",
    "motherfucker",
    "pussy",
    "shit",
    "slut",
    "twat",
    "whore",
];

pub(super) fn normalize_project_field_text(text: &str) -> String {
    text.trim().nfc().collect()
}

pub(super) fn project_text_similarity(left: &str, right: &str) -> f64 {
    let left = normalized_for_similarity(left);
    let right = normalized_for_similarity(right);
    let longest_length = left.len().max(right.len());
    if longest_length == 0 {
        return 0.0;
    }

    1.0 - levenshtein_distance(&left, &right) as f64 / longest_length as f64
}

fn normalized_for_similarity(text: &str) -> Vec<char> {
    normalize_project_field_text(text)
        .to_lowercase()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn levenshtein_distance(left: &[char], right: &[char]) -> usize {
    if left.len() > right.len() {
        return levenshtein_distance(right, left);
    }

    let mut previous_row = (0..=left.len()).collect::<Vec<_>>();
    for (right_index, right_character) in right.iter().enumerate() {
        let mut current_row = Vec::with_capacity(left.len() + 1);
        current_row.push(right_index + 1);
        for (left_index, left_character) in left.iter().enumerate() {
            current_row.push(
                (current_row[left_index] + 1)
                    .min(previous_row[left_index + 1] + 1)
                    .min(
                        previous_row[left_index]
                            + usize::from(left_character != right_character),
                    ),
            );
        }
        previous_row = current_row;
    }
    previous_row[left.len()]
}

pub(super) fn js_string_length(text: &str) -> usize {
    text.encode_utf16().count()
}

pub(super) fn profanity_matches(text: &str) -> Vec<ProfanityMatch> {
    let (prepared, raw_ranges) = prepare_profanity_text(text);
    let mut spans = term_spans(&prepared, PROFANITY_TERMS, false);
    spans.extend(term_spans(&prepared, SLUR_TERMS, true));
    spans.sort_unstable_by(|left, right| {
        left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1))
    });

    let mut matches = Vec::new();
    let mut previous_end = 0;
    for (start, end) in spans {
        if start < previous_end {
            continue;
        }
        push_profanity_match(text, &raw_ranges, start, end, &mut matches);
        previous_end = end;
    }
    matches
}

fn term_spans(
    text: &str,
    terms: &[&str],
    match_repeated_letters: bool,
) -> Vec<(usize, usize)> {
    let characters = text.chars().collect::<Vec<_>>();
    let mut spans = Vec::new();

    for start in 0..characters.len() {
        if start > 0 && is_profanity_word_character(characters[start - 1]) {
            continue;
        }

        let end = terms
            .iter()
            .flat_map(|term| {
                [
                    if match_repeated_letters {
                        match_repeated_term(&characters, start, term)
                    } else {
                        match_exact_term(&characters, start, term)
                    },
                    match_separated_term(&characters, start, term),
                ]
            })
            .flatten()
            .max();
        if let Some(end) = end {
            spans.push((start, end));
        }
    }

    spans
}

fn match_exact_term(
    characters: &[char],
    start: usize,
    term: &str,
) -> Option<usize> {
    let mut input_index = start;
    for expected in term.chars() {
        if characters.get(input_index) != Some(&expected) {
            return None;
        }
        input_index += 1;
    }

    is_whole_word_end(characters, input_index).then_some(input_index)
}

fn match_repeated_term(
    characters: &[char],
    start: usize,
    term: &str,
) -> Option<usize> {
    let term = term.as_bytes();
    let mut input_index = start;
    let mut term_index = 0;

    while term_index < term.len() {
        let expected = term[term_index] as char;
        let mut required = 1;
        while term.get(term_index + required) == Some(&term[term_index]) {
            required += 1;
        }

        let mut matched = 0;
        while characters.get(input_index) == Some(&expected) {
            matched += 1;
            input_index += 1;
        }
        if matched < required {
            return None;
        }
        term_index += required;
    }

    is_whole_word_end(characters, input_index).then_some(input_index)
}

fn match_separated_term(
    characters: &[char],
    start: usize,
    term: &str,
) -> Option<usize> {
    let mut input_index = start;
    let mut term = term.bytes().peekable();

    while let Some(expected) = term.next() {
        if characters.get(input_index) != Some(&(expected as char)) {
            return None;
        }
        input_index += 1;

        if term.peek().is_some() {
            let separator_start = input_index;
            while characters.get(input_index).is_some_and(|character| {
                !is_profanity_word_character(*character)
            }) {
                input_index += 1;
            }
            if input_index == separator_start {
                return None;
            }
        }
    }

    is_whole_word_end(characters, input_index).then_some(input_index)
}

fn is_whole_word_end(characters: &[char], end: usize) -> bool {
    characters
        .get(end)
        .is_none_or(|character| !is_profanity_word_character(*character))
}

fn is_profanity_word_character(character: char) -> bool {
    character.is_alphanumeric()
        || character == '_'
        || is_in_ranges(
            character as u32,
            &[
                (0x0300, 0x036f),
                (0x1ab0, 0x1aff),
                (0x1dc0, 0x1dff),
                (0x20d0, 0x20ff),
                (0xfe20, 0xfe2f),
            ],
        )
}

fn prepare_profanity_text(text: &str) -> (String, Vec<(usize, usize)>) {
    let mut prepared = String::new();
    let mut raw_ranges = Vec::new();

    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        for character in grapheme.nfkc().flat_map(char::to_lowercase) {
            if is_invisible_separator(character) {
                continue;
            }

            prepared.push(normalize_obfuscated_character(character));
            raw_ranges.push((start, end));
        }
    }

    (prepared, raw_ranges)
}

fn push_profanity_match(
    text: &str,
    raw_ranges: &[(usize, usize)],
    start: usize,
    end: usize,
    matches: &mut Vec<ProfanityMatch>,
) {
    let Some(raw_text) = raw_text_for_span(text, raw_ranges, (start, end))
    else {
        return;
    };
    if normalize_project_field_text(raw_text).to_lowercase() == "кооп" {
        return;
    }
    matches.push(ProfanityMatch {
        kind: if is_slur(raw_text) {
            ProfanityKind::Slur
        } else {
            ProfanityKind::Profanity
        },
        raw_text: raw_text.to_owned(),
    });
}

fn raw_text_for_span<'a>(
    text: &'a str,
    raw_ranges: &[(usize, usize)],
    (start, end): (usize, usize),
) -> Option<&'a str> {
    let &(raw_start, _) = raw_ranges.get(start)?;
    let &(_, raw_end) = raw_ranges.get(end.checked_sub(1)?)?;
    text.get(raw_start..raw_end)
}

fn is_slur(raw_text: &str) -> bool {
    let normalized = normalized_profanity_term(raw_text);
    SLUR_TERMS
        .iter()
        .any(|term| collapse_duplicate_letters(term) == normalized)
}

fn normalized_profanity_term(text: &str) -> String {
    let (prepared, _) = prepare_profanity_text(text);
    collapse_duplicate_letters(
        &prepared
            .chars()
            .filter(char::is_ascii_alphabetic)
            .collect::<String>(),
    )
}

fn collapse_duplicate_letters(text: &str) -> String {
    let mut collapsed = String::with_capacity(text.len());
    let mut previous = None;
    for character in text.chars() {
        if previous != Some(character) {
            collapsed.push(character);
            previous = Some(character);
        }
    }
    collapsed
}

fn normalize_obfuscated_character(character: char) -> char {
    match character {
        '@' | '4' => 'a',
        '8' => 'b',
        '(' | '[' | '{' | 'с' | 'ϲ' => 'c',
        '3' | 'е' | 'ε' => 'e',
        '6' | '9' => 'g',
        '!' | '1' | '/' | '|' | 'і' | 'ι' | 'ı' => 'i',
        'ј' => 'j',
        'к' | 'κ' => 'k',
        'м' | 'μ' => 'm',
        'п' => 'n',
        '0' | 'о' | 'ο' => 'o',
        'р' | 'ρ' => 'p',
        '$' | '5' | 'ѕ' => 's',
        '+' | '7' | 'т' | 'τ' => 't',
        'υ' | 'ս' => 'u',
        'х' | 'χ' => 'x',
        'у' | 'γ' => 'y',
        '2' => 'z',
        _ => character,
    }
}

fn is_invisible_separator(character: char) -> bool {
    let code = character as u32;
    is_in_ranges(
        code,
        &[
            (0x00ad, 0x00ad),
            (0x034f, 0x034f),
            (0x061c, 0x061c),
            (0x115f, 0x1160),
            (0x17b4, 0x17b5),
            (0x180b, 0x180f),
            (0x200b, 0x200f),
            (0x202a, 0x202e),
            (0x2060, 0x206f),
            (0x3164, 0x3164),
            (0xfe00, 0xfe0f),
            (0xfeff, 0xfeff),
            (0xffa0, 0xffa0),
            (0xe0100, 0xe01ef),
        ],
    )
}

pub(super) fn has_non_standard_text(text: &str) -> bool {
    non_standard_character_count(text) > 0
}

pub(super) fn non_standard_text_ratio(text: &str) -> f64 {
    let character_count = text.chars().count();
    if character_count == 0 {
        return 0.0;
    }

    non_standard_character_count(text) as f64 / character_count as f64
}

fn non_standard_character_count(text: &str) -> usize {
    let mut count = 0;
    let mut combining_marks = 0;
    let mut has_base = false;

    for character in text.chars() {
        let code = character as u32;
        let is_combining = is_in_ranges(
            code,
            &[
                (0x0300, 0x036f),
                (0x1ab0, 0x1aff),
                (0x1dc0, 0x1dff),
                (0x20d0, 0x20ff),
                (0xfe20, 0xfe2f),
            ],
        );
        if is_combining {
            combining_marks += 1;
            if !has_base || combining_marks > 2 {
                count += 1;
            }
            continue;
        }

        combining_marks = 0;
        has_base = !character.is_whitespace();
        let allowed_control = matches!(character, '\n' | '\r' | '\t');
        let control = character.is_control() && !allowed_control;
        let private_use = is_in_ranges(
            code,
            &[(0xe000, 0xf8ff), (0xf0000, 0xffffd), (0x100000, 0x10fffd)],
        );
        let invisible = is_in_ranges(
            code,
            &[
                (0x00ad, 0x00ad),
                (0x061c, 0x061c),
                (0x200e, 0x200f),
                (0x202a, 0x202e),
                (0x2060, 0x206f),
                (0xfeff, 0xfeff),
            ],
        );
        let fancy = is_in_ranges(
            code,
            &[
                (0x02b0, 0x02ff),
                (0x1d400, 0x1d7ff),
                (0x2460, 0x24ff),
                (0x2070, 0x209f),
                (0x2100, 0x214f),
                (0xfb00, 0xfb06),
                (0xff10, 0xff19),
                (0xff21, 0xff3a),
                (0xff41, 0xff5a),
                (0x1f100, 0x1f1ad),
            ],
        ) && !matches!(
            code,
            0x02d6
                | 0x02d7
                | 0x02d8
                | 0x02d9
                | 0x02da
                | 0x02db
                | 0x02dc
                | 0x02dd
                | 0x207a
                | 0x207b
                | 0x208a
                | 0x208b
                | 0x2120
                | 0x2122
                | 0x2139
        );

        if control || private_use || invisible || fancy {
            count += 1;
        }
    }

    count
}

fn is_in_ranges(code: u32, ranges: &[(u32, u32)]) -> bool {
    ranges
        .iter()
        .any(|(start, end)| code >= *start && code <= *end)
}

pub(super) fn contains_spam(text: &str) -> bool {
    let normalized =
        text.nfc().flat_map(char::to_lowercase).collect::<String>();
    let mut previous = None;
    let mut repeated = 0;
    for character in normalized.chars() {
        if !character.is_whitespace() && previous == Some(character) {
            repeated += 1;
        } else {
            repeated = usize::from(!character.is_whitespace());
            previous = Some(character);
        }
        if repeated >= 8 {
            return true;
        }
    }

    let words = WORD
        .find_iter(&normalized)
        .map(|word| word.as_str())
        .collect::<Vec<_>>();
    let mut repeated_words = 1;
    for pair in words.windows(2) {
        repeated_words = if pair[0] == pair[1] {
            repeated_words + 1
        } else {
            1
        };
        if repeated_words >= 4 {
            return true;
        }
    }

    let max_phrase_words = 8.min(words.len() / 3);
    for phrase_words in (2..=max_phrase_words).rev() {
        for phrases in words.windows(phrase_words * 3) {
            if phrases[..phrase_words]
                == phrases[phrase_words..phrase_words * 2]
                && phrases[..phrase_words]
                    == phrases[phrase_words * 2..phrase_words * 3]
            {
                return true;
            }
        }
    }

    false
}

pub(super) fn contains_description_spam(markdown: &str) -> bool {
    const MIN_CHARACTER_RUN: usize = 16;
    const MIN_CHARACTER_EXCESS: usize = 20;
    const MIN_CHARACTER_EXCESS_PERCENT: usize = 10;
    const EXTREME_CHARACTER_RUN: usize = 64;
    const MIN_REPEATED_WORDS: usize = 6;
    const MIN_REPEATED_PHRASE_WORDS: usize = 12;
    const MIN_DUPLICATE_BLOCK_PERCENT: usize = 20;

    let blocks = extract_description_spam_blocks(markdown);
    let mut block_counts = BTreeMap::<Vec<String>, usize>::new();
    let mut visible_characters = 0;
    let mut repeated_character_excess = 0;
    let mut longest_character_run = 0;
    let mut total_words = 0;

    for block in blocks {
        let words = spam_words(&block);
        if words.is_empty() {
            continue;
        }
        total_words += words.len();

        update_character_repetition(
            &words,
            &mut visible_characters,
            &mut repeated_character_excess,
            &mut longest_character_run,
        );

        if has_repeated_words(&words, MIN_REPEATED_WORDS)
            || has_repeated_phrase(&words, MIN_REPEATED_PHRASE_WORDS)
            || has_repeated_ngram_density(&words)
        {
            return true;
        }

        if is_duplicate_spam_block_candidate(&block) {
            *block_counts.entry(words).or_default() += 1;
        }
    }

    let duplicate_block_words = block_counts
        .iter()
        .filter(|(words, count)| {
            **count >= 3 && words.len() * **count >= MIN_REPEATED_PHRASE_WORDS
        })
        .map(|(words, count)| words.len() * (count - 1))
        .sum::<usize>();
    if duplicate_block_words > 0
        && duplicate_block_words * 100
            >= total_words * MIN_DUPLICATE_BLOCK_PERCENT
    {
        return true;
    }

    longest_character_run >= EXTREME_CHARACTER_RUN
        || (longest_character_run >= MIN_CHARACTER_RUN
            && repeated_character_excess >= MIN_CHARACTER_EXCESS
            && repeated_character_excess * 100
                >= visible_characters * MIN_CHARACTER_EXCESS_PERCENT)
}

fn extract_description_spam_blocks(markdown: &str) -> Vec<String> {
    let with_inline_code = INLINE_CODE.replace_all(markdown, "$1");
    let without_images = MARKDOWN_IMAGE.replace_all(&with_inline_code, " ");
    let with_link_labels = MARKDOWN_LINK.replace_all(&without_images, "$1");
    let without_html_images = HTML_IMAGE.replace_all(&with_link_labels, " ");
    let without_html = HTML_TAG.replace_all(&without_html_images, " ");
    let without_entities = HTML_ENTITY.replace_all(&without_html, " ");
    let without_links = text_without_explicit_links(&without_entities);

    let mut blocks = Vec::new();
    let mut paragraph = String::new();
    for line in without_links.lines() {
        let line = line.trim();
        if line.is_empty() {
            push_spam_block(&mut blocks, &mut paragraph);
            continue;
        }

        if is_standalone_spam_block(line) {
            push_spam_block(&mut blocks, &mut paragraph);
            blocks.push(line.to_owned());
        } else {
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(line);
        }
    }
    push_spam_block(&mut blocks, &mut paragraph);

    blocks
}

fn text_without_explicit_links(text: &str) -> String {
    let mut without_links = String::with_capacity(text.len());
    let mut previous_end = 0;

    for link in DESCRIPTION_LINK_FINDER.links(text).filter(|link| {
        let link = link.as_str();
        link.contains("://") || link.starts_with("www.")
    }) {
        without_links.push_str(&text[previous_end..link.start()]);
        without_links.push(' ');
        previous_end = link.end();
    }
    without_links.push_str(&text[previous_end..]);
    without_links
}

fn push_spam_block(blocks: &mut Vec<String>, paragraph: &mut String) {
    if !paragraph.is_empty() {
        blocks.push(std::mem::take(paragraph));
    }
}

fn is_standalone_spam_block(line: &str) -> bool {
    let line = line.trim_start_matches('>');
    let trimmed = line.trim_start();
    let is_heading = trimmed.starts_with('#');
    let is_list_item = ["- ", "* ", "+ ", "•"]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
        || trimmed.split_once(". ").is_some_and(|(prefix, _)| {
            prefix.chars().all(|char| char.is_ascii_digit())
        });
    let is_table_row = trimmed.contains('|');

    is_heading || is_list_item || is_table_row
}

fn is_duplicate_spam_block_candidate(block: &str) -> bool {
    let trimmed = block.trim_start_matches('>').trim_start();
    !trimmed.starts_with('#') && !trimmed.contains('|')
}

fn spam_words(text: &str) -> Vec<String> {
    SPAM_TOKEN
        .find_iter(text)
        .map(|word| word.as_str().nfc().flat_map(char::to_lowercase).collect())
        .collect()
}

fn update_character_repetition(
    words: &[String],
    visible_characters: &mut usize,
    repeated_character_excess: &mut usize,
    longest_character_run: &mut usize,
) {
    for word in words {
        *visible_characters += word.chars().count();
        let mut previous = None;
        let mut run: usize = 0;
        for character in word.chars() {
            if previous == Some(character) {
                run += 1;
            } else {
                *repeated_character_excess += run.saturating_sub(3);
                *longest_character_run = (*longest_character_run).max(run);
                previous = Some(character);
                run = 1;
            }
        }
        *repeated_character_excess += run.saturating_sub(3);
        *longest_character_run = (*longest_character_run).max(run);
    }
}

fn has_repeated_words(words: &[String], minimum_repetitions: usize) -> bool {
    let mut repetitions = 1;
    for pair in words.windows(2) {
        repetitions = if pair[0] == pair[1] {
            repetitions + 1
        } else {
            1
        };
        if repetitions >= minimum_repetitions {
            return true;
        }
    }

    false
}

fn has_repeated_phrase(
    words: &[String],
    minimum_repeated_words: usize,
) -> bool {
    let max_phrase_words = 8.min(words.len() / 3);
    for phrase_words in (2..=max_phrase_words).rev() {
        for start in 0..=words.len() - phrase_words * 3 {
            let phrase = &words[start..start + phrase_words];
            let mut repetitions = 1;
            while start + phrase_words * (repetitions + 1) <= words.len()
                && phrase
                    == &words[start + phrase_words * repetitions
                        ..start + phrase_words * (repetitions + 1)]
            {
                repetitions += 1;
            }
            if repetitions >= 3
                && phrase_words * repetitions >= minimum_repeated_words
            {
                return true;
            }
        }
    }

    false
}

fn has_repeated_ngram_density(words: &[String]) -> bool {
    const NGRAM_WORDS: usize = 4;
    const MIN_WORDS: usize = 30;
    const MIN_DUPLICATE_NGRAM_PERCENT: usize = 50;
    const MIN_COVERED_WORD_PERCENT: usize = 70;

    if words.len() < MIN_WORDS {
        return false;
    }

    let mut ngram_counts = BTreeMap::<&[String], usize>::new();
    for ngram in words.windows(NGRAM_WORDS) {
        *ngram_counts.entry(ngram).or_default() += 1;
    }

    let total_ngrams = words.len() - NGRAM_WORDS + 1;
    let duplicate_ngrams = ngram_counts
        .values()
        .map(|count| count.saturating_sub(1))
        .sum::<usize>();
    if duplicate_ngrams * 100 < total_ngrams * MIN_DUPLICATE_NGRAM_PERCENT {
        return false;
    }

    let mut covered_words = vec![false; words.len()];
    for (start, ngram) in words.windows(NGRAM_WORDS).enumerate() {
        if ngram_counts.get(ngram).is_some_and(|count| *count > 1) {
            covered_words[start..start + NGRAM_WORDS].fill(true);
        }
    }
    covered_words.iter().filter(|covered| **covered).count() * 100
        >= words.len() * MIN_COVERED_WORD_PERCENT
}

pub(super) fn find_link_or_ip(text: &str) -> Option<String> {
    SUMMARY_LINK_FINDER.links(text).find_map(|link| {
        let raw = link.as_str();
        if raw.contains("://") {
            return Some(raw.to_owned());
        }

        Url::parse(&format!("https://{raw}"))
            .ok()
            .filter(|url| {
                url.host_str().is_some_and(|hostname| {
                    hostname.parse::<std::net::IpAddr>().is_ok()
                        || psl::domain(hostname.as_bytes()).is_some_and(
                            |domain| domain.suffix().typ().is_some(),
                        )
                })
            })
            .map(|_| raw.to_owned())
    })
}

pub(super) fn has_summary_formatting(summary: &str) -> bool {
    has_paired_html_formatting(summary)
        || MARKDOWN_LINK.is_match(summary)
        || Parser::new(summary)
            .any(|event| matches!(event, Event::Start(Tag::Emphasis)))
        || summary.lines().any(|line| {
            let line = line.trim_start();
            line.starts_with('#')
                || line.starts_with('>')
                || line.starts_with("- ")
                || line.starts_with("* ")
                || line.starts_with("+ ")
                || line.starts_with("```")
                || line.contains("**")
                || line.contains("__")
                || line.contains("~~")
                || INLINE_CODE.is_match(line)
        })
}

pub(super) fn has_paired_html_formatting(text: &str) -> bool {
    let without_code = CODE_BLOCK.replace_all(text, "");
    let without_code = INLINE_CODE.replace_all(&without_code, "");
    HTML_OPEN_TAG.captures_iter(&without_code).any(|opening| {
        let Some(tag) = opening.get(1) else {
            return false;
        };
        let Some(opening_match) = opening.get(0) else {
            return false;
        };
        HTML_CLOSE_TAG
            .captures_iter(&without_code[opening_match.end()..])
            .any(|closing| {
                closing.get(1).is_some_and(|closing_tag| {
                    closing_tag.as_str().eq_ignore_ascii_case(tag.as_str())
                })
            })
    })
}

fn strip_description_markup(markdown: &str) -> String {
    let without_code = INLINE_CODE.replace_all(markdown, " ");
    let with_image_alt = MARKDOWN_IMAGE.replace_all(&without_code, "$1");
    let without_links = MARKDOWN_LINK.replace_all(&with_image_alt, " ");
    let with_html_image_alt = HTML_IMAGE.replace_all(
        &without_links,
        |captures: &regex::Captures<'_>| {
            ALT_ATTRIBUTE
                .captures(&captures[0])
                .and_then(|captures| {
                    captures.get(1).or_else(|| captures.get(2))
                })
                .map_or_else(|| " ".to_owned(), |alt| alt.as_str().to_owned())
        },
    );
    HTML_TAG.replace_all(&with_html_image_alt, " ").into_owned()
}

pub(super) fn extract_description_text(markdown: &str) -> String {
    strip_description_markup(markdown)
        .lines()
        .map(|line| line.trim_start_matches(['>', '#']))
        .collect::<Vec<_>>()
        .join("\n")
        .replace(['*', '_', '~', '`', '>', '-', '|'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn strip_dotted_identifiers(text: &str) -> String {
    static IDENTIFIER: LazyLock<Regex> = LazyLock::new(|| {
        // matches dotted identifiers such as `com.example.mod` or `config.enabled`
        Regex::new(r"\b[A-Za-z][A-Za-z0-9_-]*(?:\.[A-Za-z][A-Za-z0-9_-]*)+\b")
            .unwrap()
    });

    IDENTIFIER.replace_all(text, " ").into_owned()
}

fn strip_command_arguments(text: &str) -> String {
    static ARGUMENT: LazyLock<Regex> = LazyLock::new(|| {
        // matches bracketed argument placeholders such as `[player]` or `[on|off]`
        Regex::new(r"\[[A-Za-z][A-Za-z0-9_ /|.-]*\]").unwrap()
    });

    ARGUMENT.replace_all(text, " ").into_owned()
}

fn strip_technical_syntax(line: &str) -> String {
    static COMMAND: LazyLock<Regex> = LazyLock::new(|| {
        // matches slash commands such as `/help` or `/minecraft:give`.
        Regex::new(r"(^|[\s|,(])/[A-Za-z][A-Za-z0-9_:./-]*").unwrap()
    });

    let without_commands = COMMAND.replace_all(line, "$1");
    let without_identifiers = strip_dotted_identifiers(&without_commands);

    if COMMAND.is_match(line) {
        strip_command_arguments(&without_identifiers)
    } else {
        without_identifiers
    }
}

fn strip_loaders(text: &str) -> String {
    static LOADER_WORD: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(?:fabric|forge|quilt|neoforge|liteloader)\b")
            .unwrap()
    });

    LOADER_WORD.replace_all(text, " ").into_owned()
}

fn description_language_input(markdown: &str) -> String {
    let readable = strip_description_markup(markdown);
    let readable = text_without_explicit_links(&readable);
    let readable = strip_loaders(&readable);
    readable
        .lines()
        .map(strip_technical_syntax)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn extract_description_blocks(markdown: &str) -> Vec<String> {
    const MIN_PARAGRAPH_WORDS: usize = 8;

    let readable = description_language_input(markdown);
    let mut blocks = Vec::new();
    let mut paragraph = Vec::new();
    let mut paragraph_word_count = 0;
    let mut previous_line_was_heading = false;
    let mut follows_blank_line = false;

    for line in readable.lines() {
        let line = line.trim();
        if line.is_empty() {
            follows_blank_line = true;
            continue;
        }

        let is_heading = line.starts_with('#');
        let line_word_count =
            WORD.find_iter(line).take(MIN_PARAGRAPH_WORDS).count();
        let starts_new_paragraph = follows_blank_line
            && !previous_line_was_heading
            && (paragraph_word_count >= MIN_PARAGRAPH_WORDS
                || line_word_count >= MIN_PARAGRAPH_WORDS);

        if is_heading || starts_new_paragraph {
            push_description_block(&mut blocks, &mut paragraph);
            paragraph_word_count = 0;
        }

        paragraph.push(line);
        paragraph_word_count =
            (paragraph_word_count + line_word_count).min(MIN_PARAGRAPH_WORDS);
        previous_line_was_heading = is_heading;
        follows_blank_line = false;
    }

    push_description_block(&mut blocks, &mut paragraph);
    blocks
}

fn push_description_block(blocks: &mut Vec<String>, paragraph: &mut Vec<&str>) {
    if paragraph.is_empty() {
        return;
    }

    let block = extract_description_text(&paragraph.join("\n"));
    if !block.is_empty() {
        blocks.push(block);
    }
    paragraph.clear();
}

pub(super) fn has_image_without_alt_text(markdown: &str) -> bool {
    let without_code = INLINE_CODE.replace_all(markdown, "");
    MARKDOWN_IMAGE
        .captures_iter(&without_code)
        .any(|captures| captures[1].trim().is_empty())
        || HTML_IMAGE.find_iter(&without_code).any(|image| {
            ALT_ATTRIBUTE
                .captures(image.as_str())
                .and_then(|captures| {
                    captures.get(1).or_else(|| captures.get(2))
                })
                .is_none_or(|alt| alt.as_str().trim().is_empty())
        })
}

pub(super) fn project_requires_english(project: &Project) -> bool {
    let has_locale_tag = project
        .categories
        .iter()
        .chain(&project.additional_categories)
        .any(|category| category == "locale");
    let is_english_server = project
        .components
        .minecraft_server
        .as_ref()
        .is_some_and(|server| server.languages.contains(&Language::En));

    (project.components.minecraft_java_server.is_none() && !has_locale_tag)
        || is_english_server
}

#[cfg(test)]
mod tests {
    use super::{has_summary_formatting, project_text_similarity};

    #[test]
    fn summary_detects_markdown_emphasis() {
        for summary in [
            "*this*",
            "Adds *new features* to Minecraft",
            "*a*",
            "_this_",
        ] {
            assert!(has_summary_formatting(summary), "{summary:?}");
        }
    }

    #[test]
    fn summary_allows_literal_asterisks() {
        for summary in [
            "A single * asterisk",
            "2 * 3 * 4",
            r"\*this\*",
            "An unmatched *asterisk",
        ] {
            assert!(!has_summary_formatting(summary), "{summary:?}");
        }
    }

    #[test]
    fn similarity_ignores_case_whitespace_and_unicode_composition() {
        assert_eq!(
            project_text_similarity(" Café tools ", "CAFE\u{301}\nTOOLS"),
            1.0
        );
    }

    #[test]
    fn similarity_distinguishes_the_eighty_percent_boundary() {
        assert!(project_text_similarity("abcde", "abcdx") >= 0.8);
        assert!(project_text_similarity("abcde", "abcxy") < 0.8);
    }

    #[test]
    fn empty_fields_do_not_match() {
        assert_eq!(project_text_similarity(" ", "\n"), 0.0);
        assert_eq!(project_text_similarity("", "some text"), 0.0);
        assert_eq!(project_text_similarity("some text", ""), 0.0);
    }
}
