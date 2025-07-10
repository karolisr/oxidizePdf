/// Standard PDF fonts (Type 1 base 14 fonts).
///
/// These fonts are guaranteed to be available in all PDF readers
/// and don't need to be embedded in the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Font {
    // Standard 14 PDF fonts
    /// Helvetica (sans-serif)
    Helvetica,
    /// Helvetica Bold
    HelveticaBold,
    /// Helvetica Oblique (italic)
    HelveticaOblique,
    /// Helvetica Bold Oblique
    HelveticaBoldOblique,
    /// Times Roman (serif)
    TimesRoman,
    /// Times Bold
    TimesBold,
    /// Times Italic
    TimesItalic,
    /// Times Bold Italic
    TimesBoldItalic,
    /// Courier (monospace)
    Courier,
    /// Courier Bold
    CourierBold,
    /// Courier Oblique
    CourierOblique,
    /// Courier Bold Oblique
    CourierBoldOblique,
    /// Symbol font (mathematical symbols)
    Symbol,
    /// ZapfDingbats (decorative symbols)
    ZapfDingbats,
}

impl Font {
    pub fn pdf_name(&self) -> &'static str {
        match self {
            Font::Helvetica => "Helvetica",
            Font::HelveticaBold => "Helvetica-Bold",
            Font::HelveticaOblique => "Helvetica-Oblique",
            Font::HelveticaBoldOblique => "Helvetica-BoldOblique",
            Font::TimesRoman => "Times-Roman",
            Font::TimesBold => "Times-Bold",
            Font::TimesItalic => "Times-Italic",
            Font::TimesBoldItalic => "Times-BoldItalic",
            Font::Courier => "Courier",
            Font::CourierBold => "Courier-Bold",
            Font::CourierOblique => "Courier-Oblique",
            Font::CourierBoldOblique => "Courier-BoldOblique",
            Font::Symbol => "Symbol",
            Font::ZapfDingbats => "ZapfDingbats",
        }
    }

    pub fn is_symbolic(&self) -> bool {
        matches!(self, Font::Symbol | Font::ZapfDingbats)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontFamily {
    Helvetica,
    Times,
    Courier,
}

impl FontFamily {
    pub fn regular(self) -> Font {
        match self {
            FontFamily::Helvetica => Font::Helvetica,
            FontFamily::Times => Font::TimesRoman,
            FontFamily::Courier => Font::Courier,
        }
    }

    pub fn bold(self) -> Font {
        match self {
            FontFamily::Helvetica => Font::HelveticaBold,
            FontFamily::Times => Font::TimesBold,
            FontFamily::Courier => Font::CourierBold,
        }
    }

    pub fn italic(self) -> Font {
        match self {
            FontFamily::Helvetica => Font::HelveticaOblique,
            FontFamily::Times => Font::TimesItalic,
            FontFamily::Courier => Font::CourierOblique,
        }
    }

    pub fn bold_italic(self) -> Font {
        match self {
            FontFamily::Helvetica => Font::HelveticaBoldOblique,
            FontFamily::Times => Font::TimesBoldItalic,
            FontFamily::Courier => Font::CourierBoldOblique,
        }
    }
}
