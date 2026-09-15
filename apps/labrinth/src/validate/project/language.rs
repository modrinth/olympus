use std::sync::LazyLock;

use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

const MIN_ENGLISH_TO_BEST_RATIO: f64 = 0.5;
const MIN_DESCRIPTION_CONFIDENCE: f64 = 0.35;
const MIN_DESCRIPTION_MARGIN: f64 = 0.15;
const MIN_NON_ENGLISH_CONFIDENCE: f64 = 0.8;
const MIN_DOMINANT_NON_ENGLISH_CONFIDENCE: f64 = 0.6;
const MIN_DOMINANT_NON_ENGLISH_MARGIN: f64 = 0.6;
const MAX_DOMINANT_ENGLISH_CONFIDENCE: f64 = 0.05;
const MIN_NON_LATIN_CONFIDENCE: f64 = 0.5;
const MIN_SHORT_TEXT_CONFIDENCE: f64 = 0.95;
const MIN_SHORT_TEXT_MARGIN: f64 = 0.5;
const MIN_DESCRIPTION_ENGLISH_PROPORTION: f64 = 0.2;
const MIN_PASSAGE_WORDS: usize = 4;
const MIN_PASSAGE_CHARS: usize = 25;

static DETECTOR: LazyLock<LanguageDetector> =
    LazyLock::new(|| LanguageDetectorBuilder::from_all_languages().build());

static WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{L}\p{M}\p{N}]+").unwrap());

static NON_LATIN_LETTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{Alphabetic}&&[^\p{Latin}]]").unwrap());
const MIN_DESCRIPTION_WORDS: usize = 8;
const MIN_DESCRIPTION_CHARS: usize = 35;
const MIN_ENGLISH_GRAMMATICAL_WORDS: usize = 2;

struct Passage {
    text: String,
    language: Option<Language>,
    confidence: f64,
    runner_up_confidence: f64,
    eligible: bool,
    qualifies_as_english: bool,
    confidently_non_english: bool,
    alphabetic_words: usize,
    english_grammatical_words: usize,
}

#[derive(Clone, Copy, Default)]
struct Detection {
    runner_up_confidence: f64,
    language: Option<Language>,
    confidence: f64,
    english_confidence: f64,
}

impl Detection {
    fn is_english(&self) -> bool {
        self.english_to_best_ratio() >= MIN_ENGLISH_TO_BEST_RATIO
    }

    fn is_confidently_non_english(&self, text: &str) -> bool {
        let has_non_latin_evidence =
            NON_LATIN_LETTER.find_iter(text).count() >= 5;
        let minimum_confidence = if has_non_latin_evidence {
            MIN_NON_LATIN_CONFIDENCE
        } else {
            MIN_NON_ENGLISH_CONFIDENCE
        };
        let clearly_dominant = self.confidence
            >= MIN_DOMINANT_NON_ENGLISH_CONFIDENCE
            && self.confidence - self.runner_up_confidence
                >= MIN_DOMINANT_NON_ENGLISH_MARGIN
            && self.english_confidence < MAX_DOMINANT_ENGLISH_CONFIDENCE;
        (self.confidence >= minimum_confidence || clearly_dominant)
            && !self.is_english()
            && (has_non_latin_evidence
                || (text.graphemes(true).count() >= MIN_PASSAGE_CHARS
                    && alphabetic_word_count(text) >= MIN_PASSAGE_WORDS))
    }

    fn english_to_best_ratio(&self) -> f64 {
        if self.confidence > 0.0 {
            self.english_confidence / self.confidence
        } else {
            0.0
        }
    }
}

fn detect(text: &str, detector: &LanguageDetector) -> Detection {
    let scores = detector.compute_language_confidence_values(text);
    let english_confidence = scores
        .iter()
        .find(|(language, _)| *language == Language::English)
        .map_or(0.0, |(_, confidence)| *confidence);
    let best = scores.first().filter(|(_, confidence)| *confidence > 0.0);
    Detection {
        runner_up_confidence: scores.get(1).map_or(0.0, |(_, score)| *score),
        language: best.map(|(language, _)| *language),
        confidence: best.map_or(0.0, |(_, confidence)| *confidence),
        english_confidence,
    }
}

/// Short fields need stronger evidence before bypassing the usual length gate.
pub(super) fn is_confidently_non_english_short_text(text: &str) -> bool {
    let normalized: String = text.nfkc().collect();
    let detection = detect(&normalized, &DETECTOR);
    detection.confidence >= MIN_SHORT_TEXT_CONFIDENCE
        && detection.confidence - detection.runner_up_confidence
            >= MIN_SHORT_TEXT_MARGIN
        && detection.is_confidently_non_english(&normalized)
}

