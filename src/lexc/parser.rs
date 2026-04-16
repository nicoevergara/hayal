use nom::{
    IResult, Parser,
    branch::{alt, permutation},
    bytes::complete::{tag, take_until, take_while1},
    character::{
        MultiSpace0,
        complete::{line_ending, multispace0, not_line_ending, space0, space1},
    },
    combinator::{opt, recognize, rest, verify},
    error::{ErrorKind, make_error},
    multi::{fold_many1, many0},
    sequence::{delimited, preceded, separated_pair, terminated},
};

use std::fs;

const LEXICON_KEYWORD: &str = "LEXICON";
const MULTICHAR_SYMBOLS_KEYWORD: &str = "Multichar_Symbols";
const END_OF_WORD_MARKER_STR: &str = "#";
const END_OF_WORD_MARKER_CHAR: char = '#';
const COMMENT_MARKER_STR: &str = "!";
const COMMENT_MARKER_CHAR: char = '!';
const LEXICON_RULE_ENTRY_SEPARATOR_STR: &str = ";";
const LEXICON_RULE_ENTRY_SEPARATOR_CHAR: char = ';';
const LEXICON_RULE_ENTRY_FORM_SEPARATOR_STR: &str = ":";
const LEXICON_RULE_ENTRY_FORM_SEPARATOR_CHAR: char = ':';

pub struct LexcParser {}

impl LexcParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, raw_text: &str) -> Result<LexcTokens, LexcError> {
        let sanitized_text = raw_text.trim();
        let tokens = self.tokenize(sanitized_text);
        tokens
    }

    fn tokenize(&self, text: &str) -> Result<LexcTokens, LexcError> {
        let mut lexc_tokens = LexcTokens::new();
        let result = parse_multichar_symbols_section(text);
        if let Err(error) = result {
            dbg!(error);
            return Err(LexcError::ParsingError);
        }
        let (remaining, multichar_symbols_tokens) = match result {
            Ok(value) => value,
            Err(_) => return Err(LexcError::ParsingError),
        };
        lexc_tokens.multichar_symbols = multichar_symbols_tokens;

        return match parse_lexicon_section(remaining) {
            Ok((_, lexicon_blocks)) => {
                lexc_tokens.lexicon_blocks = lexicon_blocks;
                Ok(lexc_tokens)
            }
            Err(_) => Err(LexcError::ParsingError),
        };
    }
}

#[derive(Debug)]
pub enum LexcError {
    ParsingError,
}

impl<'a> nom::error::ParseError<&'a str> for LexcError {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        LexcError::ParsingError
    }

    fn append(input: &'a str, kind: nom::error::ErrorKind, sub: Self) -> Self {
        sub
    }
}

#[derive(Debug)]
pub struct LexcTokens {
    multichar_symbols: Vec<MultiCharSymbol>,
    lexicon_blocks: Vec<LexiconBlock>,
}

impl LexcTokens {
    pub fn new() -> Self {
        Self {
            multichar_symbols: Vec::new(),
            lexicon_blocks: Vec::new(),
        }
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
struct LexiconEntry {
    surface_form: Option<String>,
    analysis_form: Option<String>,
    continuation_class: String,
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

    fn surface_form(mut self, surface_form: Option<&str>) -> Self {
        self.surface_form = surface_form.map(|s| s.to_string());
        self
    }

    fn analysis_form(mut self, analysis_form: Option<&str>) -> Self {
        self.analysis_form = analysis_form.map(|s| s.to_string());
        self
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

fn parse_identifier<'a>(text: &'a str) -> IResult<&'a str, &'a str> {
    let (remaining, ident) = take_while1(|c: char| {
        !c.is_control()
            && !c.is_whitespace()
            && c != COMMENT_MARKER_CHAR
            && c != END_OF_WORD_MARKER_CHAR
            && c != LEXICON_RULE_ENTRY_SEPARATOR_CHAR
            && c != LEXICON_RULE_ENTRY_FORM_SEPARATOR_CHAR
    })(text)?;
    Ok((remaining, ident))
}

fn parse_comment<'a>(text: &'a str) -> IResult<&'a str, &'a str> {
    let (remaining, (_, comment)) = (
        multispace0,
        terminated(
            recognize((tag(COMMENT_MARKER_STR), not_line_ending)),
            many0(line_ending),
        ),
    )
        .parse(text)?;
    Ok((remaining, comment))
}

fn parse_trailing_whitespace_and_newlines<'a>(text: &'a str) -> IResult<&'a str, ()> {
    let (remaining, _) = (multispace0, many0(line_ending)).parse(text)?;
    Ok((remaining, ()))
}

