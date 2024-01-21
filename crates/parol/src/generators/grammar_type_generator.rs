use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::generators::NamingHelper as NmHlp;
use crate::grammar::ProductionAttribute;
use crate::{Pr, Symbol, Terminal};
use anyhow::{anyhow, bail, Result};
use parol_runtime::log::trace;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Debug, Display, Error, Formatter};

use crate::{grammar::SymbolAttribute, Cfg, GrammarConfig};

use super::generate_terminal_name;
use super::symbol_table::{
    Function, FunctionBuilder, InstanceEntrailsBuilder, MetaSymbolKind, SymbolId, SymbolKind,
    SymbolTable, TypeEntrails,
};
use super::symbol_table_facade::{InstanceFacade, SymbolFacade, TypeFacade};

///
/// Type information for a given grammar
///
#[derive(Debug, Default)]
pub struct GrammarTypeInfo {
    /// All symbols are managed by the symbol table
    pub(crate) symbol_table: SymbolTable,

    /// Calculated types of non-terminals.
    /// These are the types that are used in the AST.
    /// They are also used as types for the arguments of the semantic actions in the
    /// semantic actions trait (user_action_trait_id).
    /// All these types are created in global scope.
    pub(crate) non_terminal_types: BTreeMap<String, SymbolId>,

    /// The type id of the *semantic actions trait* that contains functions for each non-terminal of
    /// the given grammar.
    /// It also contains the function 'on_comment_parsed' that is called when a comment is parsed.
    /// The user action trait is created in global scope.
    /// The type name is <GrammarName>GrammarTrait
    pub(crate) semantic_actions_trait_id: Option<SymbolId>,

    /// The type id of the adapter struct that is created for the given grammar in global scope.
    /// The type name is <GrammarName>GrammarAuto.
    /// The adapter struct contains functions for each production of the given grammar.
    /// It is an adapter and calls the semantic actions functions in the appropriate places with
    /// the constructed arguments of the corresponding non-terminal type.
    pub(crate) adapter_grammar_struct_id: Option<SymbolId>,

    /// The type id of the user action trait that contains only two functions.
    /// The first calls the adapter functions in the adapter struct.
    /// The second function 'on_comment_parsed' is called when a comment is parsed. This function
    /// calls the user action function 'on_comment_parsed' in the semantic actions trait.
    /// This trait created in global scope.
    /// The type name is always 'UserActionsTrait' and it is the interface called by the parser.
    pub(crate) parser_interface_trait_id: Option<SymbolId>,

    /// Functions in the adapter struct (adapter_grammar_struct_id) that are called via the user
    /// actions trait.
    pub(crate) adapter_actions: BTreeMap<ProductionIndex, SymbolId>,

    // Output types of productions
    pub(crate) production_types: BTreeMap<ProductionIndex, SymbolId>,

    /// The type completely comprising the whole structural information that could be generated by
    /// the given expanded grammar.
    /// It is a type of enum kind.
    /// We use this as ASTType for the generated source.
    pub(crate) ast_enum_type: SymbolId,

    /// Indicates if the auto generation mode is active
    pub(crate) auto_generate: bool,

    /// Helper
    terminals: Vec<String>,
    terminal_names: Vec<String>,

    // Contains non-terminals that should be represented as vectors in the AST Enum type
    vector_typed_non_terminals: HashSet<String>,

    // Contains non-terminals that should be represented as options in the AST Enum type
    option_typed_non_terminals: HashSet<String>,
}

