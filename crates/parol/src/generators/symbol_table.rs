//! The module symbol_table provides means to mimic the uniqueness of names per scope.
//! For auto-generation of symbols we need to adhere these rules of uniqueness.
use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::generators::symbol_table_facade::InstanceItem;
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::parser::parol_grammar::UserDefinedTypeName;
use crate::{generators::NamingHelper as NmHlp, utils::generate_name};
use anyhow::{bail, Result};
use parol_runtime::log::trace;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use std::fmt::{Debug, Display, Error, Formatter};
use std::ops::{Index, IndexMut};

use super::symbol_table_facade::{InstanceFacade, SymbolFacade, SymbolItem, TypeFacade, TypeItem};

/// Index type for Symbols
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, Eq, Hash, Ord, PartialOrd, TS,
)]
#[ts(export)]
pub(crate) struct SymbolId(usize);

impl Display for SymbolId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "Sym({})", self.0)
    }
}

/// Scope local index type for SymbolNames
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct ScopedNameId(ScopeId, usize);

impl ScopedNameId {
    pub(crate) fn is_unnamed(&self) -> bool {
        self.1 == Scope::UNNAMED_TYPE_NAME_ID
    }
}

impl Display for ScopedNameId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "Nam({}, {})", self.0, self.1)
    }
}

/// Id type for Scopes
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct ScopeId(usize);

impl Display for ScopeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "Sco({})", self.0)
    }
}

fn build_indent(amount: usize) -> String {
    const SPACES_PER_TAB: usize = 4;
    let space = " ".to_string();
    space.repeat(amount * SPACES_PER_TAB)
}

///
/// Type specificities of a function type
///
#[derive(Builder, Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct Function {
    /// Associated non-terminal
    pub(crate) non_terminal: String,

    /// Semantic specification
    #[builder(default)]
    pub(crate) sem: ProductionAttribute,

    /// Production number
    #[builder(default)]
    pub(crate) prod_num: ProductionIndex,

    /// The relative index of a production within its alternatives.
    #[builder(default)]
    pub(crate) rel_idx: usize,

    /// Number of alternatives, the number of productions that exist in the grammar which have the
    /// same non-terminal
    #[builder(default)]
    pub(crate) alts: usize,

    /// Formatted production in PAR syntax.
    #[builder(default)]
    pub(crate) prod_string: String,
}

impl Function {
    pub(crate) fn format(&self, fn_name: String) -> String {
        format!(
            "fn {} /* NT: {}{} Prod: {}, Rel: {}, Alts: {} */",
            fn_name,
            self.non_terminal,
            match self.sem {
                ProductionAttribute::None => "".to_string(),
                _ => format!(", {}", self.sem),
            },
            self.prod_num,
            self.rel_idx,
            self.alts,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum MetaSymbolKind {
    /// A part of a scoped name, normally a module name
    Module,
    /// A token
    Token,
    /// A non-terminal with inner non-terminal type
    NonTerminal(SymbolId),
}

impl Display for MetaSymbolKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            MetaSymbolKind::Module => write!(f, "Module"),
            MetaSymbolKind::Token => write!(f, "Tok"),
            MetaSymbolKind::NonTerminal(t) => write!(f, "Nt({})", t),
        }
    }
}

///
/// Type information used for auto-generation
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum TypeEntrails {
    /// Not specified, used as prototype during generation
    None,
    /// Will be generated as Token structure
    Token,
    /// A type with Box semantic
    Box(SymbolId),
    /// A type with Ref semantic an mutable state
    Ref(SymbolId),
    /// A type that refers to another type. Typically used when the type was a boxed type before
    Surrogate(SymbolId),
    /// A struct, i.e. a named collection of (name, type) tuples
    Struct,
    /// Will be generated as enum with given name
    Enum,
    /// A variant of an enum with a type specified by SymbolId
    EnumVariant(SymbolId),
    /// Will be generated as `Vec<T>` where T is the type specified by SymbolId
    Vec(SymbolId),
    /// A trait, normally the semantic actions trait generated for the user grammar
    Trait,
    /// A trait function
    Function(Function),
    /// An Option type of a type specified by SymbolId
    Option(SymbolId),
    /// An invisible type
    Clipped(MetaSymbolKind),
    /// User defined type
    UserDefinedType(MetaSymbolKind, UserDefinedTypeName),
}

