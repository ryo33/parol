//!
//! Grammar flow analysis
//! FOLLOW k of productions and non-terminals
//!

use super::FollowCache;
use crate::analysis::compiled_terminal::CompiledTerminal;
use crate::analysis::k_decision::CacheEntry;
use crate::analysis::FirstCache;
use crate::grammar::symbol_string::SymbolString;
use crate::{GrammarConfig, KTuple, KTuples, Pos, Pr, Symbol, TerminalKind};
use parol_runtime::lexer::FIRST_USER_TOKEN;
use parol_runtime::log::trace;
use parol_runtime::TerminalIndex;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Result type for each non-terminal:
/// The set of the follow k terminals
type DomainType = KTuples;

/// KTuples for non-terminals in non-terminal-index (alphabetical) order
pub type FollowSet = Vec<DomainType>;

/// The result map is applied to each iteration step.
/// It is also returned after each iteration step.
/// It maps non-terminal positions to follow sets.
pub(crate) type ResultMap = HashMap<Pos, DomainType>;

/// The type of the function in the equation system
/// It is called for each non-terminal
type TransferFunction =
    Arc<dyn Fn(Arc<ResultMap>, Arc<RwLock<FollowSet>>) -> DomainType + Send + Sync + 'static>;

type EquationSystem = HashMap<Pos, TransferFunction>;

/// # [`StepFunction`] Documentation
/// The StepFunction type is a type alias for an `Arc` of a `dyn Fn`  that takes four parameters and
/// returns a `ResultMap'.
/// This function is called in each step of the iteration process until the results (the result map
/// and the non-terminal vector) don't change anymore.
///
/// ## Parameters
///   * `es: Arc<EquationSystem>` - An `Arc` of an `EquationSystem` struct.
///     Each function in this equation system is called in each iteration step while iterating over
///     the non-terminal positions of the grammar which are taken from the second parameter of this
///     function. Each position is used to retrieve the associated function from this equation
///     system.
///     It is wrapped in an `Arc` to make it read accessible from multiple threads.
///   * `result_map: Arc<ResultMap>` - An `Arc` of a `ResultMap` struct.
///     This is the actual input for each iteration generated by the previous iteration.
///     It is wrapped in an `Arc` to make it read accessible from multiple threads.
///   * `non_terminal_positions: Arc<HashMap<Pos, usize>>` - An `Arc` of a `HashMap` of `Pos` and `usize`.
///     This is the association of non-terminal positions to non-terminal indices in the
///     non-terminal vector (the fourth parameter of this function) and is used to find the correct
///     place where the non-terminal result has to be accumulated.
///     It is wrapped in an `Arc` to make it read accessible from multiple threads.
///   * `non_terminal_results: Arc<RwLock<FollowSet>>` - An `Arc` of a `RwLock` of a `Vec` of `DomainType`.
///     This is the actual value returned by the [follow_k] function and is amended in each
///     iteration step by combining all results for all position of a certain non-terminal into a
///     single result (a k-tuple, i.e. a trie of terminal strings).
///     It is wrapped in an `Arc<RwLock>` to make it write accessible from multiple threads.
/// ## Return Value
/// The `StepFunction` returns a `ResultMap` struct that was extended in each iteration step.
type StepFunction = Arc<
    dyn Fn(
        Arc<EquationSystem>,
        Arc<ResultMap>,
        Arc<HashMap<Pos, usize>>,
        Arc<RwLock<FollowSet>>,
    ) -> ResultMap,
>;