impl GrammarTypeInfo {
    /// Create a new item
    /// Initializes the inner data structures.
    pub fn try_new(grammar_type_name: &str) -> Result<Self> {
        let mut me = Self::default();
        me.symbol_table = SymbolTable::new();

        // Insert the fix UserActionsTrait into the global scope
        me.parser_interface_trait_id = Some(
            me.symbol_table
                .insert_global_type("UserActionsTrait", TypeEntrails::Trait)?,
        );

        // Insert the Semantic Actions Trait into the global scope
        me.semantic_actions_trait_id = Some(me.symbol_table.insert_global_type(
            &format!(
                "{}GrammarTrait",
                NmHlp::to_upper_camel_case(grammar_type_name)
            ),
            TypeEntrails::Trait,
        )?);

        // Insert the fix <GrammarName>GrammarAuto struct into the global scope
        me.adapter_grammar_struct_id = Some(me.symbol_table.insert_global_type(
            &format!(
                "{}GrammarAuto",
                NmHlp::to_upper_camel_case(grammar_type_name)
            ),
            TypeEntrails::Struct,
        )?);

        for n in ["new", "push", "pop", "trace_item_stack"] {
            me.symbol_table.insert_type(
                me.adapter_grammar_struct_id.unwrap(),
                n,
                TypeEntrails::Function(Function::default()),
            )?;
        }

        // Insert the fix Token type into the global scope, simply to avoid name clashes
        let token_type_id = me
            .symbol_table
            .insert_global_type("Token", TypeEntrails::Token)?;

        // Insert the fix 'on_comment_parsed' function into the semantic actions trait to avoid name
        // clashes with a possible non-terminal 'OnCommentParsed'
        let on_comment_parsed_id = me.symbol_table.insert_type(
            me.semantic_actions_trait_id.unwrap(),
            "on_comment_parsed",
            TypeEntrails::Function(Function::default()),
        )?;
        let function_type_id = me.symbol_table.symbol_as_type(on_comment_parsed_id).my_id();
        me.symbol_table.insert_instance(
            function_type_id,
            "token",
            token_type_id,
            InstanceEntrailsBuilder::default().used(true).build()?,
            SymbolAttribute::None,
            "Called on skipped language comments",
        )?;

        Ok(me)
    }

    /// Set the auto-generate mode
    /// Internally it adjust the used flags on the arguments of the actions.
    /// The arguments keep their used state only if auto generation is active.
    pub fn set_auto_generate(&mut self, auto_generate: bool) -> Result<()> {
        self.auto_generate = auto_generate;
        self.adjust_arguments_used(auto_generate)
    }

    /// Add user actions
    pub fn add_user_actions(&mut self, grammar_config: &GrammarConfig) -> Result<()> {
        grammar_config
            .non_terminals
            .iter()
            .fold(Vec::<&str>::new(), |mut acc, n| {
                if !acc.contains(&n.as_str()) {
                    acc.push(n.as_str());
                }
                acc
            })
            .iter()
            .try_for_each(|n| {
                self.add_user_action(n)?;
                Ok(())
            })
    }

    /// Add user action for the given non-terminal in the semantic actions trait.
    pub(crate) fn add_user_action(&mut self, non_terminal: &str) -> Result<SymbolId> {
        let action_fn = self.symbol_table.insert_type(
            self.semantic_actions_trait_id.unwrap(),
            non_terminal,
            TypeEntrails::Function(
                FunctionBuilder::default()
                    .non_terminal(non_terminal.to_string())
                    .build()
                    .unwrap(),
            ),
        )?;
        let function_type_id = self.symbol_table.symbol_as_type(action_fn).my_id();
        let argument_inner_type_id = self
            .symbol_table
            .get_global_type(&NmHlp::to_upper_camel_case(non_terminal))
            .ok_or_else(|| anyhow!("No type for non-terminal {} found!", non_terminal))?;
        let argument_type_id = self.symbol_table.get_or_create_type(
            SymbolTable::UNNAMED_TYPE,
            SymbolTable::GLOBAL_SCOPE,
            TypeEntrails::Ref(argument_inner_type_id),
        )?;
        self.symbol_table.insert_instance(
            function_type_id,
            "arg",
            argument_type_id,
            InstanceEntrailsBuilder::default().build()?,
            SymbolAttribute::None,
            &format!(
                "Argument of the user action for non-terminal '{}'",
                non_terminal
            ),
        )?;
        Ok(action_fn)
    }

