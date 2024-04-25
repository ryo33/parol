// ---------------------------------------------------------
// This file was generated by parol.
// It is not intended for manual editing and changes will be
// lost after next build.
// ---------------------------------------------------------

use parol_runtime::once_cell::sync::Lazy;
#[allow(unused_imports)]
use parol_runtime::parser::{LLKParser, LookaheadDFA, ParseTreeType, ParseType, Production, Trans};
use parol_runtime::{ParolError, ParseTree, TerminalIndex};
use parol_runtime::{TokenStream, Tokenizer};
use std::path::Path;

use crate::calc_grammar::CalcGrammar;
use crate::calc_grammar_trait::CalcGrammarAuto;

use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};

pub const TERMINALS: &[&str; 23] = &[
    /*  0 */ UNMATCHABLE_TOKEN,
    /*  1 */ UNMATCHABLE_TOKEN,
    /*  2 */ UNMATCHABLE_TOKEN,
    /*  3 */ UNMATCHABLE_TOKEN,
    /*  4 */ UNMATCHABLE_TOKEN,
    /*  5 */ r";",
    /*  6 */ r"==|!=",
    /*  7 */ r"(\+|-|\*|/|%|<<|>>|&|\^|\|)?=",
    /*  8 */ r"\|\|",
    /*  9 */ r"&&",
    /* 10 */ r"\|",
    /* 11 */ r"&",
    /* 12 */ r"<<|>>",
    /* 13 */ r"<=|<|>=|>",
    /* 14 */ r"\+",
    /* 15 */ r"-",
    /* 16 */ r"\*\*",
    /* 17 */ r"\*|/|%",
    /* 18 */ r"\(",
    /* 19 */ r"\)",
    /* 20 */ r"0|[1-9][0-9]*",
    /* 21 */ r"[a-zA-Z_][a-zA-Z0-9_]*",
    /* 22 */ ERROR_TOKEN,
];

pub const TERMINAL_NAMES: &[&str; 23] = &[
    /*  0 */ "EndOfInput",
    /*  1 */ "Newline",
    /*  2 */ "Whitespace",
    /*  3 */ "LineComment",
    /*  4 */ "BlockComment",
    /*  5 */ "Semicolon",
    /*  6 */ "EqualityOp",
    /*  7 */ "AssignOp",
    /*  8 */ "LogicalOrOp",
    /*  9 */ "LogicalAndOp",
    /* 10 */ "BitwiseOrOp",
    /* 11 */ "BitwiseAndOp",
    /* 12 */ "BitwiseShiftOp",
    /* 13 */ "RelationalOp",
    /* 14 */ "Plus",
    /* 15 */ "Minus",
    /* 16 */ "PowOp",
    /* 17 */ "MultOp",
    /* 18 */ "LParen",
    /* 19 */ "RParen",
    /* 20 */ "Number",
    /* 21 */ "Id",
    /* 22 */ "Error",
];

/* SCANNER_0: "INITIAL" */
const SCANNER_0: (&[&str; 5], &[TerminalIndex; 17]) = (
    &[
        /*  0 */ UNMATCHABLE_TOKEN,
        /*  1 */ NEW_LINE_TOKEN,
        /*  2 */ WHITESPACE_TOKEN,
        /*  3 */ r"(//.*(\r\n|\r|\n|$))",
        /*  4 */ r"((?ms)/\*.*?\*/)",
    ],
    &[
        5,  /* Semicolon */
        6,  /* EqualityOp */
        7,  /* AssignOp */
        8,  /* LogicalOrOp */
        9,  /* LogicalAndOp */
        10, /* BitwiseOrOp */
        11, /* BitwiseAndOp */
        12, /* BitwiseShiftOp */
        13, /* RelationalOp */
        14, /* Plus */
        15, /* Minus */
        16, /* PowOp */
        17, /* MultOp */
        18, /* LParen */
        19, /* RParen */
        20, /* Number */
        21, /* Id */
    ],
);

