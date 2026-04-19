# Foma/Xfst Syntax Reference

## Sources

This document is derived from the foma project's official documentation and reference materials.

- **Foma project**: [https://fomafst.github.io/](https://fomafst.github.io/)
- **Regular expression reference**: [https://fomafst.github.io/regexreference.html](https://fomafst.github.io/regexreference.html)
- **Getting started guide**: [https://github.com/mhulden/foma/blob/master/foma/docs/simpleintro.md](https://github.com/mhulden/foma/blob/master/foma/docs/simpleintro.md)
- **Author**: Mans Hulden
- **License**: [Apache License 2.0](https://github.com/mhulden/foma)

This syntax reference documents the foma/xfst scripting language as implemented by foma. The xfst syntax originates from Xerox's finite-state tools and is reproduced here for reference purposes only. No source code from the foma project has been incorporated into this project.

## Script Commands

| Command | Description |
|---------|-------------|
| `regex <expr> ;` | Compiles a regular expression into an automaton/transducer |
| `def <Name> <expr> ;` | Defines a named regular language constant |
| `read lexc <file>` | Reads and compiles a lexc file |
| `source <file>` | Loads and executes a foma script file |
| `print defined` | Lists all defined language constants |
| `view` | Displays automaton graphically |
| `net` | ASCII listing of transitions |
| `sigma` | Shows transducer alphabet |
| `words` | Lists all accepted words |
| `pairs` | Displays input/output pairs |
| `upper-words` | Shows input-side set |
| `lower-words` | Shows output-side set |
| `down` | Interactive input testing |
| `up` | Interactive output testing (inverse) |

## Regular Expression Operators

### Basic

| Operator | Description |
|----------|-------------|
| `X Y` | Concatenation |
| `X \| Y` | Union |
| `X & Y` | Intersection |
| `X - Y` | Difference |
| `~X` | Complement |
| `[X]` | Grouping |
| `(X)` | Optionality (zero or one) |
| `?` | Any single symbol |

### Repetition

| Operator | Description |
|----------|-------------|
| `X*` | Zero or more |
| `X+` | One or more |
| `X?` | Zero or one |
| `X^n` | Exactly n |
| `X^>n` | More than n |
| `X^<n` | Less than n |
| `X^{m,n}` | Between m and n |

### Transducer Operations

| Operator | Description |
|----------|-------------|
| `X:Y` | Cross product (maps X to Y) |
| `X .o. Y` | Composition |
| `X.u` | Domain (upper/input side) |
| `X.l` | Range (lower/output side) |
| `X.i` | Inversion |

### Containment

| Operator | Description |
|----------|-------------|
| `$X` | Contains substring X |
| `$.X` | Contains exactly one X |
| `$?X` | Contains at most one X |

### Quotient

| Operator | Description |
|----------|-------------|
| `X/Y` | Right quotient / ignore Y inside X |
| `X\\Y` | Left quotient |
| `X./.Y` | Ignore Y inside X |

## Rewrite Rules

```
A -> B || L _ R ;
```

- `A` — input pattern to match
- `B` — output replacement
- `L _ R` — left and right context (both optional)
- `_` — marks the position of the replacement

### Replacement Operators

| Operator | Description |
|----------|-------------|
| `->` | Left-to-right rewrite |
| `<-` | Right-to-left rewrite |
| `<->` | Bidirectional rewrite |
| `(->)` | Optional rewrite |
| `@->` | Longest match rewrite |

### Context Markers

| Marker | Description |
|--------|-------------|
| `\|\|` | Input context |
| `//` | Output context |
| `\\` | Left context |
| `\/` | Output both directions |

## Special Symbols

| Symbol | Description |
|--------|-------------|
| `0` | Epsilon (empty string) |
| `.#.` | Word boundary |
| `?` | Any single symbol |
| `?*` | Any string |
| `\a` | Any symbol except `a` |

## Escaping Special Characters

Special characters can be escaped two ways:
- `%symbol` — percent-sign escaping (e.g. `%+` for a literal `+`)
- `"symbol"` — quote escaping (e.g. `"+"`)

Special characters that require escaping: `! " # $ % & ( ) * + , - . / 0 : ; < > ? [ \ ] ^ _ { | } ~`

## Built-in Functions

All begin with `_`:
- `_isunambiguous()` — checks if transducer is unambiguous
- `_isfunctional()` — checks if transducer is functional
- `_notid()` — removes identity pairs
- `_flatten()` — flattens flag diacritics
- `_eq()` — equality check
- `_ambpart()` — ambiguous part

## Composing Lexc with Rules

```
def Lexicon @"lexicon.lexc" ;
def Rule1   a -> b || c _ d ;
regex Lexicon .o. Rule1 .o. Rule2 ;
```

The `.o.` operator composes the lexicon transducer with phonological rules to produce the final surface forms.