impl TypeEntrails {
    pub(crate) fn format(&self, type_id: SymbolId, symbol_table: &SymbolTable) -> String {
        let uses_type_name = || {
            matches!(self, Self::Struct)
                | matches!(self, Self::Enum)
                | matches!(self, Self::EnumVariant(_))
                | matches!(self, Self::Function(_))
                | matches!(self, Self::Trait)
        };
        let my_type_name = if uses_type_name() {
            let my_type = symbol_table.symbol_as_type(type_id);
            my_type.name()
        } else {
            String::default()
        };
        let lifetime = symbol_table.lifetime(type_id);
        match self {
            TypeEntrails::None => "*TypeError*".to_string(),
            TypeEntrails::Token => format!("Token{}", lifetime),
            TypeEntrails::Box(r) => format!(
                "Box<{}{}>",
                symbol_table.symbol(*r).name(),
                symbol_table.lifetime(*r)
            ),
            TypeEntrails::Ref(r) => {
                let inner_type = symbol_table.symbol(*r);
                format!("&{}", inner_type.to_rust())
            }
            TypeEntrails::Struct => format!("{}{}", my_type_name, lifetime),
            TypeEntrails::Enum => format!("{}{}", my_type_name, lifetime),
            TypeEntrails::EnumVariant(t) => {
                let lifetime = if symbol_table.symbol_as_type(*t).is_container() {
                    "".to_string()
                } else {
                    lifetime
                };
                format!(
                    "{}({}{}),",
                    my_type_name,
                    symbol_table.symbol_as_type(*t).name(),
                    lifetime
                )
            }
            TypeEntrails::Vec(r) => format!(
                "Vec<{}{}>",
                symbol_table.symbol(*r).name(),
                symbol_table.lifetime(*r)
            ),
            TypeEntrails::Trait => format!("trait {}{}", my_type_name, lifetime),
            TypeEntrails::Function(f) => f.format(my_type_name),
            TypeEntrails::Option(o) => format!(
                "Option<Box<{}{}>>",
                symbol_table.symbol(*o).name(),
                symbol_table.lifetime(*o)
            ),
            TypeEntrails::Clipped(k) => format!("Clipped({})", k),
            TypeEntrails::UserDefinedType(_, u) => u.get_module_scoped_name(),
            TypeEntrails::Surrogate(s) => format!(
                "{}{}",
                symbol_table.symbol(*s).name(),
                symbol_table.lifetime(*s)
            ),
        }
    }

    pub(crate) fn inner_name(&self, symbol_table: &SymbolTable) -> String {
        match self {
            TypeEntrails::Box(t)
            | TypeEntrails::Surrogate(t)
            | TypeEntrails::Ref(t)
            | TypeEntrails::Vec(t)
            | TypeEntrails::Option(t)
            | TypeEntrails::UserDefinedType(MetaSymbolKind::NonTerminal(t), _) => {
                symbol_table.symbol(*t).name()
            }
            _ => "No inner name available!".to_string(),
        }
    }

    fn to_rust(&self, type_id: SymbolId, symbol_table: &SymbolTable) -> String {
        self.format(type_id, symbol_table)
    }

    pub(crate) fn sem(&self) -> SymbolAttribute {
        SymbolAttribute::None
    }

    pub(crate) fn _is_container(&self) -> bool {
        matches!(self, Self::Vec(_) | Self::Option(_) | Self::Box(_))
    }
}

impl Default for TypeEntrails {
    fn default() -> Self {
        Self::None
    }
}

///
/// Type information used for auto-generation
///
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct Type {
    /// The type specificities
    pub(crate) entrails: TypeEntrails,

    /// The inner scope
    pub(crate) member_scope: ScopeId,
}

impl Type {
    fn format(&self, symbol_table: &SymbolTable, my_symbol: &Symbol, scope_depth: usize) -> String {
        let scope = if !matches!(self.entrails, TypeEntrails::EnumVariant(_)) {
            format!(
                " {{\n{}\n{}}}",
                symbol_table
                    .scope(self.member_scope)
                    .format(symbol_table, scope_depth + 1),
                build_indent(scope_depth),
            )
        } else {
            ",".to_string()
        };
        format!(
            "{}{} /* Type: my_id {}, name_id: {} */ {}",
            build_indent(scope_depth),
            self.entrails.format(my_symbol.my_id, symbol_table),
            my_symbol.my_id,
            my_symbol.name_id,
            scope,
        )
    }

    pub(crate) fn to_rust(&self, symbol_table: &SymbolTable, my_symbol: &Symbol) -> String {
        self.entrails.to_rust(my_symbol.my_id, symbol_table)
    }

    pub(crate) fn name(&self, symbol_table: &SymbolTable, my_symbol: &Symbol) -> String {
        if my_symbol.name_id.is_unnamed() {
            self.entrails.format(my_symbol.my_id, symbol_table)
        } else {
            symbol_table.name(my_symbol.name_id).to_string()
        }
    }

    /// Returns the name of the type without lifetime
    pub(crate) fn inner_name(&self, symbol_table: &SymbolTable, my_symbol: &Symbol) -> String {
        let is_user_defined_type = matches!(self.entrails, TypeEntrails::UserDefinedType(..));
        if is_user_defined_type || my_symbol.name_id.is_unnamed() {
            self.entrails.inner_name(symbol_table)
        } else {
            symbol_table.name(my_symbol.name_id).to_string()
        }
    }

    // Used to suppress lifetime on the container types
    pub(crate) fn is_container(&self) -> bool {
        matches!(
            self.entrails,
            TypeEntrails::Box(_) | TypeEntrails::Vec(_) | TypeEntrails::Option(_)
        )
    }