const MAX_K: usize = 2;

pub const NON_TERMINALS: &[&str; 44] = &[
    /*  0 */ "AddOp",
    /*  1 */ "AssignItem",
    /*  2 */ "AssignOp",
    /*  3 */ "Assignment",
    /*  4 */ "AssignmentList",
    /*  5 */ "BitwiseAnd",
    /*  6 */ "BitwiseAndList",
    /*  7 */ "BitwiseAndOp",
    /*  8 */ "BitwiseOr",
    /*  9 */ "BitwiseOrList",
    /* 10 */ "BitwiseOrOp",
    /* 11 */ "BitwiseShift",
    /* 12 */ "BitwiseShiftList",
    /* 13 */ "BitwiseShiftOp",
    /* 14 */ "Calc",
    /* 15 */ "CalcList",
    /* 16 */ "Equality",
    /* 17 */ "EqualityList",
    /* 18 */ "EqualityOp",
    /* 19 */ "Factor",
    /* 20 */ "Id",
    /* 21 */ "IdRef",
    /* 22 */ "Instruction",
    /* 23 */ "LogicalAnd",
    /* 24 */ "LogicalAndList",
    /* 25 */ "LogicalAndOp",
    /* 26 */ "LogicalOr",
    /* 27 */ "LogicalOrList",
    /* 28 */ "LogicalOrOp",
    /* 29 */ "Minus",
    /* 30 */ "Mult",
    /* 31 */ "MultList",
    /* 32 */ "MultOp",
    /* 33 */ "Negate",
    /* 34 */ "Number",
    /* 35 */ "Plus",
    /* 36 */ "PowOp",
    /* 37 */ "Power",
    /* 38 */ "PowerList",
    /* 39 */ "Relational",
    /* 40 */ "RelationalList",
    /* 41 */ "RelationalOp",
    /* 42 */ "Summ",
    /* 43 */ "SummList",
];