fn parse_reserved_keyword<'a>(text: &'a str) -> IResult<&'a str, &'a str> {
    alt((
        tag(LEXICON_KEYWORD),
        tag(MULTICHAR_SYMBOLS_KEYWORD),
        tag(END_OF_WORD_MARKER_STR),
        tag(COMMENT_MARKER_STR),
    ))
    .parse(text)
}

fn parse_multichar_symbols_keyword<'a>(text: &'a str) -> IResult<&'a str, ()> {
    let (remaining, _) = terminated(
        tag(MULTICHAR_SYMBOLS_KEYWORD),
        permutation((
            opt(parse_comment),
            opt(parse_trailing_whitespace_and_newlines),
        )),
    )
    .parse(text)?;
    Ok((remaining, ()))
}

fn parse_multichar_symbols_line<'a>(text: &'a str) -> IResult<&'a str, MultiCharSymbol> {
    let (remaining, multichar_symbol) = verify(
        terminated(
            preceded(space0, parse_identifier),
            preceded(space0, alt((parse_comment, line_ending))),
        ),
        |s: &str| parse_reserved_keyword(s).is_err(),
    )
    .parse(text)?;

    Ok((
        remaining,
        MultiCharSymbol::new(multichar_symbol.to_string()),
    ))
}

fn parse_multichar_symbols_section<'a>(text: &'a str) -> IResult<&'a str, Vec<MultiCharSymbol>> {
    let (remaining, _) =
        preceded(many0(parse_comment), parse_multichar_symbols_keyword).parse(text)?;
    let (lexicon_block, multichar_symbols_block) =
        alt((take_until(LEXICON_KEYWORD), rest)).parse(remaining)?;
    let (_, multichar_symbols) = fold_many1(
        parse_multichar_symbols_line,
        Vec::new,
        |mut acc: Vec<MultiCharSymbol>, symbol: MultiCharSymbol| {
            acc.push(symbol);
            acc
        },
    )
    .parse(multichar_symbols_block)?;

    Ok((lexicon_block, multichar_symbols))
}

fn parse_lexicon_symbols_keyword<'a>(text: &'a str) -> IResult<&'a str, LexiconBlock> {
    let (remaining, result) = delimited(
        terminated(tag(LEXICON_KEYWORD), space1),
        parse_identifier,
        terminated(space0, alt((parse_comment, line_ending))),
    )
    .parse(text)?;
    Ok((remaining, LexiconBlock::new(result.to_string())))
}

fn parse_lexicon_rule_entry_form<'a>(text: &'a str) -> IResult<&'a str, (&'a str, &'a str)> {
    let (remaining, result) = separated_pair(
        parse_identifier,
        tag(LEXICON_RULE_ENTRY_FORM_SEPARATOR_STR),
        parse_identifier,
    )
    .parse(text)?;
    Ok((remaining, result))
}