    /// Returns the type id behind the symbol
    pub(crate) fn inner_type(&self) -> Option<SymbolId> {
        match self.entrails {
            TypeEntrails::Box(t)
            | TypeEntrails::Surrogate(t)
            | TypeEntrails::Ref(t)
            | TypeEntrails::EnumVariant(t)
            | TypeEntrails::Vec(t)
            | TypeEntrails::Option(t) => Some(t),
            _ => None,
        }
    }

    pub(crate) fn sem(&self) -> SymbolAttribute {
        self.entrails.sem()
    }
}

///
/// Instance specificities
///
#[derive(Builder, Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct InstanceEntrails {
    /// Indicates if the argument is used
    #[builder(default)]
    pub(crate) used: bool,
}

///
/// A typed instance, usually a function argument or a struct member
///
#[derive(Builder, Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct Instance {
    /// The scope where the instance resides
    pub(crate) scope: ScopeId,

    /// The instance's type id in the symbol table
    pub(crate) type_id: SymbolId,

    /// Instance specificities
    pub(crate) entrails: InstanceEntrails,

    /// Semantic information
    pub(crate) sem: SymbolAttribute,

    /// Description
    pub(crate) description: String,
}

impl Instance {
    fn format(&self, symbol_table: &SymbolTable, my_symbol: &Symbol, scope_depth: usize) -> String {
        let desc = if self.description.is_empty() {
            String::default()
        } else {
            format!(" /* {} */", self.description)
        };
        format!(
            "{}{}: {}{}{}{}",
            build_indent(scope_depth),
            self.name(symbol_table, my_symbol),
            symbol_table.symbol(self.type_id).name(),
            symbol_table.lifetime(self.type_id),
            desc,
            match self.sem {
                SymbolAttribute::None => "".to_string(),
                _ => format!(" /* {} */", self.sem),
            }
        )
    }

    fn to_rust(&self, symbol_table: &SymbolTable, my_symbol: &Symbol) -> String {
        let desc = if self.description.is_empty() {
            String::default()
        } else {
            // "*/" must be escaped
            format!("/* {} */", self.description.replace("*/", "*\\/"))
        };
        format!(
            "{}: {}, {}",
            self.name(symbol_table, my_symbol),
            symbol_table.symbol(self.type_id).to_rust(),
            desc,
        )
    }

    pub(crate) fn name(&self, symbol_table: &SymbolTable, my_symbol: &Symbol) -> String {
        symbol_table.name(my_symbol.name_id).to_string()
    }

    fn inner_type(&self) -> Option<SymbolId> {
        Some(self.type_id)
    }

    pub(crate) fn sem(&self) -> SymbolAttribute {
        self.sem
    }
}

///
/// A more general symbol
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum SymbolKind {
    Type(Type),
    Instance(Instance),
}

///
/// A more general symbol used in the symbol table
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Symbol {
    /// The symbol's id in the symbol table
    pub(crate) my_id: SymbolId,

    /// The symbol name's id in the enveloping scope
    pub(crate) name_id: ScopedNameId,

    /// The symbol's interior
    pub(crate) kind: SymbolKind,

    /// If a lifetime is present
    pub(crate) has_lifetime: bool,
}

impl Symbol {
    pub(crate) fn new(
        my_id: SymbolId,
        name_id: ScopedNameId,
        kind: SymbolKind,
        lifetime: bool,
    ) -> Self {
        Self {
            my_id,
            name_id,
            kind,
            has_lifetime: lifetime,
        }
    }

    pub(crate) fn my_id(&self) -> SymbolId {
        self.my_id
    }

    fn inner_type(&self) -> Option<SymbolId> {
        match &self.kind {
            SymbolKind::Type(t) => t.inner_type(),
            SymbolKind::Instance(i) => i.inner_type(),
        }
    }

    pub(crate) fn format(&self, symbol_table: &SymbolTable, scope_depth: usize) -> String {
        match &self.kind {
            SymbolKind::Type(t) => t.format(symbol_table, self, scope_depth),
            SymbolKind::Instance(i) => i.format(symbol_table, self, scope_depth),
        }
    }

    pub(crate) fn to_rust(&self, symbol_table: &SymbolTable) -> String {
        match &self.kind {
            SymbolKind::Type(t) => t.to_rust(symbol_table, self),
            SymbolKind::Instance(i) => i.to_rust(symbol_table, self),
        }
    }

    pub(crate) fn member_scope(&self) -> Option<ScopeId> {
        match &self.kind {
            SymbolKind::Type(t) => Some(t.member_scope),
            SymbolKind::Instance(_) => None,
        }
    }

    pub(crate) fn name(&self, symbol_table: &SymbolTable) -> String {
        match &self.kind {
            SymbolKind::Type(t) => t.name(symbol_table, self),
            SymbolKind::Instance(i) => i.name(symbol_table, self),
        }
    }