    /// Returns the user action for the given non-terminal in the semantic actions trait.
    pub(crate) fn get_user_action(&self, non_terminal: &str) -> Result<SymbolId> {
        let user_action_trait = self.symbol_table.symbol_as_type(
            self.semantic_actions_trait_id
                .ok_or(anyhow!("User action trait not found!"))?,
        );
        self.symbol_table
            .scope(user_action_trait.member_scope())
            .symbol_by_name(
                &self.symbol_table,
                &NmHlp::to_lower_snake_case(non_terminal),
            )
            .ok_or(anyhow!("User action '{}' not found!", non_terminal))
    }

    /// Returns the user actions that are contained in the semantic actions trait.
    pub(crate) fn get_user_actions(&self) -> Vec<SymbolId> {
        self.symbol_table
            .scope(
                self.symbol_table
                    .symbol_as_type(
                        self.semantic_actions_trait_id
                            .expect("User action trait not found!"),
                    )
                    .member_scope(),
            )
            .symbols
            .iter()
            .filter(|s| self.symbol_table.symbol(**s).name() != "on_comment_parsed")
            .cloned()
            .collect::<Vec<_>>()
    }

    /// Sets the used flag on all arguments of the user actions in the adapter struct.
    fn adjust_arguments_used(&mut self, used: bool) -> Result<()> {
        for action_id in self.adapter_actions.values() {
            let arguments_scope = self.symbol_table.symbol_as_type(*action_id).member_scope();
            let args = self.symbol_table.scope(arguments_scope).symbols.clone();
            for arg in args {
                self.symbol_table.set_instance_used(arg, used)?;
            }
        }
        Ok(())
    }

    /// Add non-terminal type
    fn add_non_terminal_type(&mut self, non_terminal: &str, nt_type: SymbolId) -> Result<()> {
        self.non_terminal_types
            .insert(non_terminal.to_owned(), nt_type)
            .map_or_else(
                || {
                    trace!("Setting type for non-terminal {}", non_terminal);
                    Ok(())
                },
                |_| {
                    Err(anyhow!(
                        "Type for non-terminal {} already specified",
                        non_terminal
                    ))
                },
            )
    }

    ///
    /// Build the type information from the given grammar.
    ///
    pub fn build(&mut self, grammar_config: &GrammarConfig) -> Result<()> {
        let cfg = &grammar_config.cfg;
        self.terminals = cfg
            .get_ordered_terminals()
            .iter()
            .map(|(t, k, _)| k.expand(t))
            .collect::<Vec<String>>();

        self.terminal_names = self.terminals.iter().fold(Vec::new(), |mut acc, e| {
            let n = generate_terminal_name(e, None, cfg);
            acc.push(n);
            acc
        });

        self.create_initial_non_terminal_types(&grammar_config.cfg)?;
        self.deduce_actions(grammar_config)?;
        self.finish_non_terminal_types(&grammar_config.cfg)?;
        self.generate_ast_enum_type()?;
        self.add_user_actions(grammar_config)?;
        // self.minimize_boxed_types()?;
        self.symbol_table.propagate_lifetimes();
        Ok(())
    }

    ///
    /// Returns a vector of actions matching the given non-terminal n
    ///
    fn matching_actions(&self, n: &str) -> Vec<SymbolId> {
        self.adapter_actions
            .iter()
            .filter(|(_, a)| match &self.symbol_table.symbol(**a).kind() {
                SymbolKind::Type(t) => match &t.entrails {
                    TypeEntrails::Function(f) => f.non_terminal == n,
                    _ => panic!("Expecting a function!"),
                },
                _ => panic!("Expecting a type!"),
            })
            .map(|(_, s)| *s)
            .collect::<Vec<SymbolId>>()
    }