fn parse_lexicon_rule_entry<'a>(text: &'a str) -> IResult<&'a str, LexiconEntry> {
    let (remaining, lexicon_rule_entry) = preceded(
        space0,
        terminated(
            terminated(
                (
                    opt(parse_lexicon_rule_entry_form),
                    preceded(space0, parse_identifier),
                    opt(preceded(space0, parse_identifier)),
                ),
                preceded(space0, tag(LEXICON_RULE_ENTRY_SEPARATOR_STR)),
            ),
            permutation((
                opt(parse_comment),
                opt(parse_trailing_whitespace_and_newlines),
            )),
        ),
    )
    .parse(text)?;

    match lexicon_rule_entry {
        (Some((surface_form, analysis_form)), continuation_class, None) => Ok((
            remaining,
            LexiconEntry::new(continuation_class.to_string())
                .surface_form(Some(surface_form))
                .analysis_form(Some(analysis_form)),
        )),
        (None, continuation_class, None) => {
            Ok((remaining, LexiconEntry::new(continuation_class.to_string())))
        }
        (None, surface_form, Some(continuation_class)) => Ok((
            remaining,
            LexiconEntry::new(continuation_class.to_string()).surface_form(Some(surface_form)),
        )),
        _ => Err(nom::Err::Error(make_error(text, ErrorKind::Verify))),
    }
}

fn parse_lexicon_block(text: &str) -> IResult<&str, LexiconBlock> {
    if text.trim().is_empty() {
        return Err(nom::Err::Error(make_error(text, ErrorKind::Eof)));
    }
    let (remaining, mut lexicon_block) = parse_lexicon_symbols_keyword(text)?;

    // TODO: switch out fold_many1 with many1
    let (remaining, lexicon_entries) = fold_many1(
        parse_lexicon_rule_entry,
        Vec::new,
        |mut acc: Vec<LexiconEntry>, entry: LexiconEntry| {
            acc.push(entry);
            acc
        },
    )
    .parse(remaining)?;

    lexicon_block = lexicon_entries
        .into_iter()
        .fold(lexicon_block, |block, entry| block.add_lexicon_entry(entry));

    Ok((remaining, lexicon_block))
}

fn parse_lexicon_section<'a>(text: &'a str) -> IResult<&'a str, Vec<LexiconBlock>> {
    terminated(
        fold_many1(
            parse_lexicon_block,
            Vec::new,
            |mut acc: Vec<LexiconBlock>, block: LexiconBlock| {
                acc.push(block);
                acc
            },
        ),
        space0,
    )
    .parse(text)
}

#[cfg(test)]
mod tests {
    use nom::Input;

    use super::*;

    #[test]
    fn test_parse_multichar_symbols_keyword() {
        let trimmed_input = "Multichar_Symbols \n";
        let result = parse_multichar_symbols_keyword(trimmed_input);
        assert!(
            result.is_ok(),
            "Failed to parse Multichar_Symbols: {:?}",
            result
        );

        let (remaining, _) = result.unwrap();
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_parse_multichar_symbols_keyword_with_trailing_comment() {
        let input = "Multichar_Symbols ! this is a comment";
        let result = parse_multichar_symbols_keyword(input);
        assert!(
            result.is_ok(),
            "Failed to parse Multichar_Symbols with trailing comment: {:?}",
            result
        );
        let (remaining, _) = result.unwrap();
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_parse_multichar_symbols_keyword_with_invalid_input() {
        let input = "   Multichar_Symbols ";
        let result = parse_multichar_symbols_keyword(input);
        assert!(
            result.is_err(),
            "Successfully parsed Multichar_Symbols: {:?}",
            result
        );
        assert_eq!(
            result.unwrap_err(),
            nom::Err::Error(nom::error::Error::new(input, ErrorKind::Tag))
        );
    }

    #[test]
    fn test_parse_multichar_symbols_with_valid_comment() {
        let input = " ! this is an invalid comment\n\n ";
        let result = parse_comment(input);
        assert!(
            result.is_ok(),
            "Failed to parse Multichar_Symbols with valid comment: {:?}",
            result
        );
        assert_eq!(result.unwrap().0, " ");
    }

    #[test]
    fn test_parse_multichar_symbols_line() {
        let input = "  +N ! this is a symbol for a Noun";
        let result = parse_multichar_symbols_line(input);
        assert!(
            result.is_ok(),
            "Failed to parse Multichar_Symbols line: {:?}",
            result
        );
        let (remaining, tokens) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(tokens, MultiCharSymbol::new("+N".to_string()));
    }

    #[test]
    fn test_parse_multichar_symbols_line_with_invalid_keyword() {
        let input = "  +N something ! this is a symbol for a Noun";
        let result = parse_multichar_symbols_line(input);
        assert!(
            result.is_err(),
            "Successfully parsed Multichar_Symbols line with invalid keyword: {:?}",
            result
        );
        assert_eq!(
            result.unwrap_err(),
            nom::Err::Error(nom::error::Error::new(
                "something ! this is a symbol for a Noun",
                ErrorKind::CrLf
            ))
        )
    }

    #[test]
    fn test_parse_multichar_symbols_section() {
        let input = "! comments at the top of the document\n\nMultichar_Symbols \n\t +N ! this is a symbol for a Noun\n +V ! this is a symbol for a Verb \n\n\tLEXICON Root";
        let result = parse_multichar_symbols_section(input);
        assert!(
            result.is_ok(),
            "Failed to parse Multichar_Symbols section: {:?}",
            result
        );
        let (remaining, tokens) = result.unwrap();
        assert_eq!(remaining, "LEXICON Root");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], MultiCharSymbol::new("+N".to_string()));
        assert_eq!(tokens[1], MultiCharSymbol::new("+V".to_string()));
    }

