// ---------------------------------------------------------
// This file was generated by parol.
// It is not intended for manual editing and changes will be
// lost after next build.
// ---------------------------------------------------------

use parol_runtime::collection_literals::collection;
use parol_runtime::lr_parser::{LR1State, LRAction, LRParseTable, LRParser, LRProduction};
use parol_runtime::once_cell::sync::Lazy;
#[allow(unused_imports)]
use parol_runtime::parser::{ParseTreeType, ParseType, Production, Trans};
use parol_runtime::{ParolError, ParseTree, TerminalIndex};
use parol_runtime::{ScannerConfig, TokenStream, Tokenizer};
use std::path::Path;

use crate::scanner_states_grammar::ScannerStatesGrammar;
use crate::scanner_states_grammar_trait::ScannerStatesGrammarAuto;

use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};

pub const TERMINALS: &[&str; 11] = &[
    /*  0 */ UNMATCHABLE_TOKEN,
    /*  1 */ UNMATCHABLE_TOKEN,
    /*  2 */ UNMATCHABLE_TOKEN,
    /*  3 */ UNMATCHABLE_TOKEN,
    /*  4 */ UNMATCHABLE_TOKEN,
    /*  5 */ r"[a-zA-Z_]\w*",
    /*  6 */ r"\u{5c}[\u{22}\u{5c}bfnt]",
    /*  7 */ r"\u{5c}[\s^\n\r]*\r?\n",
    /*  8 */ r"[^\u{22}\u{5c}]+",
    /*  9 */ r"\u{22}",
    /* 10 */ ERROR_TOKEN,
];

pub const TERMINAL_NAMES: &[&str; 11] = &[
    /*  0 */ "EndOfInput",
    /*  1 */ "Newline",
    /*  2 */ "Whitespace",
    /*  3 */ "LineComment",
    /*  4 */ "BlockComment",
    /*  5 */ "Identifier",
    /*  6 */ "Escaped",
    /*  7 */ "EscapedLineEnd",
    /*  8 */ "NoneQuote",
    /*  9 */ "StringDelimiter",
    /* 10 */ "Error",
];

/* SCANNER_0: "INITIAL" */
const SCANNER_0: (&[&str; 5], &[TerminalIndex; 2]) = (
    &[
        /*  0 */ UNMATCHABLE_TOKEN,
        /*  1 */ NEW_LINE_TOKEN,
        /*  2 */ WHITESPACE_TOKEN,
        /*  3 */ r"(//.*(\r\n|\r|\n|$))",
        /*  4 */ r"((?ms)/\*.*?\*/)",
    ],
    &[5 /* Identifier */, 9 /* StringDelimiter */],
);

/* SCANNER_1: "String" */
const SCANNER_1: (&[&str; 5], &[TerminalIndex; 4]) = (
    &[
        /*  0 */ UNMATCHABLE_TOKEN,
        /*  1 */ UNMATCHABLE_TOKEN,
        /*  2 */ UNMATCHABLE_TOKEN,
        /*  3 */ UNMATCHABLE_TOKEN,
        /*  4 */ UNMATCHABLE_TOKEN,
    ],
    &[
        6, /* Escaped */
        7, /* EscapedLineEnd */
        8, /* NoneQuote */
        9, /* StringDelimiter */
    ],
);

pub const NON_TERMINALS: &[&str; 11] = &[
    /*  0 */ "Content",
    /*  1 */ "Escaped",
    /*  2 */ "EscapedLineEnd",
    /*  3 */ "Identifier",
    /*  4 */ "NoneQuote",
    /*  5 */ "Start",
    /*  6 */ "StartList",
    /*  7 */ "StringContent",
    /*  8 */ "StringContentList",
    /*  9 */ "StringDelimiter",
    /* 10 */ "StringElement",
];

