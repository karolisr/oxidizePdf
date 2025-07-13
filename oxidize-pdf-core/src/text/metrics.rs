use crate::text::Font;
use std::collections::HashMap;

/// Character width information for standard PDF fonts
/// All widths are in 1/1000 of a unit (font size 1.0)
pub struct FontMetrics {
    widths: HashMap<char, u16>,
    default_width: u16,
}

impl FontMetrics {
    fn new(default_width: u16) -> Self {
        Self {
            widths: HashMap::new(),
            default_width,
        }
    }

    fn with_widths(mut self, widths: &[(char, u16)]) -> Self {
        for &(ch, width) in widths {
            self.widths.insert(ch, width);
        }
        self
    }

    pub fn char_width(&self, ch: char) -> u16 {
        self.widths.get(&ch).copied().unwrap_or(self.default_width)
    }
}

lazy_static::lazy_static! {
    static ref FONT_METRICS: HashMap<Font, FontMetrics> = {
        let mut metrics = HashMap::new();

        // Helvetica
        metrics.insert(Font::Helvetica, FontMetrics::new(556).with_widths(&[
            (' ', 278), ('!', 278), ('"', 355), ('#', 556), ('$', 556), ('%', 889),
            ('&', 667), ('\'', 191), ('(', 333), (')', 333), ('*', 389), ('+', 584),
            (',', 278), ('-', 333), ('.', 278), ('/', 278), ('0', 556), ('1', 556),
            ('2', 556), ('3', 556), ('4', 556), ('5', 556), ('6', 556), ('7', 556),
            ('8', 556), ('9', 556), (':', 278), (';', 278), ('<', 584), ('=', 584),
            ('>', 584), ('?', 556), ('@', 1015), ('A', 667), ('B', 667), ('C', 722),
            ('D', 722), ('E', 667), ('F', 611), ('G', 778), ('H', 722), ('I', 278),
            ('J', 500), ('K', 667), ('L', 556), ('M', 833), ('N', 722), ('O', 778),
            ('P', 667), ('Q', 778), ('R', 722), ('S', 667), ('T', 611), ('U', 722),
            ('V', 667), ('W', 944), ('X', 667), ('Y', 667), ('Z', 611), ('[', 278),
            ('\\', 278), (']', 278), ('^', 469), ('_', 556), ('`', 333), ('a', 556),
            ('b', 556), ('c', 500), ('d', 556), ('e', 556), ('f', 278), ('g', 556),
            ('h', 556), ('i', 222), ('j', 222), ('k', 500), ('l', 222), ('m', 833),
            ('n', 556), ('o', 556), ('p', 556), ('q', 556), ('r', 333), ('s', 500),
            ('t', 278), ('u', 556), ('v', 500), ('w', 722), ('x', 500), ('y', 500),
            ('z', 500), ('{', 334), ('|', 260), ('}', 334), ('~', 584),
        ]));

        // Helvetica Bold
        metrics.insert(Font::HelveticaBold, FontMetrics::new(611).with_widths(&[
            (' ', 278), ('!', 333), ('"', 474), ('#', 556), ('$', 556), ('%', 889),
            ('&', 722), ('\'', 238), ('(', 333), (')', 333), ('*', 389), ('+', 584),
            (',', 278), ('-', 333), ('.', 278), ('/', 278), ('0', 556), ('1', 556),
            ('2', 556), ('3', 556), ('4', 556), ('5', 556), ('6', 556), ('7', 556),
            ('8', 556), ('9', 556), (':', 333), (';', 333), ('<', 584), ('=', 584),
            ('>', 584), ('?', 611), ('@', 975), ('A', 722), ('B', 722), ('C', 722),
            ('D', 722), ('E', 667), ('F', 611), ('G', 778), ('H', 722), ('I', 278),
            ('J', 556), ('K', 722), ('L', 611), ('M', 833), ('N', 722), ('O', 778),
            ('P', 667), ('Q', 778), ('R', 722), ('S', 667), ('T', 611), ('U', 722),
            ('V', 667), ('W', 944), ('X', 667), ('Y', 667), ('Z', 611), ('[', 333),
            ('\\', 278), (']', 333), ('^', 584), ('_', 556), ('`', 333), ('a', 556),
            ('b', 611), ('c', 556), ('d', 611), ('e', 556), ('f', 333), ('g', 611),
            ('h', 611), ('i', 278), ('j', 278), ('k', 556), ('l', 278), ('m', 889),
            ('n', 611), ('o', 611), ('p', 611), ('q', 611), ('r', 389), ('s', 556),
            ('t', 333), ('u', 611), ('v', 556), ('w', 778), ('x', 556), ('y', 556),
            ('z', 500), ('{', 389), ('|', 280), ('}', 389), ('~', 584),
        ]));

        // Times Roman
        metrics.insert(Font::TimesRoman, FontMetrics::new(500).with_widths(&[
            (' ', 250), ('!', 333), ('"', 408), ('#', 500), ('$', 500), ('%', 833),
            ('&', 778), ('\'', 180), ('(', 333), (')', 333), ('*', 500), ('+', 564),
            (',', 250), ('-', 333), ('.', 250), ('/', 278), ('0', 500), ('1', 500),
            ('2', 500), ('3', 500), ('4', 500), ('5', 500), ('6', 500), ('7', 500),
            ('8', 500), ('9', 500), (':', 278), (';', 278), ('<', 564), ('=', 564),
            ('>', 564), ('?', 444), ('@', 921), ('A', 722), ('B', 667), ('C', 667),
            ('D', 722), ('E', 611), ('F', 556), ('G', 722), ('H', 722), ('I', 333),
            ('J', 389), ('K', 722), ('L', 611), ('M', 889), ('N', 722), ('O', 722),
            ('P', 556), ('Q', 722), ('R', 667), ('S', 556), ('T', 611), ('U', 722),
            ('V', 722), ('W', 944), ('X', 722), ('Y', 722), ('Z', 611), ('[', 333),
            ('\\', 278), (']', 333), ('^', 469), ('_', 500), ('`', 333), ('a', 444),
            ('b', 500), ('c', 444), ('d', 500), ('e', 444), ('f', 333), ('g', 500),
            ('h', 500), ('i', 278), ('j', 278), ('k', 500), ('l', 278), ('m', 778),
            ('n', 500), ('o', 500), ('p', 500), ('q', 500), ('r', 333), ('s', 389),
            ('t', 278), ('u', 500), ('v', 500), ('w', 722), ('x', 500), ('y', 500),
            ('z', 444), ('{', 480), ('|', 200), ('}', 480), ('~', 541),
        ]));

        // Courier (all characters have the same width)
        metrics.insert(Font::Courier, FontMetrics::new(600).with_widths(&[
            (' ', 600), ('!', 600), ('"', 600), ('#', 600), ('$', 600), ('%', 600),
            ('&', 600), ('\'', 600), ('(', 600), (')', 600), ('*', 600), ('+', 600),
            (',', 600), ('-', 600), ('.', 600), ('/', 600), ('0', 600), ('1', 600),
            ('2', 600), ('3', 600), ('4', 600), ('5', 600), ('6', 600), ('7', 600),
            ('8', 600), ('9', 600), (':', 600), (';', 600), ('<', 600), ('=', 600),
            ('>', 600), ('?', 600), ('@', 600), ('A', 600), ('B', 600), ('C', 600),
            ('D', 600), ('E', 600), ('F', 600), ('G', 600), ('H', 600), ('I', 600),
            ('J', 600), ('K', 600), ('L', 600), ('M', 600), ('N', 600), ('O', 600),
            ('P', 600), ('Q', 600), ('R', 600), ('S', 600), ('T', 600), ('U', 600),
            ('V', 600), ('W', 600), ('X', 600), ('Y', 600), ('Z', 600), ('[', 600),
            ('\\', 600), (']', 600), ('^', 600), ('_', 600), ('`', 600), ('a', 600),
            ('b', 600), ('c', 600), ('d', 600), ('e', 600), ('f', 600), ('g', 600),
            ('h', 600), ('i', 600), ('j', 600), ('k', 600), ('l', 600), ('m', 600),
            ('n', 600), ('o', 600), ('p', 600), ('q', 600), ('r', 600), ('s', 600),
            ('t', 600), ('u', 600), ('v', 600), ('w', 600), ('x', 600), ('y', 600),
            ('z', 600), ('{', 600), ('|', 600), ('}', 600), ('~', 600),
        ]));

        // For now, use the same metrics for variations
        metrics.insert(Font::HelveticaOblique, metrics[&Font::Helvetica].clone());
        metrics.insert(Font::HelveticaBoldOblique, metrics[&Font::HelveticaBold].clone());
        metrics.insert(Font::TimesBold, metrics[&Font::TimesRoman].clone());
        metrics.insert(Font::TimesItalic, metrics[&Font::TimesRoman].clone());
        metrics.insert(Font::TimesBoldItalic, metrics[&Font::TimesRoman].clone());
        metrics.insert(Font::CourierBold, metrics[&Font::Courier].clone());
        metrics.insert(Font::CourierOblique, metrics[&Font::Courier].clone());
        metrics.insert(Font::CourierBoldOblique, metrics[&Font::Courier].clone());

        metrics
    };
}