    pub(crate) fn sem(&self) -> SymbolAttribute {
        match &self.kind {
            SymbolKind::Type(t) => t.sem(),
            SymbolKind::Instance(i) => i.sem(),
        }
    }
}
///
/// Scope with symbols inside
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct Scope {
    pub(crate) parent: Option<ScopeId>,
    pub(crate) my_id: ScopeId,
    pub(crate) symbols: Vec<SymbolId>,
    names: Vec<String>,
}

impl Scope {
    const UNNAMED_TYPE_NAME_ID: usize = 0;

    pub(crate) fn new(parent: Option<ScopeId>, my_id: ScopeId) -> Self {
        Self {
            parent,
            my_id,
            symbols: Vec::new(),
            names: vec![SymbolTable::UNNAMED_TYPE.to_string()],
        }
    }

    pub(crate) fn make_unique_name(&self, preferred_name: String) -> String {
        if preferred_name == SymbolTable::UNNAMED_TYPE {
            SymbolTable::UNNAMED_TYPE.to_string()
        } else {
            generate_name(&self.names, preferred_name)
        }
    }

    pub(crate) fn add_name(&mut self, name: String) -> ScopedNameId {
        if name == SymbolTable::UNNAMED_TYPE {
            ScopedNameId(self.my_id, Self::UNNAMED_TYPE_NAME_ID)
        } else {
            let name_id = ScopedNameId(self.my_id, self.names.len());
            self.names.push(name);
            name_id
        }
    }

    fn insert_type(
        &mut self,
        name: &str,
        symbol_id: SymbolId,
        member_scope: ScopeId,
        entrails: TypeEntrails,
    ) -> Symbol {
        let type_name = match entrails {
            TypeEntrails::Function(_) => self.make_unique_name(NmHlp::to_lower_snake_case(name)),
            TypeEntrails::UserDefinedType(..) => name.to_owned(),
            _ => self.make_unique_name(NmHlp::to_upper_camel_case(name)),
        };
        trace!(
            "Scope {}: Inserting type {}({}) {:?}",
            self.my_id,
            type_name,
            symbol_id,
            entrails
        );
        let name_id = self.add_name(type_name);
        self.symbols.push(symbol_id);
        Symbol::new(
            symbol_id,
            name_id,
            SymbolKind::Type(Type {
                entrails,
                member_scope,
            }),
            false,
        )
    }

    fn insert_instance(
        &mut self,
        name: &str,
        symbol_id: SymbolId,
        type_id: SymbolId,
        entrails: InstanceEntrails,
        sem: SymbolAttribute,
        description: &str,
    ) -> Symbol {
        let instance_name = self.make_unique_name(NmHlp::to_lower_snake_case(name));
        let name_id = self.add_name(instance_name);
        self.symbols.push(symbol_id);
        trace!(
            "Scope {}: Inserting instance {}({}) (type: {}) {:?}",
            self.my_id,
            name,
            symbol_id,
            type_id,
            entrails
        );
        Symbol::new(
            symbol_id,
            name_id,
            SymbolKind::Instance(Instance {
                scope: self.my_id,
                type_id,
                entrails,
                sem,
                description: description.to_owned(),
            }),
            false,
        )
    }

    fn has_symbol(&self, symbol_id: SymbolId) -> bool {
        self.symbols.contains(&symbol_id)
    }

    pub(crate) fn symbol_by_name(
        &self,
        symbol_table: &SymbolTable,
        name: &str,
    ) -> Option<SymbolId> {
        self.symbols
            .iter()
            .find(|s| {
                let symbol = symbol_table.symbol(**s);
                debug_assert_eq!(symbol.name_id().0, self.my_id);
                self.names[symbol.name_id().1] == name
            })
            .copied()
    }