pub const LOOKAHEAD_AUTOMATA: &[LookaheadDFA; 44] = &[
    /* 0 - "AddOp" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 14, 1, 42), Trans(0, 15, 2, 43)],
        k: 1,
    },
    /* 1 - "AssignItem" */
    LookaheadDFA {
        prod0: 17,
        transitions: &[],
        k: 0,
    },
    /* 2 - "AssignOp" */
    LookaheadDFA {
        prod0: 4,
        transitions: &[],
        k: 0,
    },
    /* 3 - "Assignment" */
    LookaheadDFA {
        prod0: 18,
        transitions: &[],
        k: 0,
    },
    /* 4 - "AssignmentList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 15, 3, -1),
            Trans(0, 18, 3, -1),
            Trans(0, 20, 4, -1),
            Trans(0, 21, 1, -1),
            Trans(1, 5, 5, 20),
            Trans(1, 6, 5, 20),
            Trans(1, 7, 2, 19),
            Trans(1, 8, 5, 20),
            Trans(1, 9, 5, 20),
            Trans(1, 10, 5, 20),
            Trans(1, 11, 5, 20),
            Trans(1, 12, 5, 20),
            Trans(1, 13, 5, 20),
            Trans(1, 14, 5, 20),
            Trans(1, 15, 5, 20),
            Trans(1, 16, 5, 20),
            Trans(1, 17, 5, 20),
            Trans(3, 15, 5, 20),
            Trans(3, 18, 5, 20),
            Trans(3, 20, 5, 20),
            Trans(3, 21, 5, 20),
            Trans(4, 5, 5, 20),
            Trans(4, 6, 5, 20),
            Trans(4, 8, 5, 20),
            Trans(4, 9, 5, 20),
            Trans(4, 10, 5, 20),
            Trans(4, 11, 5, 20),
            Trans(4, 12, 5, 20),
            Trans(4, 13, 5, 20),
            Trans(4, 14, 5, 20),
            Trans(4, 15, 5, 20),
            Trans(4, 16, 5, 20),
            Trans(4, 17, 5, 20),
        ],
        k: 2,
    },
    /* 5 - "BitwiseAnd" */
    LookaheadDFA {
        prod0: 30,
        transitions: &[],
        k: 0,
    },
    /* 6 - "BitwiseAndList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 2, 32),
            Trans(0, 8, 2, 32),
            Trans(0, 9, 2, 32),
            Trans(0, 10, 2, 32),
            Trans(0, 11, 1, 31),
            Trans(0, 19, 2, 32),
        ],
        k: 1,
    },
    /* 7 - "BitwiseAndOp" */
    LookaheadDFA {
        prod0: 8,
        transitions: &[],
        k: 0,
    },
    /* 8 - "BitwiseOr" */
    LookaheadDFA {
        prod0: 27,
        transitions: &[],
        k: 0,
    },
    /* 9 - "BitwiseOrList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 2, 29),
            Trans(0, 8, 2, 29),
            Trans(0, 9, 2, 29),
            Trans(0, 10, 1, 28),
            Trans(0, 19, 2, 29),
        ],
        k: 1,
    },
    /* 10 - "BitwiseOrOp" */
    LookaheadDFA {
        prod0: 7,
        transitions: &[],
        k: 0,
    },
    /* 11 - "BitwiseShift" */
    LookaheadDFA {
        prod0: 39,
        transitions: &[],
        k: 0,
    },
    /* 12 - "BitwiseShiftList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 2, 41),
            Trans(0, 6, 2, 41),
            Trans(0, 8, 2, 41),
            Trans(0, 9, 2, 41),
            Trans(0, 10, 2, 41),
            Trans(0, 11, 2, 41),
            Trans(0, 12, 1, 40),
            Trans(0, 13, 2, 41),
            Trans(0, 19, 2, 41),
        ],
        k: 1,
    },
    /* 13 - "BitwiseShiftOp" */
    LookaheadDFA {
        prod0: 9,
        transitions: &[],
        k: 0,
    },
    /* 14 - "Calc" */
    LookaheadDFA {
        prod0: 0,
        transitions: &[],
        k: 0,
    },
    /* 15 - "CalcList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 2, 2),
            Trans(0, 15, 1, 1),
            Trans(0, 18, 1, 1),
            Trans(0, 20, 1, 1),
            Trans(0, 21, 1, 1),
        ],
        k: 1,
    },
    /* 16 - "Equality" */
    LookaheadDFA {
        prod0: 33,
        transitions: &[],
        k: 0,
    },
    /* 17 - "EqualityList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 2, 35),
            Trans(0, 6, 1, 34),
            Trans(0, 8, 2, 35),
            Trans(0, 9, 2, 35),
            Trans(0, 10, 2, 35),
            Trans(0, 11, 2, 35),
            Trans(0, 19, 2, 35),
        ],
        k: 1,
    },
    /* 18 - "EqualityOp" */
    LookaheadDFA {
        prod0: 3,
        transitions: &[],
        k: 0,
    },
    /* 19 - "Factor" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 15, 3, 56),
            Trans(0, 18, 4, 57),
            Trans(0, 20, 1, 54),
            Trans(0, 21, 2, 55),
        ],
        k: 1,
    },
    /* 20 - "Id" */
    LookaheadDFA {
        prod0: 60,
        transitions: &[],
        k: 0,
    },
    /* 21 - "IdRef" */
    LookaheadDFA {
        prod0: 59,
        transitions: &[],
        k: 0,
    },
    /* 22 - "Instruction" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 15, 3, -1),
            Trans(0, 18, 3, -1),
            Trans(0, 20, 4, -1),
            Trans(0, 21, 1, -1),
            Trans(1, 5, 5, 16),
            Trans(1, 6, 5, 16),
            Trans(1, 7, 2, 15),
            Trans(1, 8, 5, 16),
            Trans(1, 9, 5, 16),
            Trans(1, 10, 5, 16),
            Trans(1, 11, 5, 16),
            Trans(1, 12, 5, 16),
            Trans(1, 13, 5, 16),
            Trans(1, 14, 5, 16),
            Trans(1, 15, 5, 16),
            Trans(1, 16, 5, 16),
            Trans(1, 17, 5, 16),
            Trans(3, 15, 5, 16),
            Trans(3, 18, 5, 16),
            Trans(3, 20, 5, 16),
            Trans(3, 21, 5, 16),
            Trans(4, 5, 5, 16),
            Trans(4, 6, 5, 16),
            Trans(4, 8, 5, 16),
            Trans(4, 9, 5, 16),
            Trans(4, 10, 5, 16),
            Trans(4, 11, 5, 16),
            Trans(4, 12, 5, 16),
            Trans(4, 13, 5, 16),
            Trans(4, 14, 5, 16),
            Trans(4, 15, 5, 16),
            Trans(4, 16, 5, 16),
            Trans(4, 17, 5, 16),
        ],
        k: 2,
    },
    /* 23 - "LogicalAnd" */
    LookaheadDFA {
        prod0: 24,
        transitions: &[],
        k: 0,
    },
    /* 24 - "LogicalAndList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 2, 26),
            Trans(0, 8, 2, 26),
            Trans(0, 9, 1, 25),
            Trans(0, 19, 2, 26),
        ],
        k: 1,
    },
    /* 25 - "LogicalAndOp" */
    LookaheadDFA {
        prod0: 6,
        transitions: &[],
        k: 0,
    },
    /* 26 - "LogicalOr" */
    LookaheadDFA {
        prod0: 21,
        transitions: &[],
        k: 0,
    },
    /* 27 - "LogicalOrList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 5, 2, 23), Trans(0, 8, 1, 22), Trans(0, 19, 2, 23)],
        k: 1,
    },
    /* 28 - "LogicalOrOp" */
    LookaheadDFA {
        prod0: 5,
        transitions: &[],
        k: 0,
    },
    /* 29 - "Minus" */
    LookaheadDFA {
        prod0: 12,
        transitions: &[],
        k: 0,
    },
    /* 30 - "Mult" */
    LookaheadDFA {
        prod0: 47,
        transitions: &[],
        k: 0,
    },
    /* 31 - "MultList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 2, 49),
            Trans(0, 6, 2, 49),
            Trans(0, 8, 2, 49),
            Trans(0, 9, 2, 49),
            Trans(0, 10, 2, 49),
            Trans(0, 11, 2, 49),
            Trans(0, 12, 2, 49),
            Trans(0, 13, 2, 49),
            Trans(0, 14, 2, 49),
            Trans(0, 15, 2, 49),
            Trans(0, 17, 1, 48),
            Trans(0, 19, 2, 49),
        ],
        k: 1,
    },
    /* 32 - "MultOp" */
    LookaheadDFA {
        prod0: 14,
        transitions: &[],
        k: 0,
    },
    /* 33 - "Negate" */
    LookaheadDFA {
        prod0: 53,
        transitions: &[],
        k: 0,
    },
    /* 34 - "Number" */
    LookaheadDFA {
        prod0: 58,
        transitions: &[],
        k: 0,
    },
    /* 35 - "Plus" */
    LookaheadDFA {
        prod0: 11,
        transitions: &[],
        k: 0,
    },
    /* 36 - "PowOp" */
    LookaheadDFA {
        prod0: 13,
        transitions: &[],
        k: 0,
    },
    /* 37 - "Power" */
    LookaheadDFA {
        prod0: 50,
        transitions: &[],
        k: 0,
    },
    /* 38 - "PowerList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 2, 52),
            Trans(0, 6, 2, 52),
            Trans(0, 8, 2, 52),
            Trans(0, 9, 2, 52),
            Trans(0, 10, 2, 52),
            Trans(0, 11, 2, 52),
            Trans(0, 12, 2, 52),
            Trans(0, 13, 2, 52),
            Trans(0, 14, 2, 52),
            Trans(0, 15, 2, 52),
            Trans(0, 16, 1, 51),
            Trans(0, 17, 2, 52),
            Trans(0, 19, 2, 52),
        ],
        k: 1,
    },
    /* 39 - "Relational" */
    LookaheadDFA {
        prod0: 36,
        transitions: &[],
        k: 0,
    },
    /* 40 - "RelationalList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 2, 38),
            Trans(0, 6, 2, 38),
            Trans(0, 8, 2, 38),
            Trans(0, 9, 2, 38),
            Trans(0, 10, 2, 38),
            Trans(0, 11, 2, 38),
            Trans(0, 13, 1, 37),
            Trans(0, 19, 2, 38),
        ],
        k: 1,
    },
    /* 41 - "RelationalOp" */
    LookaheadDFA {
        prod0: 10,
        transitions: &[],
        k: 0,
    },
    /* 42 - "Summ" */
    LookaheadDFA {
        prod0: 44,
        transitions: &[],
        k: 0,
    },
    /* 43 - "SummList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 2, 46),
            Trans(0, 6, 2, 46),
            Trans(0, 8, 2, 46),
            Trans(0, 9, 2, 46),
            Trans(0, 10, 2, 46),
            Trans(0, 11, 2, 46),
            Trans(0, 12, 2, 46),
            Trans(0, 13, 2, 46),
            Trans(0, 14, 1, 45),
            Trans(0, 15, 1, 45),
            Trans(0, 19, 2, 46),
        ],
        k: 1,
    },
];

pub const PRODUCTIONS: &[Production; 61] = &[
    // 0 - Calc: CalcList /* Vec */;
    Production {
        lhs: 14,
        production: &[ParseType::N(15)],
    },
    // 1 - CalcList: Instruction ";"^ /* Clipped */ CalcList;
    Production {
        lhs: 15,
        production: &[ParseType::N(15), ParseType::T(5), ParseType::N(22)],
    },
    // 2 - CalcList: ;
    Production {
        lhs: 15,
        production: &[],
    },
    // 3 - EqualityOp: "==|!=";
    Production {
        lhs: 18,
        production: &[ParseType::T(6)],
    },
    // 4 - AssignOp: "(\+|-|\*|/|%|<<|>>|&|\^|\|)?=";
    Production {
        lhs: 2,
        production: &[ParseType::T(7)],
    },
    // 5 - LogicalOrOp: "\|\|";
    Production {
        lhs: 28,
        production: &[ParseType::T(8)],
    },
    // 6 - LogicalAndOp: "&&";
    Production {
        lhs: 25,
        production: &[ParseType::T(9)],
    },
    // 7 - BitwiseOrOp: "\|";
    Production {
        lhs: 10,
        production: &[ParseType::T(10)],
    },
    // 8 - BitwiseAndOp: "&";
    Production {
        lhs: 7,
        production: &[ParseType::T(11)],
    },
    // 9 - BitwiseShiftOp: "<<|>>";
    Production {
        lhs: 13,
        production: &[ParseType::T(12)],
    },
    // 10 - RelationalOp: "<=|<|>=|>";
    Production {
        lhs: 41,
        production: &[ParseType::T(13)],
    },
    // 11 - Plus: "\+";
    Production {
        lhs: 35,
        production: &[ParseType::T(14)],
    },
    // 12 - Minus: "-";
    Production {
        lhs: 29,
        production: &[ParseType::T(15)],
    },
    // 13 - PowOp: "\*\*";
    Production {
        lhs: 36,
        production: &[ParseType::T(16)],
    },
    // 14 - MultOp: "\*|/|%";
    Production {
        lhs: 32,
        production: &[ParseType::T(17)],
    },
    // 15 - Instruction: Assignment;
    Production {
        lhs: 22,
        production: &[ParseType::N(3)],
    },
    // 16 - Instruction: LogicalOr;
    Production {
        lhs: 22,
        production: &[ParseType::N(26)],
    },
    // 17 - AssignItem: Id AssignOp;
    Production {
        lhs: 1,
        production: &[ParseType::N(2), ParseType::N(20)],
    },
    // 18 - Assignment: AssignItem AssignmentList /* Vec */ LogicalOr;
    Production {
        lhs: 3,
        production: &[ParseType::N(26), ParseType::N(4), ParseType::N(1)],
    },
    // 19 - AssignmentList: AssignItem AssignmentList;
    Production {
        lhs: 4,
        production: &[ParseType::N(4), ParseType::N(1)],
    },
    // 20 - AssignmentList: ;
    Production {
        lhs: 4,
        production: &[],
    },
    // 21 - LogicalOr: LogicalAnd LogicalOrList /* Vec */;
    Production {
        lhs: 26,
        production: &[ParseType::N(27), ParseType::N(23)],
    },
    // 22 - LogicalOrList: LogicalOrOp LogicalAnd LogicalOrList;
    Production {
        lhs: 27,
        production: &[ParseType::N(27), ParseType::N(23), ParseType::N(28)],
    },
    // 23 - LogicalOrList: ;
    Production {
        lhs: 27,
        production: &[],
    },
    // 24 - LogicalAnd: BitwiseOr LogicalAndList /* Vec */;
    Production {
        lhs: 23,
        production: &[ParseType::N(24), ParseType::N(8)],
    },
    // 25 - LogicalAndList: LogicalAndOp BitwiseOr LogicalAndList;
    Production {
        lhs: 24,
        production: &[ParseType::N(24), ParseType::N(8), ParseType::N(25)],
    },
    // 26 - LogicalAndList: ;
    Production {
        lhs: 24,
        production: &[],
    },
    // 27 - BitwiseOr: BitwiseAnd BitwiseOrList /* Vec */;
    Production {
        lhs: 8,
        production: &[ParseType::N(9), ParseType::N(5)],
    },
    // 28 - BitwiseOrList: BitwiseOrOp BitwiseAnd BitwiseOrList;
    Production {
        lhs: 9,
        production: &[ParseType::N(9), ParseType::N(5), ParseType::N(10)],
    },
    // 29 - BitwiseOrList: ;
    Production {
        lhs: 9,
        production: &[],
    },
    // 30 - BitwiseAnd: Equality BitwiseAndList /* Vec */;
    Production {
        lhs: 5,
        production: &[ParseType::N(6), ParseType::N(16)],
    },
    // 31 - BitwiseAndList: BitwiseAndOp Equality BitwiseAndList;
    Production {
        lhs: 6,
        production: &[ParseType::N(6), ParseType::N(16), ParseType::N(7)],
    },
    // 32 - BitwiseAndList: ;
    Production {
        lhs: 6,
        production: &[],
    },
    // 33 - Equality: Relational EqualityList /* Vec */;
    Production {
        lhs: 16,
        production: &[ParseType::N(17), ParseType::N(39)],
    },
    // 34 - EqualityList: EqualityOp Relational EqualityList;
    Production {
        lhs: 17,
        production: &[ParseType::N(17), ParseType::N(39), ParseType::N(18)],
    },
    // 35 - EqualityList: ;
    Production {
        lhs: 17,
        production: &[],
    },
    // 36 - Relational: BitwiseShift RelationalList /* Vec */;
    Production {
        lhs: 39,
        production: &[ParseType::N(40), ParseType::N(11)],
    },
    // 37 - RelationalList: RelationalOp BitwiseShift RelationalList;
    Production {
        lhs: 40,
        production: &[ParseType::N(40), ParseType::N(11), ParseType::N(41)],
    },
    // 38 - RelationalList: ;
    Production {
        lhs: 40,
        production: &[],
    },
    // 39 - BitwiseShift: Summ BitwiseShiftList /* Vec */;
    Production {
        lhs: 11,
        production: &[ParseType::N(12), ParseType::N(42)],
    },
    // 40 - BitwiseShiftList: BitwiseShiftOp Summ BitwiseShiftList;
    Production {
        lhs: 12,
        production: &[ParseType::N(12), ParseType::N(42), ParseType::N(13)],
    },
    // 41 - BitwiseShiftList: ;
    Production {
        lhs: 12,
        production: &[],
    },
    // 42 - AddOp: Plus;
    Production {
        lhs: 0,
        production: &[ParseType::N(35)],
    },
    // 43 - AddOp: Minus;
    Production {
        lhs: 0,
        production: &[ParseType::N(29)],
    },
    // 44 - Summ: Mult SummList /* Vec */;
    Production {
        lhs: 42,
        production: &[ParseType::N(43), ParseType::N(30)],
    },
    // 45 - SummList: AddOp Mult SummList;
    Production {
        lhs: 43,
        production: &[ParseType::N(43), ParseType::N(30), ParseType::N(0)],
    },
    // 46 - SummList: ;
    Production {
        lhs: 43,
        production: &[],
    },
    // 47 - Mult: Power MultList /* Vec */;
    Production {
        lhs: 30,
        production: &[ParseType::N(31), ParseType::N(37)],
    },
    // 48 - MultList: MultOp Power MultList;
    Production {
        lhs: 31,
        production: &[ParseType::N(31), ParseType::N(37), ParseType::N(32)],
    },
    // 49 - MultList: ;
    Production {
        lhs: 31,
        production: &[],
    },
    // 50 - Power: Factor PowerList /* Vec */;
    Production {
        lhs: 37,
        production: &[ParseType::N(38), ParseType::N(19)],
    },
    // 51 - PowerList: PowOp Factor PowerList;
    Production {
        lhs: 38,
        production: &[ParseType::N(38), ParseType::N(19), ParseType::N(36)],
    },
    // 52 - PowerList: ;
    Production {
        lhs: 38,
        production: &[],
    },
    // 53 - Negate: Minus;
    Production {
        lhs: 33,
        production: &[ParseType::N(29)],
    },
    // 54 - Factor: Number;
    Production {
        lhs: 19,
        production: &[ParseType::N(34)],
    },
    // 55 - Factor: IdRef;
    Production {
        lhs: 19,
        production: &[ParseType::N(21)],
    },
    // 56 - Factor: Negate Factor;
    Production {
        lhs: 19,
        production: &[ParseType::N(19), ParseType::N(33)],
    },
    // 57 - Factor: "\("^ /* Clipped */ LogicalOr "\)"^ /* Clipped */;
    Production {
        lhs: 19,
        production: &[ParseType::T(19), ParseType::N(26), ParseType::T(18)],
    },
    // 58 - Number: "0|[1-9][0-9]*";
    Production {
        lhs: 34,
        production: &[ParseType::T(20)],
    },
    // 59 - IdRef: Id;
    Production {
        lhs: 21,
        production: &[ParseType::N(20)],
    },
    // 60 - Id: "[a-zA-Z_][a-zA-Z0-9_]*";
    Production {
        lhs: 20,
        production: &[ParseType::T(21)],
    },
];

static TOKENIZERS: Lazy<Vec<(&'static str, Tokenizer)>> = Lazy::new(|| {
    vec![(
        "INITIAL",
        Tokenizer::build(TERMINALS, SCANNER_0.0, SCANNER_0.1).unwrap(),
    )]
});

pub fn parse<'t, T>(
    input: &'t str,
    file_name: T,
    user_actions: &mut CalcGrammar<'t>,
) -> Result<ParseTree<'t>, ParolError>
where
    T: AsRef<Path>,
{
    let mut llk_parser = LLKParser::new(
        14,
        LOOKAHEAD_AUTOMATA,
        PRODUCTIONS,
        TERMINAL_NAMES,
        NON_TERMINALS,
    );
    // Initialize wrapper
    let mut user_actions = CalcGrammarAuto::new(user_actions);
    llk_parser.parse(
        TokenStream::new(input, file_name, &TOKENIZERS, MAX_K).unwrap(),
        &mut user_actions,
    )
}