    /// Create the initial non-terminal types for each non-terminal of the grammar.
    fn create_initial_non_terminal_types(&mut self, cfg: &Cfg) -> Result<()> {
        for nt in cfg.get_non_terminal_set() {
            let alternatives = cfg.matching_productions(&nt);
            if alternatives.is_empty() {
                continue;
            }
            if let Ok(nt_type) = self.create_initial_non_terminal_type(&nt, alternatives) {
                self.add_non_terminal_type(&nt, nt_type)?;
            }
        }
        Ok(())
    }

    /// Create the initial non-terminal type. This is done by looking at the productions of the
    /// non-terminal and deducing the type from the production attributes resp. from the number
    /// of alternatives.
    /// If there is only one production for the non-terminal, we create an empty struct.
    /// If there are two productions for the non-terminal and special production attributes are
    /// present, we create an empty struct, too.
    /// Otherwise, we create an empty enum.
    fn create_initial_non_terminal_type(
        &mut self,
        non_terminal: &str,
        alternatives: Vec<(usize, &Pr)>,
    ) -> Result<SymbolId> {
        if alternatives.len() == 2 {
            match alternatives[0].1.get_attribute() {
                ProductionAttribute::None => (),
                ProductionAttribute::CollectionStart
                | ProductionAttribute::AddToCollection
                | ProductionAttribute::OptionalSome
                | ProductionAttribute::OptionalNone => {
                    return self
                        .symbol_table
                        .insert_global_type(non_terminal, TypeEntrails::Struct);
                }
            }
        }
        match alternatives.len() {
            // Productions can be optimized away, when they have duplicates!
            // This shouldn't actually happen anymore because structural equivalent (right-hand
            // sides of) productions aren't optimized away anymore (see issue #166).
            0 => bail!("Not supported!"),
            // Only one production for this non-terminal: we create an empty Struct
            1 => self
                .symbol_table
                .insert_global_type(non_terminal, TypeEntrails::Struct),
            // Otherwise: we generate an empty Enum
            _ => self
                .symbol_table
                .insert_global_type(non_terminal, TypeEntrails::Enum),
        }
    }

    fn finish_non_terminal_types(&mut self, cfg: &Cfg) -> Result<()> {
        for nt in cfg.get_non_terminal_set() {
            self.finish_non_terminal_type(&nt, cfg)?;
        }
        Ok(())
    }

    fn arguments(&self, action_id: SymbolId) -> Result<Vec<SymbolId>> {
        let action_scope = self.symbol_table.symbol_as_type(action_id).member_scope();
        Ok(self.symbol_table.scope(action_scope).symbols.clone())
    }