    pub(crate) fn format(&self, symbol_table: &SymbolTable, scope_depth: usize) -> String {
        format!(
            "{}// Scope: my_id: {}, parent: {}\n{}",
            build_indent(scope_depth),
            self.my_id,
            self.parent
                .map_or("No parent".to_string(), |i| format!("{}", i)),
            self.symbols
                .iter()
                .map(|s| symbol_table.symbol(*s).format(scope_depth))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "// Scope: my_id: {}, parent: {}\n//   Symbols:\n{}\n//   Names:\n{}",
            self.my_id,
            self.parent
                .map_or("No parent".to_string(), |i| format!("{}", i)),
            self.symbols
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
            self.names
                .iter()
                .enumerate()
                .map(|(i, n)| format!("{}: {}", ScopedNameId(self.my_id, i), n))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

///
/// Collection of symbols
///
/// Mimics rust's rules of uniqueness of symbol names within a certain scope.
/// This struct models the scopes and symbols within them only to the extend needed to auto-generate
/// flawless type and instance names.
/// Especially the deduction of the existence of lifetime parameter on generated types is modelled
/// as simple as possible.
///
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SymbolTable {
    // All symbols, ever created
    pub(crate) symbols: Vec<Symbol>,

    // All scopes
    // The one and only global scope has always index 0
    pub(crate) scopes: Vec<Scope>,
}

impl SymbolTable {
    pub(crate) const GLOBAL_SCOPE: ScopeId = ScopeId(0);
    // Some type symbols don't have a user defined name. You can think of it as built in types.
    // Regarding their names those types are identical. The differences are determined by other
    // properties. We need not to keep track of their individual names.
    pub(crate) const UNNAMED_TYPE: &'static str = "";

    /// Creates a new symbol table
    pub(crate) fn new() -> Self {
        Self {
            symbols: Vec::new(),
            scopes: vec![Scope::new(None, Self::GLOBAL_SCOPE)],
        }
    }

    /// Returns the number of symbols in the symbol table which can be used as id for the next
    /// symbol to be created
    pub(crate) fn next_symbol_id(&self) -> SymbolId {
        SymbolId(self.symbols.len())
    }

    /// Returns the number of scopes in the symbol table which can be used as id for the next
    /// scope to be created
    pub(crate) fn next_scope_id(&self) -> ScopeId {
        ScopeId(self.scopes.len())
    }

    /// Returns true if the symbol has a lifetime
    pub(crate) fn has_lifetime(&self, symbol_id: SymbolId) -> bool {
        self[symbol_id].has_lifetime
    }

    /// Returns the string representation of the lifetime of the symbol
    pub(crate) fn lifetime(&self, symbol_id: SymbolId) -> String {
        if self[symbol_id].has_lifetime {
            "<'t>".to_string()
        } else {
            "".to_string()
        }
    }

    /// Returns the name of the symbol
    pub(crate) fn name(&self, name_id: ScopedNameId) -> &str {
        &self.scope(name_id.0).names[name_id.1]
    }

    /// Returns a reference to the member symbols of the given type
    pub(crate) fn members(&self, type_id: SymbolId) -> Result<&[SymbolId]> {
        let type_symbol = self.symbol_as_type(type_id);
        Ok(&self.scope(type_symbol.member_scope()).symbols)
    }

    /// Returns a reference to the scope with the given id
    pub(crate) fn scope(&self, scope_id: ScopeId) -> &Scope {
        &self[scope_id]
    }

    /// Returns a mutable reference to the scope with the given id
    pub(crate) fn scope_mut(&mut self, scope_id: ScopeId) -> &mut Scope {
        &mut self[scope_id]
    }

    /// Returns a symbol facade of the symbol with the given id
    pub(crate) fn symbol(&self, symbol_id: SymbolId) -> impl SymbolFacade<'_> {
        SymbolItem::new(&self[symbol_id], self)
    }

    /// Returns the type name behind the symbol
    pub(crate) fn type_name(&self, symbol_id: SymbolId) -> Result<String> {
        let type_symbol = self.symbol(symbol_id);
        debug_assert!(matches!(type_symbol.kind(), SymbolKind::Type(_)));
        Ok(type_symbol.name())
    }

    /// Returns the instance name behind the symbol
    pub(crate) fn _instance_name(&self, symbol_id: SymbolId) -> Result<String> {
        let instance_symbol = self.symbol(symbol_id);
        debug_assert!(matches!(instance_symbol.kind(), SymbolKind::Instance(_)));
        Ok(instance_symbol.name())
    }

    /// Returns an instance facade of the symbol with the given id
    pub(crate) fn symbol_as_instance(&self, symbol_id: SymbolId) -> impl InstanceFacade<'_> {
        let instance_symbol = self.symbol(symbol_id);
        debug_assert!(
            matches!(instance_symbol.kind(), SymbolKind::Instance(_)),
            "Symbol {} is not an instance: {:?}!",
            symbol_id,
            instance_symbol.kind()
        );
        let instance = match &self[symbol_id].kind {
            SymbolKind::Type(_) => panic!("Ain't no instance!"),
            SymbolKind::Instance(i) => i,
        };
        InstanceItem::new(SymbolItem::new(&self[symbol_id], self), instance)
    }

    /// Sets the used flag of the symbol with the given id
    pub(crate) fn set_instance_used(&mut self, symbol_id: SymbolId, used: bool) -> Result<()> {
        match &mut self[symbol_id].kind {
            SymbolKind::Type(_) => bail!("Ain't no instance!"),
            SymbolKind::Instance(ref mut i) => i.entrails.used &= used,
        }
        Ok(())
    }

    /// Returns a type facade of the symbol with the given id
    pub(crate) fn symbol_as_type(&self, symbol_id: SymbolId) -> impl TypeFacade {
        let symbol_type = match &self[symbol_id].kind {
            SymbolKind::Type(t) => t,
            SymbolKind::Instance(_) => panic!("Ain't no type!"),
        };
        TypeItem::new(SymbolItem::new(&self[symbol_id], self), symbol_type)
    }

    /// Returns the function entrails of the symbol with the given id
    pub(crate) fn symbol_as_function(&self, symbol_id: SymbolId) -> Result<Function> {
        let function_type = self.symbol_as_type(symbol_id);
        match function_type.entrails() {
            TypeEntrails::Function(f) => Ok(f.clone()),
            _ => bail!("Expecting a function here"),
        }
    }

    /// Returns the function semantic of the symbol with the given id
    pub(crate) fn function_type_semantic(
        &self,
        symbol_id: SymbolId,
    ) -> Result<ProductionAttribute> {
        let function_type = self.symbol_as_type(symbol_id);
        match function_type.entrails() {
            TypeEntrails::Function(f) => Ok(f.sem),
            _ => bail!("Expecting a function here"),
        }
    }

    /// Creates a new scope and returns its id
    fn insert_scope(&mut self, parent: Option<ScopeId>) -> ScopeId {
        let my_id = self.next_scope_id();
        self.scopes.push(Scope::new(parent, my_id));
        my_id
    }

    /// Creates a new symbol and returns its id
    fn insert_symbol(&mut self, mut symbol: Symbol) -> SymbolId {
        let symbol_id = self.next_symbol_id();
        let is_clipped = symbol.sem() == SymbolAttribute::Clipped;
        symbol.has_lifetime = !is_clipped
            && matches!(
                symbol.kind,
                SymbolKind::Type(Type {
                    entrails: TypeEntrails::Token,
                    ..
                })
            );
        self.symbols.push(symbol);
        symbol_id
    }

    /// Returns all scopes that contain the symbol with the given id
    fn find_containing_scopes(&self, symbol_id: SymbolId) -> Vec<ScopeId> {
        self.scopes
            .iter()
            .filter(|scope| scope.has_symbol(symbol_id))
            .map(|scope| scope.my_id)
            .collect::<Vec<ScopeId>>()
    }

    /// Returns all symbols that have a member scope that is contained in the given scopes
    fn find_symbols_with_member_scopes(&self, scope_ids: &[ScopeId]) -> Vec<SymbolId> {
        self.symbols
            .iter()
            .filter(|symbol| {
                if let Some(scope) = symbol.member_scope() {
                    symbol.sem() != SymbolAttribute::Clipped && scope_ids.contains(&scope)
                } else {
                    false
                }
            })
            .map(|symbol| symbol.my_id())
            .collect::<Vec<SymbolId>>()
    }

    /// Returns all symbols that have a lifetime
    fn symbols_with_lifetime(&self) -> Vec<SymbolId> {
        self.symbols
            .iter()
            .enumerate()
            .filter_map(|(i, symbol)| {
                if symbol.has_lifetime {
                    debug_assert_eq!(i, symbol.my_id().0);
                    Some(symbol.my_id())
                } else {
                    None
                }
            })
            .collect::<Vec<SymbolId>>()
    }

    /// Propagates the lifetime from the given symbol up to the containing scopes
    fn propagate_lifetime(&mut self, symbol_id: SymbolId) {
        let parent_scope_ids = self.find_containing_scopes(symbol_id);
        let parent_symbols = self.find_symbols_with_member_scopes(&parent_scope_ids);
        for parent_symbol in &parent_symbols {
            self[*parent_symbol].has_lifetime = true;
        }
        let containing_symbols = self
            .symbols
            .iter()
            .filter(|symbol| {
                if let Some(inner_type) = symbol.inner_type() {
                    inner_type == symbol_id
                } else {
                    false
                }
            })
            .map(|symbol| symbol.my_id())
            .collect::<Vec<SymbolId>>();

        for containing_symbol in &containing_symbols {
            self[*containing_symbol].has_lifetime = true;
        }
    }

    /// Propagates lifetimes from the bottom up
    pub(crate) fn propagate_lifetimes(&mut self) {
        let mut symbols_with_lifetime = self.symbols_with_lifetime();
        let mut count = symbols_with_lifetime.len();
        let mut old_count = 0;
        while count != old_count {
            old_count = count;
            for symbol_id in symbols_with_lifetime {
                self.propagate_lifetime(symbol_id);
            }
            symbols_with_lifetime = self.symbols_with_lifetime();
            count = symbols_with_lifetime.len();
        }
    }

    /// Creates a new type symbol with the given name and type in the given scope and returns its id
    pub(crate) fn insert_type(
        &mut self,
        parent_symbol: SymbolId,
        type_name: &str,
        entrails: TypeEntrails,
    ) -> Result<SymbolId> {
        debug_assert!(parent_symbol.0 < self.symbols.len());
        let parent_scope = self
            .scope(self.symbol_as_type(parent_symbol).member_scope())
            .my_id;
        self.insert_type_in_scope(parent_scope, type_name, entrails)
    }

    /// Creates a new type symbol with the given name and type in the given scope and returns its id
    pub(crate) fn insert_type_in_scope(
        &mut self,
        parent_scope: ScopeId,
        type_name: &str,
        entrails: TypeEntrails,
    ) -> Result<SymbolId> {
        let symbol_id = self.next_symbol_id();
        let member_scope = self.insert_scope(Some(parent_scope));
        let symbol =
            self.scope_mut(parent_scope)
                .insert_type(type_name, symbol_id, member_scope, entrails);
        Ok(self.insert_symbol(symbol))
    }

    /// Creates a new type symbol with the given name and type in the global scope and returns its
    /// id
    pub(crate) fn insert_global_type(
        &mut self,
        type_name: &str,
        entrails: TypeEntrails,
    ) -> Result<SymbolId> {
        let symbol_id = self.next_symbol_id();
        let member_scope = self.insert_scope(Some(SymbolTable::GLOBAL_SCOPE));
        let symbol = self.scope_mut(Self::GLOBAL_SCOPE).insert_type(
            type_name,
            symbol_id,
            member_scope,
            entrails,
        );
        Ok(self.insert_symbol(symbol))
    }

    /// Creates a new instance symbol with the given name and type in the given scope and returns
    /// its id
    pub(crate) fn insert_instance(
        &mut self,
        parent_symbol: SymbolId,
        instance_name: &str,
        type_id: SymbolId,
        entrails: InstanceEntrails,
        sem: SymbolAttribute,
        description: &str,
    ) -> Result<SymbolId> {
        debug_assert!(parent_symbol.0 < self.symbols.len());
        let symbol_id = self.next_symbol_id();
        let member_scope = self.symbol_as_type(parent_symbol).member_scope();
        let symbol = self.scope_mut(member_scope).insert_instance(
            instance_name,
            symbol_id,
            type_id,
            entrails,
            sem,
            description,
        );
        Ok(self.insert_symbol(symbol))
    }

    /// Returns the symbol id of the type with the given name in the given scope
    /// If the type does not exist, it will be created
    pub(crate) fn get_or_create_type(
        &mut self,
        type_name: &str,
        scope: ScopeId,
        entrails: TypeEntrails,
    ) -> Result<SymbolId> {
        if let Some(symbol_id) = self.scope(scope).symbols.iter().find(|symbol_id| {
            let type_symbol = self.symbol_as_type(**symbol_id);
            (*type_symbol.entrails() == entrails && type_symbol.name() == type_name)
                || matches!(*type_symbol.entrails(), TypeEntrails::Token)
                    && matches!(entrails, TypeEntrails::Token)
        }) {
            return Ok(*symbol_id);
        }

        // Here we have to create a new type
        if scope == Self::GLOBAL_SCOPE {
            self.insert_global_type(type_name, entrails)
        } else {
            self.insert_type_in_scope(scope, type_name, entrails)
        }
    }

    /// Returns the symbol id of the type with the given name in the global scope
    pub(crate) fn get_global_type(&self, non_terminal: &str) -> Option<SymbolId> {
        self.scope(Self::GLOBAL_SCOPE)
            .symbols
            .iter()
            .find(|symbol_id| self[**symbol_id].name(self) == non_terminal)
            .copied()
    }

    /// Creates a new type whose name is interpreted as scoped name.
    /// The scopes are created if necessary.
    /// The type is created if necessary in the inner most scope.
    pub(crate) fn get_or_create_scoped_user_defined_type(
        &mut self,
        symbol_kind: MetaSymbolKind,
        user_defined_type: &UserDefinedTypeName,
    ) -> Result<SymbolId> {
        let mut symbol_id: SymbolId = SymbolId::default();
        let mut parent_scope = Self::GLOBAL_SCOPE;
        let mut stacked_names = Vec::new();
        let last = user_defined_type.len() - 1;
        for (i, type_part) in user_defined_type.names().iter().enumerate() {
            stacked_names.push(type_part.to_string());
            let symbol_kind = if i == last {
                symbol_kind
            } else {
                MetaSymbolKind::Module
            };
            symbol_id = self.get_or_create_type(
                type_part,
                parent_scope,
                TypeEntrails::UserDefinedType(
                    symbol_kind,
                    UserDefinedTypeName::new(stacked_names.clone()),
                ),
            )?;
            parent_scope = self.symbol_as_type(symbol_id).member_scope();
        }
        Ok(symbol_id)
    }

    /// Replace the type of the given instance symbol with the type of the referred symbol
    pub(crate) fn _replace_type_of_inst(
        &mut self,
        inst_symbol_id: SymbolId,
        referred_type_id: SymbolId,
    ) -> Result<()> {
        debug_assert!(matches!(
            self.symbol(inst_symbol_id).kind(),
            SymbolKind::Instance(_)
        ));
        match &mut self[inst_symbol_id].kind {
            SymbolKind::Type(_) => panic!("Ain't no instance!"),
            SymbolKind::Instance(i) => i.type_id = referred_type_id,
        };
        Ok(())
    }

    fn _inner_is_recursive(&self, mut ancestors: Vec<SymbolId>, next_symbol: SymbolId) -> bool {
        if ancestors.contains(&next_symbol) {
            return true;
        }
        match &self[next_symbol].kind {
            SymbolKind::Type(t) => match t.entrails {
                TypeEntrails::Ref(t)
                | TypeEntrails::Surrogate(t)
                | TypeEntrails::EnumVariant(t)
                | TypeEntrails::Option(t) => {
                    ancestors.push(t);
                    self._inner_is_recursive(ancestors, t)
                }
                TypeEntrails::UserDefinedType(MetaSymbolKind::NonTerminal(t), _) => {
                    ancestors.push(t);
                    self._inner_is_recursive(ancestors, t)
                }
                TypeEntrails::Struct | TypeEntrails::Enum => {
                    for member in self.members(next_symbol).unwrap() {
                        if self._inner_is_recursive(ancestors.clone(), *member) {
                            return true;
                        }
                    }
                    false
                }
                _ => false,
            },
            SymbolKind::Instance(t) => {
                ancestors.push(next_symbol);
                self._inner_is_recursive(ancestors, t.type_id)
            }
        }
    }

    pub(crate) fn _is_recursive_in(&self, parent_type_id: SymbolId, child_id: SymbolId) -> bool {
        let stack = vec![parent_type_id];
        self._inner_is_recursive(stack, child_id)
    }
}

impl Index<SymbolId> for SymbolTable {
    type Output = Symbol;