static PARSE_TABLE: Lazy<LRParseTable> = Lazy::new(|| {
    LRParseTable::new(vec![
        // State 0
        LR1State {
            actions: collection! {
                0 /* '<$>' */ => LRAction::Reduce(6 /*StartList*/, 2),
                5 /* '[a-zA-Z_]\w*' */ => LRAction::Reduce(6 /*StartList*/, 2),
                9 /* '\u{22}' */ => LRAction::Reduce(6 /*StartList*/, 2),
            },
            gotos: collection! {
                6 /* StartList */ => 1,
            },
        },
        // State 1
        LR1State {
            actions: collection! {
                0 /* '<$>' */ => LRAction::Accept,
                5 /* '[a-zA-Z_]\w*' */ => LRAction::Shift(2),
                9 /* '\u{22}' */ => LRAction::Shift(3),
            },
            gotos: collection! {
                0 /* Content */ => 4,
                3 /* Identifier */ => 5,
                9 /* StringDelimiter */ => 6,
            },
        },
        // State 2
        LR1State {
            actions: collection! {
                0 /* '<$>' */ => LRAction::Reduce(3 /*Identifier*/, 12),
                5 /* '[a-zA-Z_]\w*' */ => LRAction::Reduce(3 /*Identifier*/, 12),
                9 /* '\u{22}' */ => LRAction::Reduce(3 /*Identifier*/, 12),
            },
            gotos: collection! {},
        },
        // State 3
        LR1State {
            actions: collection! {
                0 /* '<$>' */ => LRAction::Reduce(9 /*StringDelimiter*/, 16),
                5 /* '[a-zA-Z_]\w*' */ => LRAction::Reduce(9 /*StringDelimiter*/, 16),
                6 /* '\u{5c}[\u{22}\u{5c}bfnt]' */ => LRAction::Reduce(9 /*StringDelimiter*/, 16),
                7 /* '\u{5c}[\s^\n\r]*\r?\n' */ => LRAction::Reduce(9 /*StringDelimiter*/, 16),
                8 /* '[^\u{22}\u{5c}]+' */ => LRAction::Reduce(9 /*StringDelimiter*/, 16),
                9 /* '\u{22}' */ => LRAction::Reduce(9 /*StringDelimiter*/, 16),
            },
            gotos: collection! {},
        },
        // State 4
        LR1State {
            actions: collection! {
                0 /* '<$>' */ => LRAction::Reduce(6 /*StartList*/, 1),
                5 /* '[a-zA-Z_]\w*' */ => LRAction::Reduce(6 /*StartList*/, 1),
                9 /* '\u{22}' */ => LRAction::Reduce(6 /*StartList*/, 1),
            },
            gotos: collection! {},
        },
        // State 5
        LR1State {
            actions: collection! {
                0 /* '<$>' */ => LRAction::Reduce(0 /*Content*/, 3),
                5 /* '[a-zA-Z_]\w*' */ => LRAction::Reduce(0 /*Content*/, 3),
                9 /* '\u{22}' */ => LRAction::Reduce(0 /*Content*/, 3),
            },
            gotos: collection! {},
        },
        // State 6
        LR1State {
            actions: collection! {
                6 /* '\u{5c}[\u{22}\u{5c}bfnt]' */ => LRAction::Reduce(8 /*StringContentList*/, 7),
                7 /* '\u{5c}[\s^\n\r]*\r?\n' */ => LRAction::Reduce(8 /*StringContentList*/, 7),
                8 /* '[^\u{22}\u{5c}]+' */ => LRAction::Reduce(8 /*StringContentList*/, 7),
                9 /* '\u{22}' */ => LRAction::Reduce(8 /*StringContentList*/, 7),
            },
            gotos: collection! {
                7 /* StringContent */ => 7,
                8 /* StringContentList */ => 8,
            },
        },
        // State 7
        LR1State {
            actions: collection! {
                9 /* '\u{22}' */ => LRAction::Shift(3),
            },
            gotos: collection! {
                9 /* StringDelimiter */ => 9,
            },
        },
        // State 8
        LR1State {
            actions: collection! {
                6 /* '\u{5c}[\u{22}\u{5c}bfnt]' */ => LRAction::Shift(10),
                7 /* '\u{5c}[\s^\n\r]*\r?\n' */ => LRAction::Shift(11),
                8 /* '[^\u{22}\u{5c}]+' */ => LRAction::Shift(12),
                9 /* '\u{22}' */ => LRAction::Reduce(7 /*StringContent*/, 5),
            },
            gotos: collection! {
                1 /* Escaped */ => 13,
                2 /* EscapedLineEnd */ => 14,
                4 /* NoneQuote */ => 15,
                10 /* StringElement */ => 16,
            },
        },
        // State 9
        LR1State {
            actions: collection! {
                0 /* '<$>' */ => LRAction::Reduce(0 /*Content*/, 4),
                5 /* '[a-zA-Z_]\w*' */ => LRAction::Reduce(0 /*Content*/, 4),
                9 /* '\u{22}' */ => LRAction::Reduce(0 /*Content*/, 4),
            },
            gotos: collection! {},
        },
        // State 10
        LR1State {
            actions: collection! {
                6 /* '\u{5c}[\u{22}\u{5c}bfnt]' */ => LRAction::Reduce(1 /*Escaped*/, 13),
                7 /* '\u{5c}[\s^\n\r]*\r?\n' */ => LRAction::Reduce(1 /*Escaped*/, 13),
                8 /* '[^\u{22}\u{5c}]+' */ => LRAction::Reduce(1 /*Escaped*/, 13),
                9 /* '\u{22}' */ => LRAction::Reduce(1 /*Escaped*/, 13),
            },
            gotos: collection! {},
        },
        // State 11
        LR1State {
            actions: collection! {
                6 /* '\u{5c}[\u{22}\u{5c}bfnt]' */ => LRAction::Reduce(2 /*EscapedLineEnd*/, 14),
                7 /* '\u{5c}[\s^\n\r]*\r?\n' */ => LRAction::Reduce(2 /*EscapedLineEnd*/, 14),
                8 /* '[^\u{22}\u{5c}]+' */ => LRAction::Reduce(2 /*EscapedLineEnd*/, 14),
                9 /* '\u{22}' */ => LRAction::Reduce(2 /*EscapedLineEnd*/, 14),
            },
            gotos: collection! {},
        },
        // State 12
        LR1State {
            actions: collection! {
                6 /* '\u{5c}[\u{22}\u{5c}bfnt]' */ => LRAction::Reduce(4 /*NoneQuote*/, 15),
                7 /* '\u{5c}[\s^\n\r]*\r?\n' */ => LRAction::Reduce(4 /*NoneQuote*/, 15),
                8 /* '[^\u{22}\u{5c}]+' */ => LRAction::Reduce(4 /*NoneQuote*/, 15),
                9 /* '\u{22}' */ => LRAction::Reduce(4 /*NoneQuote*/, 15),
            },
            gotos: collection! {},
        },
        // State 13
        LR1State {
            actions: collection! {
                6 /* '\u{5c}[\u{22}\u{5c}bfnt]' */ => LRAction::Reduce(10 /*StringElement*/, 9),
                7 /* '\u{5c}[\s^\n\r]*\r?\n' */ => LRAction::Reduce(10 /*StringElement*/, 9),
                8 /* '[^\u{22}\u{5c}]+' */ => LRAction::Reduce(10 /*StringElement*/, 9),
                9 /* '\u{22}' */ => LRAction::Reduce(10 /*StringElement*/, 9),
            },
            gotos: collection! {},
        },
        // State 14
        LR1State {
            actions: collection! {
                6 /* '\u{5c}[\u{22}\u{5c}bfnt]' */ => LRAction::Reduce(10 /*StringElement*/, 10),
                7 /* '\u{5c}[\s^\n\r]*\r?\n' */ => LRAction::Reduce(10 /*StringElement*/, 10),
                8 /* '[^\u{22}\u{5c}]+' */ => LRAction::Reduce(10 /*StringElement*/, 10),
                9 /* '\u{22}' */ => LRAction::Reduce(10 /*StringElement*/, 10),
            },
            gotos: collection! {},
        },
        // State 15
        LR1State {
            actions: collection! {
                6 /* '\u{5c}[\u{22}\u{5c}bfnt]' */ => LRAction::Reduce(10 /*StringElement*/, 11),
                7 /* '\u{5c}[\s^\n\r]*\r?\n' */ => LRAction::Reduce(10 /*StringElement*/, 11),
                8 /* '[^\u{22}\u{5c}]+' */ => LRAction::Reduce(10 /*StringElement*/, 11),
                9 /* '\u{22}' */ => LRAction::Reduce(10 /*StringElement*/, 11),
            },
            gotos: collection! {},
        },
        // State 16
        LR1State {
            actions: collection! {
                6 /* '\u{5c}[\u{22}\u{5c}bfnt]' */ => LRAction::Reduce(8 /*StringContentList*/, 6),
                7 /* '\u{5c}[\s^\n\r]*\r?\n' */ => LRAction::Reduce(8 /*StringContentList*/, 6),
                8 /* '[^\u{22}\u{5c}]+' */ => LRAction::Reduce(8 /*StringContentList*/, 6),
                9 /* '\u{22}' */ => LRAction::Reduce(8 /*StringContentList*/, 6),
            },
            gotos: collection! {},
        },
    ])
});

