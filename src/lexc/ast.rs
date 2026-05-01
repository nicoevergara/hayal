use std::fmt;

pub const LEXICON_KEYWORD: &str = "LEXICON";
pub const MULTICHAR_SYMBOLS_KEYWORD: &str = "Multichar_Symbols";
pub const END_OF_WORD_MARKER_STR: &str = "#";
pub const END_OF_WORD_MARKER_CHAR: char = '#';
pub const COMMENT_MARKER_STR: &str = "!";
pub const COMMENT_MARKER_CHAR: char = '!';
pub const LEXICON_RULE_ENTRY_SEPARATOR_STR: &str = ";";
pub const LEXICON_RULE_ENTRY_SEPARATOR_CHAR: char = ';';
pub const LEXICON_RULE_ENTRY_FORM_SEPARATOR_STR: &str = ":";
pub const LEXICON_RULE_ENTRY_FORM_SEPARATOR_CHAR: char = ':';

#[derive(Debug, PartialEq)]
pub struct LexcFile {
    multichar_symbols: Vec<MultiCharSymbol>,
    lexicon_blocks: Vec<LexiconBlock>,
}

impl LexcFile {
    /// Creates a new LexcFile
    /// # Returns
    ///
    /// Returns an empty `LexcFile`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hayal::lexc::ast::LexcFile;
    ///
    /// let lexc_file = LexcFile::new();
    /// ```
    pub fn new() -> Self {
        Self {
            multichar_symbols: Vec::new(),
            lexicon_blocks: Vec::new(),
        }
    }

    /// Adds a `MultiCharSymbol` to the `LexcFile`.
    pub fn add_multichar_symbol(mut self, multichar_symbol: MultiCharSymbol) -> Self {
        self.multichar_symbols.push(multichar_symbol);
        self
    }

    /// Adds a `LexiconBlock` to the `LexcFile`.
    pub fn add_lexicon_block(mut self, lexicon_block: LexiconBlock) -> Self {
        self.lexicon_blocks.push(lexicon_block);
        self
    }

    pub fn validate(&self) -> bool {
        true
    }
}

// Represents a single lexicon entry following the pattern:
// [optional_analysis_form[:optional_surface_form]] continuation_class ;
//
// The colon separates the analysis (upper) side from the surface (lower) side:
//   - analysis_form: what appears on the analysis side, e.g. +Pl (the morphological tag)
//   - surface_form:  what appears on the surface side, e.g. lAr (the actual string)
//
// Examples:
//   Noun ;               — no form, just a continuation class
//   ev NounStem ;        — analysis and surface are the same (no colon needed)
//   +Pl:lAr Case ;       — +Pl on analysis side, lAr on surface side
//   ev NounStem ; ! house — with optional trailing comment
#[derive(Debug, PartialEq, Clone)]
pub struct LexiconEntry {
    continuation_class: String,
    surface_form: Option<String>,
    analysis_form: Option<String>,
    comment: Option<String>,
}

impl LexiconEntry {
    pub fn new(continuation_class: String) -> Self {
        Self {
            surface_form: None,
            analysis_form: None,
            continuation_class,
            comment: None,
        }
    }

    pub fn surface_form(mut self, surface_form: Option<&str>) -> Self {
        self.surface_form = surface_form.map(|s| s.to_string());
        self
    }

    pub fn analysis_form(mut self, analysis_form: Option<&str>) -> Self {
        self.analysis_form = analysis_form.map(|s| s.to_string());
        self
    }
}

impl fmt::Display for LexiconEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(analysis_form) = &self.analysis_form {
            writeln!(f, "Analysis Form: {}", analysis_form);
        }
        if let Some(surface_form) = &self.surface_form {
            writeln!(f, "Surface Form: {}", surface_form);
        }
        writeln!(f, "Continuation Class: {}", self.continuation_class)
    }
}

#[derive(Debug, PartialEq)]
pub struct MultiCharSymbol {
    symbol: String,
    comment: Option<LexcComment>,
}

impl MultiCharSymbol {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            comment: None,
        }
    }

    pub fn add_comment(mut self, comment: Option<LexcComment>) -> Self {
        self.comment = comment;
        self
    }
}

impl fmt::Display for MultiCharSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

#[derive(Debug, PartialEq)]
pub struct LexcComment(String);

#[derive(Debug, PartialEq)]
pub struct LexiconBlock {
    pub name: String,
    pub entries: Vec<LexiconEntry>,
    pub comment: Option<String>,
}

impl LexiconBlock {
    pub fn new(name: String) -> Self {
        Self {
            name,
            entries: Vec::new(),
            comment: None,
        }
    }

    pub fn add_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn add_lexicon_entry(mut self, entry: LexiconEntry) -> Self {
        self.entries.push(entry);
        self
    }
}

impl fmt::Display for LexiconBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "LEXICON {}", self.name);
        for entry in &self.entries {
            writeln!(f, "{}", entry);
        }
        Ok(())
    }
}

impl fmt::Display for LexcFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "LexcFile");
        writeln!(f, "MultiChar Symbols");
        for symbol in &self.multichar_symbols {
            writeln!(f, "{}", symbol);
        }
        writeln!(f, "\nLexicon Blocks");
        for lexicon_block in &self.lexicon_blocks {
            writeln!(f, "{}", lexicon_block);
        }
        Ok(())
    }
}

// TODO: add tests
#[cfg(test)]
mod tests {
    use super::*;
}
