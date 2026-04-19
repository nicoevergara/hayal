# Lexc Syntax Reference

## Sources

This document is derived from the foma project's morphological analysis tutorial and documentation.

- **Foma project**: [https://fomafst.github.io/](https://fomafst.github.io/)
- **Morphology tutorial**: [https://fomafst.github.io/morphtut.html](https://fomafst.github.io/morphtut.html)
- **Author**: Mans Hulden
- **License**: [Apache License 2.0](https://github.com/mhulden/foma)

This syntax reference documents the lexc formalism as implemented by foma. The lexc format originates from Xerox's finite-state tools and is reproduced here for reference purposes only. No source code from the foma project has been incorporated into this project.

## File Structure

A lexc file consists of two optional sections in order:
1. `Multichar_Symbols` declaration
2. One or more `LEXICON` blocks

## Multichar Symbols

```
Multichar_Symbols +N +V +Pl +Sg +Past
```

- Must appear before any `LEXICON` blocks
- Declares symbol sequences that should be treated as single atomic units
- Space-separated list of symbols

## Lexicon Blocks

```
LEXICON <Name>
<entries>
```

- `LEXICON` keyword followed by the lexicon name
- Every file must have a `LEXICON Root` as the entry point
- A lexicon contains zero or more entries

## Entry Format

```
[analysis_form[:surface_form]] continuation_class ;
```

### Variants

| Form | Example | Meaning |
|------|---------|---------|
| Continuation only | `Noun ;` | Transition to another lexicon with no form |
| Single form | `cat Ninf ;` | Form with continuation class (analysis and surface are identical) |
| Two-level form | `+Pl:s Ninf ;` | Analysis form on left, surface form on right of `:` |
| End of word | `# ;` | Word boundary — terminates a path |

## Special Characters

| Character | Meaning |
|-----------|---------|
| `#` | End-of-word marker |
| `:` | Separates analysis (upper) from surface (lower) side |
| `;` | Terminates an entry |
| `0` | Epsilon (empty string) |
| `!` | Begins a comment (rest of line is ignored) |

## Comments

```
! This is a comment
cat Ninf ; ! This is a trailing comment
```

Comments begin with `!` and run to the end of the line.

## Continuation Classes

Each entry must specify a continuation class — the name of the next lexicon to transition into. This is how valid morpheme sequences are encoded: the structure of which lexicons point to which other lexicons defines the grammar.

## Example

```
Multichar_Symbols +N +Pl +Sg

LEXICON Root
cat   Noun ;
dog   Noun ;

LEXICON Noun
+Sg:0  # ;
+Pl:s  # ;
```