    #[test]
    fn test_parse_lexicon_symbols_keyword() {
        let input = "LEXICON Root ! this is a comment\n\t Noun ;";
        let result = parse_lexicon_symbols_keyword(input);
        assert!(
            result.is_ok(),
            "Failed to parse Lexicon entry: {:?}",
            result
        );
        let (remaining, token) = result.unwrap();
        assert_eq!(remaining, "\t Noun ;");
        assert_eq!(token, LexiconBlock::new("Root".to_string()));
    }

    #[test]
    fn test_parse_lexicon_rule_entry() {
        let input = "\t Noun ;";
        let result = parse_lexicon_rule_entry(input);
        assert!(
            result.is_ok(),
            "Failed to parse Lexicon entry: {:?}",
            result
        );
        let (remaining, token) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(token, LexiconEntry::new("Noun".to_string()));

        let input = "\t Noun ; ! this is a comment";
        let result = parse_lexicon_rule_entry(input);
        assert!(
            result.is_ok(),
            "Failed to parse Lexicon entry with comment: {:?}",
            result
        );
        let (remaining, token) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(token, LexiconEntry::new("Noun".to_string()));

        let input = "ev Noun ;";
        let result = parse_lexicon_rule_entry(input);
        assert!(
            result.is_ok(),
            "Failed to parse Lexicon block: {:?}",
            result
        );
        let (remaining, token) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(
            token,
            LexiconEntry::new("Noun".to_string()).surface_form(Some("ev"))
        );

        let input = "+Pl:lAr Noun ;";
        let result = parse_lexicon_rule_entry(input);
        assert!(
            result.is_ok(),
            "Failed to parse Lexicon block: {:?}",
            result
        );
        let (remaining, token) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(
            token,
            LexiconEntry::new("Noun".to_string())
                .surface_form(Some("+Pl"))
                .analysis_form(Some("lAr"))
        );
    }

    #[test]
    fn test_parse_lexicon_block() {
        let input =
            "LEXICON Root ! this is a comment\n\t Noun ;\n adjective; ! this is for an adjective";
        let result = parse_lexicon_block(input);
        assert!(
            result.is_ok(),
            "Failed to parse Lexicon block: {:?}",
            result
        );
        let (remaining, lexicon_block) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(
            lexicon_block,
            LexiconBlock::new("Root".to_string())
                .add_lexicon_entry(LexiconEntry::new("Noun".to_string()))
                .add_lexicon_entry(LexiconEntry::new("adjective".to_string()))
        );
    }

    #[test]
    fn test_parse_lexicon_rule_entry_form() {
        let input = "+N:0";
        let result = parse_lexicon_rule_entry_form(input);
        assert!(
            result.is_ok(),
            "Failed to parse Lexicon rule entry: {:?}",
            result
        );
        let (remaining, token) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(token, ("+N", "0"));
    }