impl FontMetrics {
    fn clone(&self) -> Self {
        Self {
            widths: self.widths.clone(),
            default_width: self.default_width,
        }
    }
}

/// Measure the width of a text string in a given font and size
pub fn measure_text(text: &str, font: Font, font_size: f64) -> f64 {
    if font.is_symbolic() {
        // Symbol and ZapfDingbats need special handling
        return text.len() as f64 * font_size * 0.6;
    }

    let metrics = FONT_METRICS.get(&font).expect("Font metrics not found");

    let width_units: u32 = text.chars().map(|ch| metrics.char_width(ch) as u32).sum();

    (width_units as f64 / 1000.0) * font_size
}

/// Measure the width of a single character
pub fn measure_char(ch: char, font: Font, font_size: f64) -> f64 {
    if font.is_symbolic() {
        return font_size * 0.6;
    }

    let metrics = FONT_METRICS.get(&font).expect("Font metrics not found");

    (metrics.char_width(ch) as f64 / 1000.0) * font_size
}

/// Split text into words, preserving spaces
pub fn split_into_words(text: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let mut start = 0;
    let mut in_space = false;

    for (i, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if !in_space {
                if i > start {
                    words.push(&text[start..i]);
                }
                start = i;
                in_space = true;
            }
        } else if in_space {
            if i > start {
                words.push(&text[start..i]);
            }
            start = i;
            in_space = false;
        }
    }

    if start < text.len() {
        words.push(&text[start..]);
    }

    words
}