///
/// Calculates the FOLLOW k sets for all non-terminals of the given grammar.
///
pub fn follow_k(
    grammar_config: &GrammarConfig,
    k: usize,
    first_cache: &FirstCache,
    follow_cache: &FollowCache,
) -> (ResultMap, FollowSet) {
    let cfg = &grammar_config.cfg;

    let terminals = grammar_config.cfg.get_ordered_terminals_owned();

    let terminal_index = Arc::new(move |t: &str, k: TerminalKind| -> TerminalIndex {
        (terminals
            .iter()
            .position(|(trm, kind, _)| *trm == t && kind.behaves_like(k))
            .unwrap() as TerminalIndex)
            + FIRST_USER_TOKEN
    });

    let first_k_of_nt = Arc::new(first_cache.get(k, grammar_config).1);

    let start_symbol = cfg.get_start_symbol();

    let non_terminal_index_finder = cfg.get_non_terminal_index_function();

    let non_terminal_positions = Arc::new(
        cfg.get_non_terminal_positions()
            .iter()
            .filter(|(p, _)| p.sy_index() > 0)
            .fold(HashMap::<Pos, usize>::new(), |mut acc, (p, s)| {
                acc.insert(*p, non_terminal_index_finder(s));
                acc
            }),
    );

    let equation_system: EquationSystem =
        cfg.pr
            .iter()
            .enumerate()
            .fold(EquationSystem::new(), |es, (i, pr)| {
                update_production_equations(
                    es,
                    i,
                    pr,
                    &non_terminal_index_finder,
                    first_k_of_nt.clone(),
                    terminal_index.clone(),
                    k,
                )
            });

    trace!(
        "Number of equations in equation system for FOLLOW(k) is {}",
        equation_system.len()
    );

    let equation_system = Arc::new(equation_system);

    // Single threaded variant
    let step_function: StepFunction = Arc::new(
        move |es: Arc<EquationSystem>,
              result_map: Arc<ResultMap>,
              non_terminal_positions: Arc<HashMap<Pos, usize>>,
              non_terminal_results: Arc<RwLock<FollowSet>>| {
            let new_result_vector = Arc::new(RwLock::new(ResultMap::new()));
            result_map.par_iter().for_each(|(pos, _)| {
                // Call each function of the equation system
                let pos_result = es[pos](result_map.clone(), non_terminal_results.clone());

                // Combine the result to the non_terminal_results.
                let sym = non_terminal_positions.get(pos).unwrap();
                if let Some(set) = non_terminal_results.write().unwrap().get_mut(*sym) {
                    *set = set.union(&pos_result).0;
                }

                // And put the result into the new result vector.
                new_result_vector.write().unwrap().insert(*pos, pos_result);
            });
            Arc::into_inner(new_result_vector)
                .unwrap()
                .into_inner()
                .unwrap()
        },
    );

    // Heuristically tweaked
    // let factor = 4;
    // let max_threads: usize = num_cpus::get() * factor;

    // let step_function: StepFunction = Arc::new(
    //     move |es: Arc<EquationSystem>,
    //           result_map: Arc<ResultMap>,
    //           non_terminal_positions: Arc<HashMap<Pos, usize>>,
    //           non_terminal_results: Arc<RwLock<FollowSet>>| {
    //         let (tx, rx) = std::sync::mpsc::channel();
    //         let iter = &mut result_map.iter().map(|(pos, _)| *pos) as &mut dyn Iterator<Item = Pos>;
    //         let mut new_result_vector = ResultMap::new();
    //         loop {
    //             let mut threads = 0;
    //             // We take chunks of length `max_thread` from the iterator over the result map and
    //             // spawn a thread for each element of the current chunk.
    //             iter.take(max_threads).for_each(|pos| {
    //                 threads += 1;
    //                 let tx = tx.clone();
    //                 let es = es.clone();
    //                 let result_map = result_map.clone();
    //                 let non_terminal_results = non_terminal_results.clone();

    //                 // Call each function of the equation system...
    //                 std::thread::spawn(move || {
    //                     tx.send((pos, es[&pos](result_map, non_terminal_results)))
    //                         .unwrap();
    //                 });
    //             });

    //             (0..threads).for_each(|_| {
    //                 let (pos, pos_result) = rx.recv().unwrap();

    //                 // Also combine the result to the non_terminal_results.
    //                 let sym = non_terminal_positions.get(&pos).unwrap();
    //                 if let Some(set) = non_terminal_results.write().unwrap().get_mut(*sym) {
    //                     *set = set.union(&pos_result).0;
    //                 }

    //                 // ...and put the result into the new result vector.
    //                 new_result_vector.insert(pos, pos_result);
    //             });
    //             if threads == 0 {
    //                 // No threads could be created because the iteration of the result map ended.
    //                 break;
    //             }
    //         }
    //         new_result_vector
    //     },
    // );

    let non_terminal_results = Arc::new(RwLock::new(cfg.get_non_terminal_set().iter().fold(
        Vec::new(),
        |mut acc, nt| {
            if nt == start_symbol {
                acc.push(DomainType::end(k));
            } else {
                acc.push(DomainType::new(k));
            }
            acc
        },
    )));

    let mut result_map = if k == 0 {
        // k == 0: No previous cache result available
        Arc::new(
            non_terminal_positions
                .iter()
                .fold(ResultMap::new(), |mut acc, (p, _)| {
                    acc.insert(*p, DomainType::new(k));
                    acc
                }),
        )
    } else {
        let CacheEntry(r, _) = follow_cache.get(k - 1, grammar_config, first_cache);
        Arc::new(r.iter().map(|(p, t)| (*p, t.clone().set_k(k))).collect())
    };

    let mut iterations = 0usize;
    let mut new_result_vector;
    loop {
        new_result_vector = step_function(
            equation_system.clone(),
            result_map.clone(),
            non_terminal_positions.clone(),
            non_terminal_results.clone(),
        );
        if new_result_vector == *result_map {
            break;
        }
        result_map = Arc::new(new_result_vector);
        iterations += 1;
        trace!("Iteration number {} completed", iterations);
    }

    (
        new_result_vector,
        Arc::try_unwrap(non_terminal_results)
            .unwrap()
            .into_inner()
            .unwrap()
            .drain(..)
            .collect::<FollowSet>(),
    )
}