    #[test]
    fn test_parse_lexicon_section() {
        let input = "LEXICON Root ! this is a comment\n\t Noun ;\n adjective; ! this is for an adjective\nLEXICON Noun\n ev       NounStem ;";
        let result = parse_lexicon_section(input);
        assert!(
            result.is_ok(),
            "Failed to parse Lexicon section: {:?}",
            result
        );
        let (remaining, lexicon_block) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(
            lexicon_block[0],
            LexiconBlock::new("Root".to_string())
                .add_lexicon_entry(LexiconEntry::new("Noun".to_string()))
                .add_lexicon_entry(LexiconEntry::new("adjective".to_string()))
        );
        assert_eq!(
            lexicon_block[1],
            LexiconBlock::new("Noun".to_string()).add_lexicon_entry(
                LexiconEntry::new("NounStem".to_string()).surface_form(Some("ev"))
            )
        );
    }

    #[test]
    fn test_parse_multichar_symbols_with_full_file() {
        let input = fs::read_to_string("tests/fixtures/turkish_nouns.lexc")
            .expect("Failed to read turkish_nouns.lexc");
        let parser = LexcParser::new();
        let result = parser.parse(&input);
        assert!(result.is_ok(), "Failed to parse file: {:?}", result);

        let lexc_tokens = result.unwrap();

        // Multichar_Symbols
        assert_eq!(
            lexc_tokens.multichar_symbols,
            vec![
                MultiCharSymbol::new("+N".to_string()),
                MultiCharSymbol::new("+Pl".to_string()),
                MultiCharSymbol::new("+Sg".to_string()),
                MultiCharSymbol::new("+Nom".to_string())
            ]
        );

        assert_eq!(
            lexc_tokens.lexicon_blocks,
            vec![
                // LEXICON Root
                LexiconBlock::new("Root".to_string())
                    .add_lexicon_entry(LexiconEntry::new("Noun".to_string())),
                // LEXICON Noun
                LexiconBlock::new("Noun".to_string())
                    .add_lexicon_entry(
                        LexiconEntry::new("NounStem".to_string()).surface_form(Some("ev"))
                    )
                    .add_lexicon_entry(
                        LexiconEntry::new("NounStem".to_string()).surface_form(Some("kitap"))
                    )
                    .add_lexicon_entry(
                        LexiconEntry::new("NounStem".to_string()).surface_form(Some("göz"))
                    )
                    .add_lexicon_entry(
                        LexiconEntry::new("NounStem".to_string()).surface_form(Some("araba"))
                    )
                    .add_lexicon_entry(
                        LexiconEntry::new("NounStem".to_string()).surface_form(Some("fil"))
                    )
                    .add_lexicon_entry(
                        LexiconEntry::new("NounStem".to_string()).surface_form(Some("köy"))
                    )
                    .add_lexicon_entry(
                        LexiconEntry::new("NounStem".to_string()).surface_form(Some("odun"))
                    )
                    .add_lexicon_entry(
                        LexiconEntry::new("NounStem".to_string()).surface_form(Some("ırk"))
                    )
                    .add_lexicon_entry(
                        LexiconEntry::new("NounStem".to_string()).surface_form(Some("göğüs"))
                    ),
                // LEXICON NounStem
                LexiconBlock::new("NounStem".to_string()).add_lexicon_entry(
                    LexiconEntry::new("Number".to_string())
                        .surface_form(Some("+N"))
                        .analysis_form(Some("0"))
                ),
                // LEXICON Number
                LexiconBlock::new("Number".to_string())
                    .add_lexicon_entry(
                        LexiconEntry::new("Case".to_string())
                            .surface_form(Some("+Sg"))
                            .analysis_form(Some("0"))
                    )
                    .add_lexicon_entry(
                        LexiconEntry::new("Case".to_string())
                            .surface_form(Some("+Pl"))
                            .analysis_form(Some("lAr"))
                    ),
            ]
        );
    }
}
