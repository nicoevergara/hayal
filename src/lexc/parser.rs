use nom::{
    IResult, Parser,
    branch::{alt, permutation},
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{line_ending, multispace0, not_line_ending, space0, space1},
    combinator::{opt, recognize, rest, verify},
    error::{ErrorKind, make_error},
    multi::{fold_many1, many0, many1},
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

    pub fn parse(&self, raw_text: &str) -> Result<Vec<Token>, LexcError> {
        let sanitized_text = raw_text.trim();
        let tokens = self.tokenize(sanitized_text);
        tokens
    }

    fn tokenize(&self, text: &str) -> Result<Vec<Token>, LexcError> {
        let mut tokens = Vec::new();

        let result = parse_multichar_symbols_section(text);
        if result.is_err() {
            return Err(LexcError::ParsingError);
        }
        let (remaining, multichar_symbols_tokens) = result.unwrap();
        tokens.extend(multichar_symbols_tokens);

        let result = parse_lexicon_section(remaining);
        if result.is_err() {
            return Err(LexcError::ParsingError);
        }
        let (_, lexicon_tokens) = result.unwrap();
        tokens.extend(lexicon_tokens);

        Ok(tokens)
    }
}

#[derive(Debug)]
enum LexcError {
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
#[derive(Debug, PartialEq)]
struct LexcRule {
    surface_form: Option<String>,
    analysis_form: Option<String>,
    continuation_class: String,
    comment: Option<String>,
}

impl LexcRule {
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
enum Token {
    MultiCharSymbol(String),
    Lexicon(String),
    Rule(LexcRule),
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

fn parse_multichar_symbols_line<'a>(text: &'a str) -> IResult<&'a str, Token> {
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
        Token::MultiCharSymbol(multichar_symbol.to_string()),
    ))
}

fn parse_multichar_symbols_section<'a>(text: &'a str) -> IResult<&'a str, Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    let (remaining, _) =
        preceded(many0(parse_comment), parse_multichar_symbols_keyword).parse(text)?;
    let (lexicon_block, multichar_symbols_block) =
        alt((take_until(LEXICON_KEYWORD), rest)).parse(remaining)?;
    let (_, multichar_symbol_tokens) = fold_many1(
        parse_multichar_symbols_line,
        Vec::new,
        |mut acc: Vec<Token>, token: Token| {
            acc.push(token);
            acc
        },
    )
    .parse(multichar_symbols_block)?;

    tokens.extend(multichar_symbol_tokens);

    Ok((lexicon_block, tokens))
}

fn parse_lexicon_symbols_keyword<'a>(text: &'a str) -> IResult<&'a str, Token> {
    let (remaining, result) = delimited(
        terminated(tag(LEXICON_KEYWORD), space1),
        parse_identifier,
        terminated(space0, alt((parse_comment, line_ending))),
    )
    .parse(text)?;
    Ok((remaining, Token::Lexicon(result.to_string())))
}

fn parse_lexicon_rule_entry_form<'a>(text: &'a str) -> IResult<&'a str, (&'a str, &'a str)> {
    let (remaining, result) = separated_pair(
        parse_identifier,
        tag(LEXICON_RULE_ENTRY_FORM_SEPARATOR_STR),
        parse_identifier,
    )
    .parse(text)?;
    println!("lexicon entry form - {:?}", result);
    Ok((remaining, result))
}