/// Use Lingua's inferred boundaries, then rescore each span independently so
/// English can qualify even when another language has the highest score.
fn mixed_language_passages(
    text: &str,
    detector: &LanguageDetector,
) -> Vec<String> {
    detector
        .detect_multiple_languages_of(text)
        .into_iter()
        .map(|span| {
            text[span.start_index()..span.end_index()].trim().to_owned()
        })
        .collect()
}

fn has_enough_description_content(text: &str) -> bool {
    alphabetic_word_count(text) >= MIN_DESCRIPTION_WORDS
        && text.trim().graphemes(true).count() >= MIN_DESCRIPTION_CHARS
}

/// Keep script runs disjoint so surrounding foreign text cannot qualify a short English fragment.
fn script_passages(text: &str) -> Vec<&str> {
    let mut passages = Vec::new();
    let mut start = 0;
    let mut previous = None;
    for (index, character) in text.char_indices() {
        if !character.is_alphabetic() {
            continue;
        }
        let mut buffer = [0; 4];
        let non_latin =
            NON_LATIN_LETTER.is_match(character.encode_utf8(&mut buffer));
        if previous.is_some_and(|previous| previous != non_latin) {
            passages.push(text[start..index].trim());
            start = index;
        }
        previous = Some(non_latin);
    }
    if !text[start..].trim().is_empty() {
        passages.push(text[start..].trim());
    }
    passages
}

fn description_passage(text: &str) -> Passage {
    let mut passage = classify_passage(
        text.to_owned(),
        has_enough_description_content(text),
        1.0,
        &DETECTOR,
        has_enough_description_content(text)
            || NON_LATIN_LETTER.find_iter(text).count() >= 5,
    );
    let strong = passage.confidence >= MIN_DESCRIPTION_CONFIDENCE
        && passage.confidence - passage.runner_up_confidence
            >= MIN_DESCRIPTION_MARGIN;
    let foreign_eligible =
        passage.eligible || NON_LATIN_LETTER.find_iter(text).count() >= 5;
    passage.qualifies_as_english = passage.eligible
        && strong
        && (passage.english_grammatical_words >= MIN_ENGLISH_GRAMMATICAL_WORDS
            || has_lowercase_prose(text))
        && passage.language == Some(Language::English);
    passage.confidently_non_english = foreign_eligible
        && strong
        && passage
            .language
            .is_some_and(|language| language != Language::English);
    passage.eligible = foreign_eligible;
    passage
}

/// Capitalized names and acronyms alone do not establish English prose.
fn has_lowercase_prose(text: &str) -> bool {
    WORD.find_iter(text)
        .filter(|word| {
            let word = word.as_str();
            word.chars().all(char::is_lowercase) && word.chars().count() > 1
        })
        .take(2)
        .count()
        >= 2
}

fn summary_translation_passage(text: &str) -> Passage {
    let mut passage = description_passage(text);
    if passage.qualifies_as_english
        && passage.english_grammatical_words < MIN_ENGLISH_GRAMMATICAL_WORDS
    {
        passage.qualifies_as_english = false;
    }
    passage
}

/// Bound detector input without overlapping or splitting words.
fn bounded_passages(text: &str) -> Vec<&str> {
    let mut passages = Vec::new();
    let mut start = 0;
    for (index, character) in text.char_indices() {
        if index - start >= 4000 && character.is_whitespace() {
            passages.push(text[start..index].trim());
            start = index + character.len_utf8();
        }
    }
    if !text[start..].trim().is_empty() {
        passages.push(text[start..].trim());
    }
    passages
}

/// Only replace a coherent block when Lingua finds independently supported English and foreign portions.
fn description_passages(text: &str) -> Vec<Passage> {
    bounded_passages(text)
        .into_iter()
        .flat_map(script_passages)
        .flat_map(|text| {
            let whole = description_passage(text);
            if whole.alphabetic_words < MIN_DESCRIPTION_WORDS * 2 {
                return vec![whole];
            }
            let spans = mixed_language_passages(text, &DETECTOR);
            if spans.len() <= 1 {
                return vec![whole];
            }
            let candidates = spans
                .iter()
                .map(|span| description_passage(span))
                .collect::<Vec<_>>();
            if candidates.iter().any(|span| span.qualifies_as_english)
                && candidates.iter().any(|span| span.confidently_non_english)
            {
                candidates
            } else {
                vec![whole]
            }
        })
        .collect()
}

fn alphabetic_word_count(text: &str) -> usize {
    WORD.find_iter(text)
        .filter(|word| word.as_str().chars().any(char::is_alphabetic))
        .count()
}

