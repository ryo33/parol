use crate::oberon_0_grammar_trait::Oberon0GrammarTrait;
use parol_runtime::log::trace;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure used to build up a oberon_0 structure item during parsing
///
#[derive(Debug, Clone)]
pub enum Oberon0GrammarItem {}

impl Display for Oberon0GrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "TODO!")
    }
}

///
/// Data structure used to build up a oberon_0 structure during parsing
///
#[derive(Debug, Default)]
pub struct Oberon0Grammar<'t> {
    pub item_stack: Vec<Oberon0GrammarItem>,
    // Just to hold the lifetime generated by parol
    phantom: std::marker::PhantomData<&'t str>,
}

impl Oberon0Grammar<'_> {
    pub fn new() -> Self {
        Oberon0Grammar::default()
    }

    fn _push(&mut self, item: Oberon0GrammarItem, context: &str) {
        trace!("push    {}: {}", context, item);
        self.item_stack.push(item)
    }

    fn _pop(&mut self, context: &str) -> Option<Oberon0GrammarItem> {
        if !self.item_stack.is_empty() {
            let item = self.item_stack.pop();
            if let Some(ref item) = item {
                trace!("pop     {}: {}", context, item);
            }
            item
        } else {
            None
        }
    }
}

impl Display for Oberon0Grammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f)
    }
}

impl Oberon0GrammarTrait<'_> for Oberon0Grammar<'_> {}