    fn finish_non_terminal_type(&mut self, nt: &str, cfg: &Cfg) -> Result<()> {
        let mut vector_typed_non_terminal_opt = None;
        let mut option_typed_non_terminal_opt = None;

        trace!("Finishing non-terminal type for {}", nt);

        let actions = self.matching_actions(nt).iter().try_fold(
            Vec::new(),
            |mut res: Vec<(SymbolId, ProductionAttribute)>, a| {
                self.symbol_table.function_type_semantic(*a).map(|t| {
                    res.push((*a, t));
                    res
                })
            },
        )?;

        if actions.len() == 1 {
            let arguments = self.arguments(actions[0].0)?;
            let non_terminal_type = *self.non_terminal_types.get(nt).unwrap();
            // Copy the arguments as struct members
            self.arguments_to_struct_members(&arguments, non_terminal_type)?;
        } else if actions.len() == 2
            && (actions[0].1 == ProductionAttribute::AddToCollection
                || actions[0].1 == ProductionAttribute::CollectionStart)
        {
            let primary_action = match (&actions[0].1, &actions[1].1) {
                (ProductionAttribute::AddToCollection, ProductionAttribute::CollectionStart) => {
                    actions[0].0
                }
                (ProductionAttribute::CollectionStart, ProductionAttribute::AddToCollection) => {
                    actions[1].0
                }
                _ => bail!("Unexpected combination of production attributes"),
            };
            let mut arguments = self.arguments(primary_action)?;
            arguments.pop(); // Remove the recursive part. Vec is wrapped outside.
            vector_typed_non_terminal_opt = Some(nt.to_string());
            let non_terminal_type = *self.non_terminal_types.get(nt).unwrap();
            // Copy the arguments as struct members
            self.arguments_to_struct_members(&arguments, non_terminal_type)?;
        } else if actions.len() == 2
            && (actions[0].1 == ProductionAttribute::OptionalNone
                || actions[0].1 == ProductionAttribute::OptionalSome)
        {
            let primary_action = match (&actions[0].1, &actions[1].1) {
                (ProductionAttribute::OptionalSome, ProductionAttribute::OptionalNone) => {
                    actions[0].0
                }
                (ProductionAttribute::OptionalNone, ProductionAttribute::OptionalSome) => {
                    actions[1].0
                }
                _ => bail!("Unexpected combination of production attributes"),
            };
            let arguments = self.arguments(primary_action)?;
            option_typed_non_terminal_opt = Some(nt.to_string());
            let non_terminal_type = *self.non_terminal_types.get(nt).unwrap();
            // Copy the arguments as struct members
            self.arguments_to_struct_members(&arguments, non_terminal_type)?;
        } else {
            // This is the "enum case". We generate an enum variant for each production with a name
            // built from the right-hand side of the corresponding production.
            let non_terminal_type = *self.non_terminal_types.get(nt).unwrap();
            for (action_id, _) in actions {
                let function = self.symbol_table.symbol_as_function(action_id)?;
                let variant_name = self.generate_production_rhs_name(function.prod_num, cfg);
                let entrails = TypeEntrails::EnumVariant(
                    *self.production_types.get(&function.prod_num).unwrap(),
                );
                self.symbol_table
                    .insert_type(non_terminal_type, &variant_name, entrails)?;
            }
        }

        if let Some(vector_typed_non_terminal) = vector_typed_non_terminal_opt {
            self.vector_typed_non_terminals
                .insert(vector_typed_non_terminal);
        }

        if let Some(option_typed_non_terminal) = option_typed_non_terminal_opt {
            self.option_typed_non_terminals
                .insert(option_typed_non_terminal);
        }

        Ok(())
    }

    /// Deduce the actions from the grammar.
    /// Actions are functions in the adapter struct (adapter_grammar_struct_id).
    fn deduce_actions(&mut self, grammar_config: &GrammarConfig) -> Result<()> {
        let scanner_state_resolver = grammar_config.get_scanner_state_resolver();
        let user_type_resolver = grammar_config.get_user_type_resolver();

        for (i, pr) in grammar_config.cfg.pr.iter().enumerate() {
            let rel_idx = grammar_config
                .cfg
                .get_alternation_index_of_production(i)
                .unwrap();

            let alts = grammar_config.cfg.get_alternations_count(i).unwrap();

            let function_entrails = FunctionBuilder::default()
                .non_terminal(pr.get_n())
                .prod_num(i)
                .rel_idx(rel_idx)
                .alts(alts)
                .prod_string(pr.format(&scanner_state_resolver, &user_type_resolver)?)
                .sem(pr.2)
                .build()
                .unwrap();

            let type_name = if alts == 1 {
                NmHlp::to_lower_snake_case(pr.get_n_str())
            } else {
                NmHlp::to_lower_snake_case(&format!("{}_{}", pr.get_n_str(), rel_idx))
            };

            let function_id = self.symbol_table.insert_type(
                self.adapter_grammar_struct_id.unwrap(),
                &type_name,
                TypeEntrails::Function(function_entrails),
            )?;

            self.build_arguments(grammar_config, function_id)?;

            self.adapter_actions.insert(i, function_id);

            self.build_production_type(function_id, i, &grammar_config.cfg)?;
        }
        Ok(())
    }