fn parse_lexicon_rule_entry<'a>(text: &'a str) -> IResult<&'a str, Token> {
    let (remaining, lexicon_rule_entry) = preceded(
        space0,
        terminated(
            terminated(
                (
                    |i| {
                        let (surface_form, analysis_form) =
                            opt(parse_lexicon_rule_entry_form).parse(i)?;
                        println!("lexicon rule entry - step 1");
                        println!("\tinput - {:?}", i);
                        println!("\t\tsurface_form - {:?}", surface_form);
                        println!("\t\tanalysis_form - {:?}", analysis_form);
                        Ok((surface_form, analysis_form))
                    },
                    |i| {
                        let (remaining, ident) = preceded(space0, parse_identifier).parse(i)?;
                        println!("lexicon rule entry - step 2");
                        println!("\tinput - {:?}", i);
                        println!("\t\tident - {:?}", ident);
                        Ok((remaining, ident))
                    },
                    |i| {
                        let (remaining, ident) =
                            opt(preceded(space0, parse_identifier)).parse(i)?;
                        println!("lexicon rule entry - step 3");
                        println!("\tinput - {:?}", i);
                        println!("\t\tident - {:?}", ident);
                        Ok((remaining, ident))
                    },
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
    println!("lexicon rule entry - step 4");
    println!("\tinput - {:?}", text);
    println!("\t\tlexicon_rule_entry - {:?}", lexicon_rule_entry);

    match lexicon_rule_entry {
        (Some((surface_form, analysis_form)), continuation_class, None) => Ok((
            remaining,
            Token::Rule(
                LexcRule::new(continuation_class.to_string())
                    .surface_form(Some(surface_form))
                    .analysis_form(Some(analysis_form)),
            ),
        )),
        (None, continuation_class, None) => Ok((
            remaining,
            Token::Rule(LexcRule::new(continuation_class.to_string())),
        )),
        (None, surface_form, Some(continuation_class)) => Ok((
            remaining,
            Token::Rule(
                LexcRule::new(continuation_class.to_string()).surface_form(Some(surface_form)),
            ),
        )),
        _ => Err(nom::Err::Error(make_error(text, ErrorKind::Verify))),
    }
}

fn parse_lexicon_block(text: &str) -> IResult<&str, Vec<Token>> {
    if text.trim().is_empty() {
        return Err(nom::Err::Error(make_error(text, ErrorKind::Eof)));
    }
    let mut tokens = Vec::new();
    println!("parsing lexicon block {:?} of length {}", text, text.len());
    let (remaining, lexicon_token) = parse_lexicon_symbols_keyword(text)?;
    tokens.push(lexicon_token);

    let (remaining, lexicon_tokens) = fold_many1(
        |i| {
            let (remaining, tokens) = parse_lexicon_rule_entry(i)?;
            println!("tokens {:?}", tokens);
            println!("remaining lex section{:?}", remaining);
            Ok((remaining, tokens))
        },
        Vec::new,
        |mut acc: Vec<Token>, token: Token| {
            acc.push(token);
            acc
        },
    )
    .parse(remaining)?;

    tokens.extend(lexicon_tokens);
    Ok((remaining, tokens))
}

fn parse_lexicon_section<'a>(text: &'a str) -> IResult<&'a str, Vec<Token>> {
    let (remaining, lexicon_tokens) = terminated(
        fold_many1(
            parse_lexicon_block,
            Vec::new,
            |mut acc: Vec<Token>, tokens: Vec<Token>| {
                acc.extend(tokens);
                acc
            },
        ),
        space0,
    )
    .parse(text)?;
    Ok((remaining, lexicon_tokens))
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
        assert_eq!(tokens, Token::MultiCharSymbol("+N".to_string()));
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
        assert_eq!(tokens[0], Token::MultiCharSymbol("+N".to_string()));
        assert_eq!(tokens[1], Token::MultiCharSymbol("+V".to_string()));
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
        assert_eq!(token, Token::Lexicon("Root".to_string()));
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
        assert_eq!(token, Token::Rule(LexcRule::new("Noun".to_string())));

        let input = "\t Noun ; ! this is a comment";
        let result = parse_lexicon_rule_entry(input);
        assert!(
            result.is_ok(),
            "Failed to parse Lexicon entry with comment: {:?}",
            result
        );
        let (remaining, token) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(token, Token::Rule(LexcRule::new("Noun".to_string())));

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
            Token::Rule(LexcRule::new("Noun".to_string()).surface_form(Some("ev")))
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
            Token::Rule(
                LexcRule::new("Noun".to_string())
                    .surface_form(Some("+Pl"))
                    .analysis_form(Some("lAr"))
            )
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
        let (remaining, tokens) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(tokens[0], Token::Lexicon("Root".to_string()));
        assert_eq!(tokens[1], Token::Rule(LexcRule::new("Noun".to_string())));
        assert_eq!(
            tokens[2],
            Token::Rule(LexcRule::new("adjective".to_string()))
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
        let (remaining, tokens) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(tokens[0], Token::Lexicon("Root".to_string()));
        assert_eq!(tokens[1], Token::Rule(LexcRule::new("Noun".to_string())));
        assert_eq!(
            tokens[2],
            Token::Rule(LexcRule::new("adjective".to_string()))
        );
        assert_eq!(tokens[3], Token::Lexicon("Noun".to_string()));
        assert_eq!(
            tokens[4],
            Token::Rule(LexcRule::new("NounStem".to_string()).surface_form(Some("ev")))
        );
    }

    #[test]
    fn test_parse_multichar_symbols_with_full_file() {
        let input = fs::read_to_string("tests/fixtures/turkish_nouns.lexc")
            .expect("Failed to read turkish_nouns.lexc");
        let parser = LexcParser::new();
        let result = parser.parse(&input);
        assert!(result.is_ok(), "Failed to parse file: {:?}", result);

        let tokens = result.unwrap();

        // Multichar_Symbols
        assert_eq!(tokens[0], Token::MultiCharSymbol("+N".to_string()));
        assert_eq!(tokens[1], Token::MultiCharSymbol("+Pl".to_string()));
        assert_eq!(tokens[2], Token::MultiCharSymbol("+Sg".to_string()));
        assert_eq!(tokens[3], Token::MultiCharSymbol("+Nom".to_string()));

        // LEXICON Root
        assert_eq!(tokens[4], Token::Lexicon("Root".to_string()));
        assert_eq!(tokens[5], Token::Rule(LexcRule::new("Noun".to_string())));

        // LEXICON Noun
        assert_eq!(tokens[6], Token::Lexicon("Noun".to_string()));
        assert_eq!(
            tokens[7],
            Token::Rule(LexcRule::new("NounStem".to_string()).surface_form(Some("ev")))
        );
        assert_eq!(
            tokens[8],
            Token::Rule(LexcRule::new("NounStem".to_string()).surface_form(Some("kitap")))
        );
        assert_eq!(
            tokens[9],
            Token::Rule(LexcRule::new("NounStem".to_string()).surface_form(Some("göz")))
        );
        assert_eq!(
            tokens[10],
            Token::Rule(LexcRule::new("NounStem".to_string()).surface_form(Some("araba")))
        );
        assert_eq!(
            tokens[11],
            Token::Rule(LexcRule::new("NounStem".to_string()).surface_form(Some("fil")))
        );
        assert_eq!(
            tokens[12],
            Token::Rule(LexcRule::new("NounStem".to_string()).surface_form(Some("köy")))
        );
        assert_eq!(
            tokens[13],
            Token::Rule(LexcRule::new("NounStem".to_string()).surface_form(Some("odun")))
        );
        assert_eq!(
            tokens[14],
            Token::Rule(LexcRule::new("NounStem".to_string()).surface_form(Some("ırk")))
        );
        assert_eq!(
            tokens[15],
            Token::Rule(LexcRule::new("NounStem".to_string()).surface_form(Some("göğüs")))
        );

        // LEXICON NounStem
        assert_eq!(tokens[16], Token::Lexicon("NounStem".to_string()));
        assert_eq!(
            tokens[17],
            Token::Rule(
                LexcRule::new("Number".to_string())
                    .surface_form(Some("+N"))
                    .analysis_form(Some("0"))
            )
        );

        // LEXICON Number
        assert_eq!(tokens[18], Token::Lexicon("Number".to_string()));
        assert_eq!(
            tokens[19],
            Token::Rule(
                LexcRule::new("Case".to_string())
                    .surface_form(Some("+Sg"))
                    .analysis_form(Some("0"))
            )
        );
        assert_eq!(
            tokens[20],
            Token::Rule(
                LexcRule::new("Case".to_string())
                    .surface_form(Some("+Pl"))
                    .analysis_form(Some("lAr"))
            )
        );

        assert_eq!(tokens.len(), 21);
    }
}