    fn index(&self, index: SymbolId) -> &Self::Output {
        &self.symbols[index.0]
    }
}

impl IndexMut<SymbolId> for SymbolTable {
    fn index_mut(&mut self, index: SymbolId) -> &mut Self::Output {
        &mut self.symbols[index.0]
    }
}

impl Index<ScopeId> for SymbolTable {
    type Output = Scope;

    fn index(&self, index: ScopeId) -> &Self::Output {
        &self.scopes[index.0]
    }
}

impl IndexMut<ScopeId> for SymbolTable {
    fn index_mut(&mut self, index: ScopeId) -> &mut Self::Output {
        &mut self.scopes[index.0]
    }
}

impl Index<ScopedNameId> for SymbolTable {
    type Output = str;

    fn index(&self, index: ScopedNameId) -> &Self::Output {
        &self.scope(index.0).names[index.1]
    }
}

impl IndexMut<ScopedNameId> for SymbolTable {
    fn index_mut(&mut self, index: ScopedNameId) -> &mut Self::Output {
        &mut self.scope_mut(index.0).names[index.1]
    }
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "// Symbols:")?;
        for (i, sym) in self.symbols.iter().enumerate() {
            writeln!(f, "Sym({}): {}", i, sym.format(self, 0))?;
        }
        writeln!(f, "// Scopes:")?;
        for scope in &self.scopes {
            writeln!(f, "{}", scope)?;
        }
        writeln!(f, "// Scope hierarchy:")?;
        write!(f, "{}", self.scope(Self::GLOBAL_SCOPE).format(self, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_scope_relations() {
        let mut symbol_table = SymbolTable::new();
        // Global scope should have been added automatically in `new`
        assert_eq!(1, symbol_table.scopes.len());
        // Global scope should have no parent
        assert_eq!(None, symbol_table.scope(SymbolTable::GLOBAL_SCOPE).parent);

        let struct_id = symbol_table
            .insert_global_type("StructA", TypeEntrails::Struct)
            .expect("insert_global_type should succeed");
        assert_eq!(0, struct_id.0);

        // Member scope of new struct should have been added in `insert_global_type`
        assert_eq!(2, symbol_table.scopes.len());
        // New scope should have global scope as parent
        assert_eq!(
            Some(SymbolTable::GLOBAL_SCOPE),
            symbol_table.scope(ScopeId(1)).parent
        );

        if let SymbolKind::Type(struct_type) = &symbol_table.symbol(struct_id).kind() {
            let symbol = symbol_table.symbol(struct_id);
            assert_eq!(0, symbol.my_id().0);
            assert_eq!(1, symbol.name_id().1);
            assert_eq!(
                Some(SymbolTable::GLOBAL_SCOPE),
                symbol_table.scope(struct_type.member_scope).parent
            );
            assert_eq!(1, struct_type.member_scope.0);
            // UNNAMED_TYPE's pseudo name '' is already inserted
            assert_eq!(1, symbol_table.scope(struct_type.member_scope).names.len());
            assert_eq!("StructA", symbol_table.name(symbol.name_id()));
        } else {
            panic!("StructA should be a type!");
        }

        let fn_id = symbol_table
            .insert_type(
                struct_id,
                "new",
                TypeEntrails::Function(Function::default()),
            )
            .expect("insert_type should succeed");

        if let SymbolKind::Type(struct_type) = &symbol_table.symbol(struct_id).kind() {
            assert_eq!(2, symbol_table.scope(struct_type.member_scope).names.len());
        } else {
            panic!("StructA should be a type!");
        }

        // Member scope of new function should have been added in `insert_type`
        assert_eq!(3, symbol_table.scopes.len());

        if let SymbolKind::Type(fn_type) = &symbol_table.symbol(fn_id).kind() {
            let symbol = symbol_table.symbol(fn_id);
            assert_eq!(1, symbol.my_id().0);
            assert_eq!(1, symbol.name_id().1);
            assert_eq!(
                Some(ScopeId(1)),
                symbol_table.scope(fn_type.member_scope).parent
            );
            assert_eq!(2, fn_type.member_scope.0);
            assert_eq!(1, symbol_table.scope(fn_type.member_scope).names.len());
            assert_eq!("new", symbol_table.name(symbol.name_id()));
        };
    }
}