    fn get_terminal_index(&self, tr: &str) -> usize {
        self.terminals.iter().position(|t| *t == tr).unwrap()
    }

    /// Generates a member name from a symbol that stems from a production's right-hand side
    /// The second string in the returned tuple is used as description, here the terminal's content.
    pub fn generate_member_name(&self, s: &Symbol) -> (String, String) {
        match s {
            Symbol::N(n, ..) => (NmHlp::to_lower_snake_case(n), String::default()),
            Symbol::T(Terminal::Trm(t, k, ..)) => {
                let terminal_name = &self.terminal_names[self.get_terminal_index(&k.expand(t))];
                (NmHlp::to_lower_snake_case(terminal_name), t.to_string())
            }
            _ => panic!("Invalid symbol type {}", s),
        }
    }

    /// Convenience function
    pub fn generate_member_names(&self, rhs: &[Symbol]) -> Vec<(String, String)> {
        rhs.iter()
            .filter(|s| s.is_n() || s.is_t())
            .map(|s| self.generate_member_name(s))
            .collect::<Vec<(String, String)>>()
    }

    /// Build the arguments of the given function.
    /// The function is associated with a production.
    /// The arguments are the symbols of the right-hand side of the production.
    fn build_arguments(
        &mut self,
        grammar_config: &GrammarConfig,
        function_id: SymbolId,
    ) -> Result<()> {
        let entrails = self
            .symbol_table
            .symbol_as_type(function_id)
            .entrails()
            .clone();
        if let TypeEntrails::Function(function_entrails) = entrails {
            let prod = &grammar_config.cfg[function_entrails.prod_num];
            let mut types = prod
                .get_r()
                .iter()
                .filter(|s| s.is_t() || s.is_n())
                .try_fold(Vec::new(), |mut acc, s| {
                    self.deduce_type_of_symbol(s).map(|t| {
                        acc.push((t, s.attribute()));
                        acc
                    })
                })?;

            if function_entrails.sem == ProductionAttribute::AddToCollection {
                let ref_mut_last_type = &mut types.last_mut().unwrap().0;
                *ref_mut_last_type = match &ref_mut_last_type {
                    TypeEntrails::Box(r) => TypeEntrails::Vec(*r),
                    _ => bail!("Unexpected last symbol in production with AddToCollection"),
                    // _ => ref_mut_last_type.clone(),
                };
            }

            let result = self
                .generate_member_names(prod.get_r())
                .iter()
                .zip(types.drain(..))
                .try_for_each(|((n, r), (t, a))| {
                    // let type_name = if t.is_container() {
                    //     SymbolTable::UNNAMED_TYPE.to_owned()
                    // } else {
                    //     NmHlp::to_upper_camel_case(n)
                    // };
                    // Tokens are taken from the parameter list per definition.
                    let mut used =
                        matches!(t, TypeEntrails::Token) && a != SymbolAttribute::Clipped;
                    let type_id = if let TypeEntrails::UserDefinedType(k, ref u) = t {
                        if k == MetaSymbolKind::Token {
                            used = true;
                        }
                        self.symbol_table
                            .get_or_create_scoped_user_defined_type(k, u)?
                    } else {
                        self.symbol_table.get_or_create_type(
                            // &type_name,
                            SymbolTable::UNNAMED_TYPE,
                            SymbolTable::GLOBAL_SCOPE,
                            t,
                        )?
                    };
                    self.symbol_table
                        .insert_instance(
                            function_id,
                            n,
                            type_id,
                            InstanceEntrailsBuilder::default().used(used).build()?,
                            a,
                            r,
                        )
                        .map(|_| Ok(()))?
                });
            result
        } else {
            bail!("No function!")
        }
    }