pub const PRODUCTIONS: &[LRProduction; 17] = &[
    // 0 - Start: StartList /* Vec */;
    LRProduction { lhs: 5, len: 1 },
    // 1 - StartList: StartList Content;
    LRProduction { lhs: 6, len: 2 },
    // 2 - StartList: ;
    LRProduction { lhs: 6, len: 0 },
    // 3 - Content: Identifier;
    LRProduction { lhs: 0, len: 1 },
    // 4 - Content: StringDelimiter StringContent StringDelimiter;
    LRProduction { lhs: 0, len: 3 },
    // 5 - StringContent: StringContentList /* Vec */;
    LRProduction { lhs: 7, len: 1 },
    // 6 - StringContentList: StringContentList StringElement;
    LRProduction { lhs: 8, len: 2 },
    // 7 - StringContentList: ;
    LRProduction { lhs: 8, len: 0 },
    // 8 - StringContent: ;
    LRProduction { lhs: 7, len: 0 },
    // 9 - StringElement: Escaped;
    LRProduction { lhs: 10, len: 1 },
    // 10 - StringElement: EscapedLineEnd;
    LRProduction { lhs: 10, len: 1 },
    // 11 - StringElement: NoneQuote;
    LRProduction { lhs: 10, len: 1 },
    // 12 - Identifier: "[a-zA-Z_]\w*";
    LRProduction { lhs: 3, len: 1 },
    // 13 - Escaped: "\u{5c}[\u{22}\u{5c}bfnt]";
    LRProduction { lhs: 1, len: 1 },
    // 14 - EscapedLineEnd: "\u{5c}[\s^\n\r]*\r?\n";
    LRProduction { lhs: 2, len: 1 },
    // 15 - NoneQuote: "[^\u{22}\u{5c}]+";
    LRProduction { lhs: 4, len: 1 },
    // 16 - StringDelimiter: "\u{22}";
    LRProduction { lhs: 9, len: 1 },
];

static SCANNERS: Lazy<Vec<ScannerConfig>> = Lazy::new(|| {
    vec![
        ScannerConfig::new(
            "INITIAL",
            Tokenizer::build(TERMINALS, SCANNER_0.0, SCANNER_0.1).unwrap(),
            &[(9, 1)],
        ),
        ScannerConfig::new(
            "String",
            Tokenizer::build(TERMINALS, SCANNER_1.0, SCANNER_1.1).unwrap(),
            &[(9, 0)],
        ),
    ]
});

pub fn parse<'t, T>(
    input: &'t str,
    file_name: T,
    user_actions: &mut ScannerStatesGrammar<'t>,
) -> Result<ParseTree<'t>, ParolError>
where
    T: AsRef<Path>,
{
    let mut lr_parser = LRParser::new(5, &PARSE_TABLE, PRODUCTIONS, TERMINAL_NAMES, NON_TERMINALS);
    lr_parser.trim_parse_tree();

    // Initialize wrapper
    let mut user_actions = ScannerStatesGrammarAuto::new(user_actions);
    lr_parser.parse(
        TokenStream::new(input, file_name, &SCANNERS, 1).unwrap(),
        &mut user_actions,
    )
}