///
/// Creates functions that calculate the FOLLOW k sets for each occurrence of
/// a non-terminal in the given production and adds them to the equation system.
///
fn update_production_equations<T>(
    mut es: EquationSystem,
    prod_num: usize,
    pr: &Pr,
    non_terminal_index_finder: &impl Fn(&str) -> usize,
    first_k_of_nt: Arc<FollowSet>,
    terminal_index: Arc<T>,
    k: usize,
) -> EquationSystem
where
    T: Fn(&str, TerminalKind) -> TerminalIndex + Clone + Send + Sync + 'static,
{
    let parts = pr.get_r().iter().enumerate().fold(
        Vec::<(usize, SymbolString)>::new(),
        |mut acc, (i, s)| {
            match s {
                // For each non-terminal create a separate SymbolString
                Symbol::N(..) => acc.push((i + 1, SymbolString(vec![s.clone()]))),
                // Stack terminals as long as possible
                Symbol::T(_) => {
                    if acc.is_empty() {
                        acc.push((i + 1, SymbolString(vec![s.clone()])));
                    } else {
                        let last = acc.len() - 1;
                        let last_len = acc[last].1.len();
                        let last_terminal = &acc[last].1 .0[last_len - 1];
                        if matches!(last_terminal, Symbol::T(_)) {
                            // Only add to terminals
                            acc[last].1 .0.push(s.clone());
                        } else {
                            // Create a new start of terminal list
                            acc.push((i + 1, SymbolString(vec![s.clone()])));
                        }
                    }
                }
                Symbol::S(_) => (),
                Symbol::Push(_) => (),
                Symbol::Pop => (),
            }
            acc
        },
    );

    // For each non-terminal of the production (parts are separated into strings
    // of terminals and single non-terminals combined with the symbol-index) we
    // have to provide an equation.
    for (part_index, (symbol_index, symbol_string)) in parts.iter().enumerate() {
        if let Symbol::N(..) = &symbol_string.0[0] {
            let mut result_function: TransferFunction = Arc::new(move |_, _| DomainType::eps(k));
            for (_, symbol_string) in parts.iter().skip(part_index + 1) {
                let symbol_string_clone = symbol_string.clone();
                let symbol = symbol_string_clone.0[0].clone();
                match symbol {
                    Symbol::T(_) => {
                        let terminal_index = terminal_index.clone();
                        result_function =
                            Arc::new(move |result_map: Arc<ResultMap>, non_terminal_results| {
                                let mapper =
                                    |s| CompiledTerminal::create(s, terminal_index.clone());
                                result_function(result_map, non_terminal_results).k_concat(
                                    &DomainType::of(
                                        &[KTuple::from_slice_with(
                                            &symbol_string_clone.0,
                                            mapper,
                                            k,
                                        )],
                                        k,
                                    ),
                                    k,
                                )
                            });
                    }
                    Symbol::N(nt, _, _) => {
                        let first_k_of_nt = first_k_of_nt.clone();
                        let nt_i = non_terminal_index_finder(&nt);
                        result_function =
                            Arc::new(move |result_map: Arc<ResultMap>, non_terminal_results| {
                                let first_of_nt = first_k_of_nt.get(nt_i).unwrap();
                                result_function(result_map, non_terminal_results)
                                    .k_concat(first_of_nt, k)
                            });
                    }
                    Symbol::S(_) => (),
                    Symbol::Push(_) => (),
                    Symbol::Pop => (),
                }
            }
            let nt = non_terminal_index_finder(pr.get_n_str());
            es.insert(
                (prod_num, *symbol_index).into(),
                Arc::new(
                    move |result_map, non_terminal_results: Arc<RwLock<FollowSet>>| {
                        result_function(result_map, non_terminal_results.clone())
                            .k_concat(non_terminal_results.read().unwrap().get(nt).unwrap(), k)
                    },
                ),
            );
        }
    }

    es
}