    fn deduce_type_of_symbol(&self, symbol: &Symbol) -> Result<TypeEntrails> {
        match symbol {
            Symbol::T(Terminal::Trm(_, _, _, a, u)) => {
                if *a == SymbolAttribute::Clipped {
                    Ok(TypeEntrails::Clipped(MetaSymbolKind::Token))
                } else if let Some(ref user_defined_type) = u {
                    Ok(TypeEntrails::UserDefinedType(
                        MetaSymbolKind::Token,
                        user_defined_type.clone(),
                    ))
                } else {
                    Ok(TypeEntrails::Token)
                }
            }
            Symbol::N(n, a, u) => {
                let inner_type = self.non_terminal_types.get(n).unwrap();
                if let Some(ref user_defined_type) = u {
                    Ok(TypeEntrails::UserDefinedType(
                        MetaSymbolKind::NonTerminal(*inner_type),
                        user_defined_type.clone(),
                    ))
                } else {
                    match a {
                        SymbolAttribute::None => Ok(TypeEntrails::Box(*inner_type)),
                        // {
                        //     let inner_type = self.symbol_table.symbol_as_type(*inner_type);
                        //     Ok(inner_type.entrails().clone())
                        // }
                        SymbolAttribute::RepetitionAnchor => Ok(TypeEntrails::Vec(*inner_type)),
                        SymbolAttribute::Option => Ok(TypeEntrails::Option(*inner_type)),
                        SymbolAttribute::Clipped => Ok(TypeEntrails::Clipped(
                            MetaSymbolKind::NonTerminal(*inner_type),
                        )),
                    }
                }
            }
            _ => Err(anyhow!("Unexpected symbol kind: {}", symbol)),
        }
    }

    fn build_production_type(
        &mut self,
        function_id: SymbolId,
        prod_num: ProductionIndex,
        cfg: &Cfg,
    ) -> Result<()> {
        let non_terminal = self
            .symbol_table
            .symbol_as_function(function_id)?
            .non_terminal;
        let rhs_name = self.generate_production_rhs_name(prod_num, cfg);
        let struct_name = NmHlp::to_upper_camel_case(&format!("{}_{}", non_terminal, rhs_name));
        let production_type = self
            .symbol_table
            .insert_global_type(&struct_name, TypeEntrails::Struct)?;

        let arguments = self.arguments(function_id)?;
        // Copy the arguments as struct members
        self.arguments_to_struct_members(&arguments, production_type)?;
        self.production_types.insert(prod_num, production_type);
        Ok(())
    }

    /// Copy the arguments as struct members into the given production type.
    /// Here we convert members to boxed members if cycles are introduced.
    fn arguments_to_struct_members(
        &mut self,
        arguments: &[SymbolId],
        production_type: SymbolId,
    ) -> Result<()> {
        for arg in arguments {
            let inst_name = self.symbol_table.symbol(*arg).name().to_string();
            let (type_of_inst, description, sem) = {
                let inst = self.symbol_table.symbol_as_instance(*arg);
                (inst.type_id(), inst.description().to_owned(), inst.sem())
            };

            let _member_id = self.symbol_table.insert_instance(
                production_type,
                &inst_name,
                type_of_inst,
                InstanceEntrailsBuilder::default().used(true).build()?,
                sem,
                &description,
            )?;

            // {
            //     let type_symbol = self.symbol_table.symbol_as_type(type_of_inst);
            //     if matches!(type_symbol.entrails(), TypeEntrails::UserDefinedType(..)) {
            //         // We don't want to box user defined types
            //         continue;
            //     }
            // }

            // if self
            //     .symbol_table
            //     .is_recursive_in(production_type, type_of_inst)
            // {
            //     // Create a boxed type for the recursive type
            //     let boxed_type = self.symbol_table.get_or_create_type(
            //         SymbolTable::UNNAMED_TYPE,
            //         SymbolTable::GLOBAL_SCOPE,
            //         TypeEntrails::Box(type_of_inst),
            //     )?;
            //     debug_assert!(boxed_type != type_of_inst);
            //     trace!(
            //         "Replacing type of instance {} from {} to {}",
            //         member_id,
            //         type_of_inst,
            //         boxed_type
            //     );
            //     self.symbol_table
            //         .replace_type_of_inst(member_id, boxed_type)?;
            // }
        }
        Ok(())
    }