/// Require grammatical context before treating technical names as an English translation.
fn english_grammatical_word_count(text: &str) -> usize {
    WORD.find_iter(text)
        .filter(|word| {
            matches!(
                word.as_str().to_lowercase().as_str(),
                "a" | "an"
                    | "the"
                    | "this"
                    | "that"
                    | "these"
                    | "those"
                    | "it"
                    | "its"
                    | "you"
                    | "your"
                    | "we"
                    | "our"
                    | "they"
                    | "their"
                    | "to"
                    | "for"
                    | "with"
                    | "from"
                    | "in"
                    | "on"
                    | "of"
                    | "and"
                    | "or"
                    | "by"
                    | "into"
                    | "is"
                    | "are"
                    | "be"
                    | "can"
                    | "will"
                    | "has"
                    | "have"
                    | "does"
                    | "not"
            )
        })
        .count()
}

fn is_eligible_passage(text: &str) -> bool {
    alphabetic_word_count(text) >= MIN_PASSAGE_WORDS
        && text.graphemes(true).count() >= MIN_PASSAGE_CHARS
}

fn classify_passage(
    text: String,
    eligible: bool,
    minimum_ratio: f64,
    detector: &LanguageDetector,
    should_detect: bool,
) -> Passage {
    let detection = if should_detect {
        detect(&text, detector)
    } else {
        Detection::default()
    };
    Passage {
        runner_up_confidence: detection.runner_up_confidence,
        qualifies_as_english: eligible
            && detection.english_to_best_ratio() >= minimum_ratio,
        confidently_non_english: eligible
            && detection.is_confidently_non_english(&text),
        alphabetic_words: alphabetic_word_count(&text),
        english_grammatical_words: english_grammatical_word_count(&text),
        text,
        language: detection.language,
        confidence: detection.confidence,
        eligible,
    }
}

/// A foreign summary needs a contiguous English passage, not scattered product names.
fn summary_rescue_passages(text: &str) -> Vec<Passage> {
    script_passages(text)
        .into_iter()
        .filter(|text| !NON_LATIN_LETTER.is_match(text))
        .flat_map(|text| {
            let normalized =
                text.split_whitespace().collect::<Vec<_>>().join(" ");
            let whole = summary_translation_passage(&normalized);
            if whole.qualifies_as_english
                || !whole.eligible
                || whole.english_grammatical_words
                    < MIN_ENGLISH_GRAMMATICAL_WORDS
            {
                return vec![whole];
            }
            let spans = mixed_language_passages(&normalized, &DETECTOR);
            if spans.len() <= 1 {
                return vec![whole];
            }
            spans
                .into_iter()
                .map(|span| summary_translation_passage(&span))
                .collect()
        })
        .collect()
}

pub(super) fn is_likely_english_summary(text: &str) -> bool {
    let normalized: String = text.nfkc().collect();
    let mut passages = vec![classify_passage(
        normalized.clone(),
        true,
        MIN_ENGLISH_TO_BEST_RATIO,
        &DETECTOR,
        true,
    )];
    let requires_strong_rescue = passages[0].confidently_non_english;
    if requires_strong_rescue {
        passages.extend(summary_rescue_passages(&normalized));
    } else if NON_LATIN_LETTER.is_match(&normalized) {
        let latin = NON_LATIN_LETTER.replace_all(&normalized, " ");
        passages[0].qualifies_as_english = false;
        passages.push(classify_passage(
            latin.to_string(),
            is_eligible_passage(&latin),
            MIN_ENGLISH_TO_BEST_RATIO,
            &DETECTOR,
            is_eligible_passage(&latin),
        ));
    } else if !passages[0].qualifies_as_english {
        passages.extend(
            mixed_language_passages(&normalized, &DETECTOR)
                .into_iter()
                .map(|text| {
                    let eligible = is_eligible_passage(&text);
                    classify_passage(
                        text,
                        eligible,
                        MIN_ENGLISH_TO_BEST_RATIO,
                        &DETECTOR,
                        eligible,
                    )
                }),
        );
    }
    passages.iter().any(|passage| passage.qualifies_as_english)
        || !passages
            .iter()
            .any(|passage| passage.confidently_non_english)
}

pub(super) fn has_sufficient_english_blocks(blocks: &[String]) -> bool {
    let passages = blocks
        .iter()
        .flat_map(|block| {
            let normalized: String = block.nfkc().collect();
            description_passages(&normalized)
        })
        .collect::<Vec<_>>();
    let english_utf16_length: usize = passages
        .iter()
        .filter(|passage| passage.qualifies_as_english)
        .map(|passage| passage.text.encode_utf16().count())
        .sum();
    let total_utf16_length: usize = passages
        .iter()
        .filter(|passage| passage.eligible)
        .map(|passage| passage.text.encode_utf16().count())
        .sum();
    let has_foreign_evidence = passages
        .iter()
        .any(|passage| passage.confidently_non_english);
    let english_proportion = if total_utf16_length == 0 {
        0.0
    } else {
        english_utf16_length as f64 / total_utf16_length as f64
    };

    !has_foreign_evidence
        || english_proportion >= MIN_DESCRIPTION_ENGLISH_PROPORTION
}