    fn generate_ast_enum_type(&mut self) -> Result<()> {
        self.ast_enum_type = self
            .symbol_table
            .insert_global_type("ASTType", TypeEntrails::Enum)?;

        let variants = self
            .non_terminal_types
            .iter()
            .fold(Vec::new(), |mut acc, nt| {
                let inner_type = if self.vector_typed_non_terminals.contains(nt.0) {
                    self.symbol_table
                        .get_or_create_type(
                            SymbolTable::UNNAMED_TYPE,
                            SymbolTable::GLOBAL_SCOPE,
                            TypeEntrails::Vec(*nt.1),
                        )
                        .unwrap()
                } else if self.option_typed_non_terminals.contains(nt.0) {
                    self.symbol_table
                        .get_or_create_type(
                            SymbolTable::UNNAMED_TYPE,
                            SymbolTable::GLOBAL_SCOPE,
                            TypeEntrails::Option(*nt.1),
                        )
                        .unwrap()
                } else {
                    *nt.1
                };

                acc.push((nt.0.to_string(), TypeEntrails::EnumVariant(inner_type)));
                acc
            });

        for (n, e) in variants {
            self.symbol_table.insert_type(self.ast_enum_type, &n, e)?;
        }

        Ok(())
    }

    // Generates an enum variant name for the given production from its right-hand side. If the
    // production has an empty RHS we simple name this enum variant "<NonTerminal>Empty".
    fn generate_production_rhs_name(&self, prod_num: usize, cfg: &Cfg) -> String {
        let pr = &cfg[prod_num];
        let lhs = pr.get_r();
        if lhs.is_empty() {
            format!("{}Empty", NmHlp::to_upper_camel_case(pr.get_n_str()))
        } else {
            lhs.iter().fold(String::new(), |mut acc, s| {
                match s {
                    Symbol::N(n, _, _) => acc.push_str(&NmHlp::to_upper_camel_case(n)),
                    Symbol::T(Terminal::Trm(t, k, ..)) => {
                        acc.push_str(&NmHlp::to_upper_camel_case(
                            &self.terminal_names[self.get_terminal_index(&k.expand(t))],
                        ))
                    }
                    _ => (),
                }
                acc
            })
        }
    }

    /// Returns a reference to the inner symbol table.
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}

impl Display for GrammarTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "// Symbol table:")?;
        writeln!(f, "{}", self.symbol_table)?;
        writeln!(f, "// Production types:")?;
        for (p, i) in &self.production_types {
            writeln!(
                f,
                "Prod: {p}: {i} /* {} */",
                self.symbol_table.symbol(*i).name()
            )?;
        }
        writeln!(f, "// Non-terminal types:")?;
        for (n, i) in &self.non_terminal_types {
            writeln!(f, "{n}: {i} /* {} */", self.symbol_table.symbol(*i).name())?;
        }
        writeln!(f, "// User actions:")?;
        self.get_user_actions().iter().try_for_each(|a| {
            let fun = self.symbol_table.symbol_as_function(*a).unwrap();
            writeln!(
                f,
                "{}: {} /* {} */",
                fun.prod_num, fun.non_terminal, fun.prod_string
            )
        })?;
        writeln!(f, "// Adapter actions:")?;
        for (p, i) in &self.adapter_actions {
            writeln!(
                f,
                "Prod: {p}: {i} /* {} */",
                self.symbol_table.symbol(*i).name()
            )?;
        }
        writeln!(f, "// Vector non-terminals:")?;
        for nt in &self.vector_typed_non_terminals {
            writeln!(f, "{}", nt)?;
        }
        writeln!(f, "// Option non-terminals:")?;
        for nt in &self.option_typed_non_terminals {
            writeln!(f, "{}", nt)?;
        }
        Ok(())
    }
}
