use parol_runtime::TerminalIndex;

use crate::analysis::compiled_terminal::{EPS, INVALID};
use crate::{CompiledTerminal, MAX_K};
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::{Hash, Hasher};

const EOI: TerminalIndex = 0;
const NEW_LINE: TerminalIndex = 1;
const WHITESPACE: TerminalIndex = 2;
const LINE_COMMENT: TerminalIndex = 3;
const BLOCK_COMMENT: TerminalIndex = 4;

/// Common functions needed for terminal handling
pub trait TerminalMappings<T> {
    /// Create an epsilon representation
    fn eps() -> T;
    /// Create an end-of-input representation
    fn end() -> T;
    /// Check for epsilon
    fn is_eps(&self) -> bool;
    /// Check for end-of-input
    fn is_end(&self) -> bool;
    /// Check for invalid (i.e. unassigned) terminal
    fn is_inv(&self) -> bool;
}

/// When storing MAX_K terminals in 128 bits, the maximum number of bits used per terminal is 12.
const MAX_BITS: u8 = 12;

/// A collection of terminals
///
/// The terminals are stored in a 128 bit integer where each terminal is stored in a fixed number of
/// bits. The number of bits is determined by the number of terminals to store.
/// The maximum number of terminals when storing MAX_K terminals in 128 bits is:
/// 128 / MAX_K = 128 / 10 = 12.8 => 12 bits
/// The maximum number of terminals that can be stored is 2^12 = 4096.
/// The maximum value of member bits is therefore 12 and can safely be stored in a u8.
/// We store a mask to more easily extract the terminals from the 128 bits unsigned integer.
/// The mask is calculated as 2^bits - 1 that is equivalent to the expression !(!0u128 << bits).
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq)]
pub struct Terminals {
    // The terminals
    pub(crate) t: u128,
    // A mask to extract the terminal at position i
    mask: u128,
    // The index of next insertion
    pub(crate) i: u8,
    // Number of bits used per terminal
    bits: u8,
}

impl Terminals {
    /// Creates a new item
    /// ```
    /// use parol::analysis::k_tuple::Terminals;
    /// use parol::analysis::compiled_terminal::CompiledTerminal;
    /// let t = Terminals::new();
    /// assert!(t.is_empty());
    /// assert_eq!(0, t.len(), "len");
    /// assert_eq!(0, t.k_len(5), "k_len");
    /// assert_eq!(CompiledTerminal::default(), t[0]);
    /// assert_eq!(CompiledTerminal::default(), t[9]);
    /// ```
    pub fn new(max_terminal_index: usize) -> Self {
        // max_terminal_index + 1: we also need to store EPS
        let bits = (max_terminal_index + 1).ilog2() as u8 + 1;
        if bits > MAX_BITS {
            panic!(
                "The number of bits required to store {} terminals is {} which is greater than the maximum of {}",
                max_terminal_index + 1, bits, MAX_BITS
            );
        }
        let mask = !(!0u128 << bits);
        Self {
            t: 0,
            i: 0,
            bits,
            mask,
        }
    }

    /// Creates a new item with epsilon semantic
    /// ```
    /// use parol::analysis::k_tuple::{Terminals, TerminalMappings};
    /// use parol::analysis::compiled_terminal::CompiledTerminal;
    /// let t = Terminals::eps();
    /// assert!(!t.is_empty());
    /// assert_eq!(1, t.len(), "len");
    /// assert_eq!(1, t.k_len(5), "k_len");
    /// assert_eq!(CompiledTerminal::eps(), t[0]);
    /// assert_eq!(CompiledTerminal::default(), t[1]);
    /// assert_eq!(CompiledTerminal::default(), t[9]);
    /// ```
    pub fn eps(max_terminal_index: usize) -> Terminals {
        let mut t = Self::new(max_terminal_index);
        t.t = t.mask;
        t.i = 1;
        t
    }

    /// Creates a new item with end (EOI) semantic
    /// Such a terminal can't be extended, i.e. you can't append more terminals
    /// ```
    /// use parol::analysis::k_tuple::{Terminals, TerminalMappings};
    /// use parol::analysis::compiled_terminal::CompiledTerminal;
    /// let t = Terminals::end();
    /// assert!(!t.is_empty());
    /// assert_eq!(1, t.len());
    /// assert_eq!(1, t.k_len(5));
    /// assert_eq!(CompiledTerminal::end(), t[0]);
    /// assert_eq!(CompiledTerminal::default(), t[1]);
    /// assert_eq!(CompiledTerminal::default(), t[9]);
    /// ```
    pub fn end(max_terminal_index: usize) -> Terminals {
        let mut t = Self::new(max_terminal_index);
        // t.t = 0; // EOI as u128 & t.mask;
        t.i = 1;
        t
    }

    ///
    /// Creates a new object with maximum k length from another object
    ///
    #[must_use]
    pub fn of(k: usize, other: Self) -> Self {
        let bits = other.bits;
        let mask = other.mask;
        let i = other.k_len(k) as u8;
        let mut copy_mask = 0u128;
        (0..i).for_each(|_| {
            copy_mask <<= bits as usize;
            copy_mask |= mask;
        });
        let t = other.t & copy_mask;
        Self { t, i, bits, mask }
    }

    /// Returns the length of the collection
    #[inline]
    pub fn len(&self) -> usize {
        self.i as usize
    }
    /// Checks if the collection is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.i == 0
    }

    #[must_use]
    fn last(&self) -> Option<CompiledTerminal> {
        if self.is_empty() {
            None
        } else {
            Some(self.get(self.i as usize - 1))
        }
    }

    /// Checks if the collection is k-complete, i.e. no terminals can be added
    /// ```
    /// use parol::analysis::k_tuple::Terminals;
    /// let t = Terminals::end();
    /// assert!(t.is_k_complete(5));
    /// ```
    pub fn is_k_complete(&self, k: usize) -> bool {
        !self.is_eps() && (self.len() >= k || self.last().map_or(false, |t| t.is_end()))
    }

    /// Returns the k-length, i.e. the number of symbols that contributes to lookahead sizes
    #[must_use]
    pub fn k_len(&self, k: usize) -> usize {
        let mut k_len = 0;
        let mut t_dup = self.t;
        for _i in 0..self.i {
            if k_len >= k {
                break;
            }
            k_len += 1;
            let t = t_dup & self.mask;
            if t == EOI as u128 {
                break;
            }
            t_dup >>= self.bits as usize;
        }
        k_len
    }

    /// Clears the collection
    pub fn clear(&mut self) {
        self.t = 0;
        self.i = 0;
    }

    /// Concatenates two collections with respect to the rules of k-concatenation
    pub fn k_concat(mut self, other: &Self, k: usize) -> Self {
        debug_assert!(
            other.bits == self.bits,
            "Bits must be the same, self:({:?}) != other:({:?})",
            self,
            other
        );
        if other.is_eps() {
            // w + ε = W
            return self;
        }

        if self.is_eps() {
            // ε + w = w
            // Remove possible epsilon terminal
            self.clear();
        }

        if self.is_k_complete(k) {
            // k: w would be the same as k: (w + x)
            return self;
        }

        let my_k_len = self.k_len(k);
        let to_take = other.k_len(k - my_k_len);
        let mask = !0u128 << (to_take * self.bits as usize);
        let value = (other.t & !mask) << (my_k_len * self.bits as usize);
        self.t &= !mask;
        self.t |= value;
        self.i = (my_k_len + to_take) as u8;
        self
    }

    /// Adds a new terminal to self if max size is not reached yet and if last is not EOI
    pub fn push(&mut self, t: CompiledTerminal) {
        if self.i >= MAX_K as u8 {
            panic!("Maximum number of terminals reached");
        }
        if matches!(self.last(), Some(CompiledTerminal(EOI))) {
            return;
        }
        debug_assert_ne!(t.0, INVALID, "Invalid terminal");
        self.set(self.i.into(), t);
        self.i += 1;
    }

    /// Checks if self is an Epsilon
    #[inline]
    pub fn is_eps(&self) -> bool {
        self.i == 1 && ((self.t & self.mask) == self.mask)
    }

    /// Creates an iterator over the terminals
    pub fn iter(&self) -> TermIt {
        TermIt::new(*self)
    }

    /// Returns the terminal at position i
    pub fn get(&self, i: usize) -> CompiledTerminal {
        let mut terminal_index = (self.t >> (i * self.bits as usize)) & self.mask;
        if terminal_index == self.mask {
            // Epsilon is defined as 0xFFFF and stored as a value identical to self.mask, i.e. all
            // bits set to 1. We need to convert it back to 0xFFFF.
            terminal_index = EPS as u128;
        }
        CompiledTerminal(terminal_index as TerminalIndex)
    }

    /// Sets the terminal at position i
    pub fn set(&mut self, i: usize, t: CompiledTerminal) {
        debug_assert!(
            t.0 <= self.mask as TerminalIndex || t.0 == EPS as TerminalIndex,
            "Terminal index {} out of range",
            t.0
        );
        debug_assert_ne!(t.0, INVALID, "Invalid terminal");
        let v = (t.0 as u128 & self.mask) << (i * self.bits as usize);
        let mask = !(self.mask << (i * self.bits as usize));
        self.t &= mask;
        self.t |= v;
    }
}

impl Ord for Terminals {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.i.cmp(&other.i) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => {
                <&Self as Into<u128>>::into(self).cmp(&<&Self as Into<u128>>::into(other))
            }
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for Terminals {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Terminals {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "[{}(i{})]",
            (0..self.i)
                .map(|i| format!("{}", self.get(i as usize)))
                .collect::<Vec<String>>()
                .join(", "),
            self.i,
        )
    }
}

// Used for comparison in implementation of Ord
impl From<&Terminals> for u128 {
    fn from(t: &Terminals) -> Self {
        // If the assertion never fails, the extra masking is not necessary
        debug_assert!(t.t & (!0u128 << (t.i * t.bits) as usize) == 0);
        // Mask out the unused bits although it should not be necessary
        t.t & !(!0u128 << (t.i * t.bits) as usize)
    }
}

impl Extend<CompiledTerminal> for Terminals {
    fn extend<I: IntoIterator<Item = CompiledTerminal>>(&mut self, iter: I) {
        for t in iter {
            self.push(t);
        }
    }
}

impl Extend<TerminalIndex> for Terminals {
    fn extend<I: IntoIterator<Item = TerminalIndex>>(&mut self, iter: I) {
        for t in iter {
            self.push(CompiledTerminal(t));
        }
    }
}

impl Debug for Terminals {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0b{:b}, i:{}, bits:0x{:x}, mask:0x{:x}",
            self.t, self.i, self.bits, self.mask
        )
    }
}

/// Iterator for Terminals
/// It returns the terminal indices
#[derive(Debug)]
pub struct TermIt {
    t: Terminals,
    i: usize,
}

impl TermIt {
    fn new(t: Terminals) -> Self {
        Self { t, i: 0 }
    }
}

impl Iterator for TermIt {
    type Item = TerminalIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.t.i as usize {
            let t = self.t.t & self.t.mask;
            self.t.t >>= self.t.bits as usize;
            self.i += 1;
            if t == self.t.mask {
                // Epsilon is defined as 0xFFFF and stored as a value identical to self.mask, i.e.
                // all bits set to 1. We need to convert it back to 0xFFFF.
                Some(EPS)
            } else {
                Some(t as TerminalIndex)
            }
        } else {
            None
        }
    }
}

/// Terminal string with support for k-completeness
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum TerminalString {
    /// Incomplete sequence
    Incomplete(Terminals),
    /// k-complete sequence
    Complete(Terminals),
}

impl TerminalString {
    /// Returns the length of the sequence
    pub fn len(&self) -> usize {
        self.inner().len()
    }
    /// Checks if the sequence is empty
    pub fn is_empty(&self) -> bool {
        self.inner().is_empty()
    }

    /// Checks if the sequence is k-complete
    pub fn is_k_complete(&self) -> bool {
        match self {
            Self::Incomplete(_) => false,
            Self::Complete(_) => true,
        }
    }

    /// Checks if the inner sequence is k-complete
    pub fn is_complete(&self, k: usize) -> bool {
        self.inner().is_k_complete(k)
    }

    /// Change the state to k-complete
    pub fn make_complete(self) -> Self {
        if let Self::Incomplete(e) = self {
            Self::Complete(e)
        } else {
            self
        }
    }

    /// Revoke the k-complete state
    pub fn make_incomplete(self) -> Self {
        if let Self::Complete(e) = self {
            Self::Incomplete(e)
        } else {
            self
        }
    }

    /// Clear the sequences
    pub fn clear(self) -> Self {
        let mut inner = match self {
            Self::Incomplete(t) | Self::Complete(t) => t,
        };
        inner.clear();

        Self::Incomplete(inner)
    }

    /// Return the inner sequences
    pub fn inner(&self) -> &Terminals {
        match self {
            Self::Incomplete(v) => v,
            Self::Complete(v) => v,
        }
    }

    /// Checks if self is an Epsilon
    pub fn is_eps(&self) -> bool {
        match self {
            Self::Incomplete(v) => v.is_eps(),
            Self::Complete(_) => false,
        }
    }

    /// Push a new terminal
    pub fn push(&mut self, t: CompiledTerminal, k: usize) {
        match self {
            Self::Incomplete(v) => {
                v.push(t);
                if v.is_k_complete(k) {
                    *self = Self::Complete(*v);
                }
            }
            Self::Complete(_) => {}
        }
    }

    /// Concat self with another sequence while consuming self
    pub fn k_concat(self, other: &Self, k: usize) -> Self {
        match self {
            Self::Incomplete(v) => {
                let terminals = v.k_concat(other.inner(), k);
                if terminals.is_k_complete(k) {
                    TerminalString::Complete(terminals)
                } else {
                    TerminalString::Incomplete(terminals)
                }
            }
            Self::Complete(_) => self,
        }
    }
}

impl Display for TerminalString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Incomplete(v) => write!(f, "Incomplete({})", v),
            Self::Complete(v) => write!(f, "Complete  ({})", v),
        }
    }
}

impl Debug for TerminalString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Incomplete(v) => write!(f, "Incomplete({:?})", v),
            Self::Complete(v) => write!(f, "Complete  ({:?})", v),
        }
    }
}

/// A builder for KTuple
#[derive(Clone, Default)]
pub struct KTupleBuilder<'a> {
    k: Option<usize>,
    max_terminal_index: Option<usize>,
    k_tuple: Option<&'a KTuple>,
    terminal_string: Option<&'a [TerminalIndex]>,
}

impl<'a> KTupleBuilder<'a> {
    /// Creates a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the lookahead size
    pub fn k(mut self, k: usize) -> Self {
        self.k = Some(k);
        self
    }

    /// Sets the maximum terminal index
    pub fn max_terminal_index(mut self, max_terminal_index: usize) -> Self {
        self.max_terminal_index = Some(max_terminal_index);
        self
    }

    /// Sets the k-tuple to be used during construction
    pub fn k_tuple(mut self, k_tuple: &'a KTuple) -> Self {
        self.k_tuple = Some(k_tuple);
        self
    }

    /// Sets the terminal string to be used during construction
    pub fn terminal_string(mut self, terminal_string: &'a [TerminalIndex]) -> Self {
        self.terminal_string = Some(terminal_string);
        self
    }

    /// Builds a new KTuple
    pub fn build(self) -> Result<KTuple, String> {
        if self.k.is_none() {
            return Err("k is not set".to_owned());
        }
        if self.max_terminal_index.is_none() {
            return Err("max_terminal_index is not set".to_owned());
        }
        let k = self.k.unwrap_or(0);
        let max_terminal_index = self.max_terminal_index.unwrap_or(0);
        if let Some(k_tuple) = self.k_tuple {
            let mut terminals = Terminals::new(max_terminal_index);
            for t in k_tuple.terminals.inner().iter().take(k) {
                terminals.push(CompiledTerminal(t));
            }
            let terminals = if terminals.is_k_complete(k) {
                TerminalString::Complete(terminals)
            } else {
                TerminalString::Incomplete(terminals)
            };
            Ok(KTuple {
                terminals,
                k: std::cmp::min(k, MAX_K),
            })
        } else if let Some(terminal_string) = self.terminal_string {
            let mut terminals = Terminals::new(max_terminal_index);
            for t in terminal_string.iter().take(k) {
                terminals.push(CompiledTerminal(*t));
            }
            let terminals = if terminals.is_k_complete(k) {
                TerminalString::Complete(terminals)
            } else {
                TerminalString::Incomplete(terminals)
            };
            Ok(KTuple {
                terminals,
                k: std::cmp::min(k, MAX_K),
            })
        } else {
            Err("k_tuple or terminal_string must be set".to_owned())
        }
    }

    ///
    /// Creates a new ε object
    ///
    pub fn eps(self) -> Result<KTuple, String> {
        if self.k.is_none() {
            return Err("k is not set".to_owned());
        }
        if self.max_terminal_index.is_none() {
            return Err("max_terminal_index is not set".to_owned());
        }
        let terminals =
            TerminalString::Incomplete(Terminals::eps(self.max_terminal_index.unwrap()));
        Ok(KTuple {
            terminals,
            k: self.k.unwrap(),
        })
    }
    ///
    /// Creates a new End object
    ///
    pub fn end(self) -> Result<KTuple, String> {
        if self.k.is_none() {
            return Err("k is not set".to_owned());
        }
        if self.max_terminal_index.is_none() {
            return Err("max_terminal_index is not set".to_owned());
        }
        let terminals = TerminalString::Complete(Terminals::end(self.max_terminal_index.unwrap()));
        Ok(KTuple {
            terminals,
            k: self.k.unwrap(),
        })
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Terminal symbol string type
///
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct KTuple {
    /// The sequence of terminals
    terminals: TerminalString,
    /// The lookahead size
    k: usize,
}

impl KTuple {
    /// Used for debugging only
    pub fn with_terminal_indices(self, terms: &[TerminalIndex]) -> Self {
        let k = self.k;
        let mut terminals = match self.terminals {
            TerminalString::Incomplete(s) => s,
            TerminalString::Complete(s) => s,
        };

        terms.iter().take(k).enumerate().for_each(|(i, t)| {
            terminals.set(i, CompiledTerminal(*t));
            terminals.i += 1;
        });

        let terminals = if terminals.is_k_complete(k) {
            TerminalString::Complete(terminals)
        } else {
            TerminalString::Incomplete(terminals)
        };

        Self { terminals, k }
    }

    ///
    /// Creates a new object from a slice of CompiledTerminals
    ///
    pub fn from_slice(others: &[CompiledTerminal], k: usize, max_terminal_index: usize) -> Self {
        let mut terminals = Terminals::new(max_terminal_index);
        terminals.extend(others.iter().take(k).cloned());
        let terminals = if terminals.is_k_complete(k) {
            TerminalString::Complete(terminals)
        } else {
            TerminalString::Incomplete(terminals)
        };
        Self { terminals, k }
    }

    ///
    /// Creates a new object from a vector of terminal symbols
    ///
    pub fn of(t: Terminals, k: usize) -> Self {
        let terminals = Terminals::of(k, t);

        let terminals = if terminals.is_k_complete(k) {
            TerminalString::Complete(terminals)
        } else {
            TerminalString::Incomplete(terminals)
        };
        Self { terminals, k }
    }

    /// Adds a new terminal to self while consuming self
    pub fn push(&mut self, t: CompiledTerminal) {
        self.terminals.push(t, self.k)
    }

    /// Checks if self is an Epsilon
    pub fn is_eps(&self) -> bool {
        self.terminals.is_eps()
    }
    /// Returns the length of the sequence
    pub fn len(&self) -> usize {
        self.terminals.len()
    }
    /// Checks if the sequence is empty
    pub fn is_empty(&self) -> bool {
        self.terminals.is_empty()
    }
    /// Returns the k-length of the sequence
    pub fn k_len(&self, k: usize) -> usize {
        self.terminals.inner().k_len(k)
    }
    /// Checks if the sequence is k-complete
    pub fn is_k_complete(&self) -> bool {
        self.terminals.is_k_complete()
    }

    /// Concat self with another sequence while consuming self
    pub fn k_concat(self, other: &Self, k: usize) -> Self {
        let terminals = self.terminals.k_concat(&other.terminals, k);
        let k = terminals.inner().k_len(k);
        Self { terminals, k }
    }

    /// Sets the lookahead size
    pub fn set_k(mut self, k: usize) -> Self {
        if self.terminals.is_complete(k) {
            self.terminals = self.terminals.make_complete();
        } else {
            self.terminals = self.terminals.make_incomplete();
        }
        self.k = k;
        self
    }

    /// Conversion to string with the help of the terminals slice
    pub fn to_string(&self, terminals: &[String]) -> String {
        format!(
            "[{}]",
            self.terminals
                .inner()
                .iter()
                .map(|t| match t {
                    EOI => "$".to_owned(),
                    NEW_LINE => "NewLine".to_owned(),
                    WHITESPACE => "WhiteSpace".to_owned(),
                    LINE_COMMENT => "LineComment".to_owned(),
                    BLOCK_COMMENT => "BlockComment".to_owned(),
                    EPS => "\u{03B5}".to_owned(),
                    _ => terminals[t as usize].to_string(),
                })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    /// Returns the k value
    #[inline]
    pub fn k(&self) -> usize {
        self.k
    }

    /// Returns the terminals
    #[inline]
    pub fn terminals(&self) -> &Terminals {
        self.terminals.inner()
    }
}

impl Debug for KTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "[{:?}(i{})](k{})",
            self.terminals,
            self.terminals.inner().i,
            self.k
        )
    }
}

impl Display for KTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "[{}(i{})](k{})",
            self.terminals,
            self.terminals.inner().i,
            self.k
        )
    }
}

impl Hash for KTuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let self_inner = self.terminals.inner();
        self_inner.t.hash(state)
    }
}

impl Extend<CompiledTerminal> for KTuple {
    fn extend<I: IntoIterator<Item = CompiledTerminal>>(&mut self, iter: I) {
        if !self.terminals.is_k_complete() {
            for t in iter.into_iter().take(self.k - self.len()) {
                self.push(t);
            }
        }
    }
}

impl Extend<TerminalIndex> for KTuple {
    fn extend<I: IntoIterator<Item = TerminalIndex>>(&mut self, iter: I) {
        if !self.terminals.is_k_complete() {
            for t in iter.into_iter().take(self.k - self.len()) {
                self.push(CompiledTerminal(t));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use parol_runtime::TerminalIndex;

    use super::{TerminalString, Terminals};
    use crate::{
        analysis::k_tuple::{KTupleBuilder, EOI},
        CompiledTerminal, KTuple, MAX_K,
    };

    fn term(terminals: &[TerminalIndex], k: usize, max_terminal_index: usize) -> Terminals {
        debug_assert!(k <= MAX_K);
        let mut t = Terminals::new(max_terminal_index);
        t.extend(terminals.iter().map(|t| CompiledTerminal(*t)));
        t
    }

    #[test]
    fn check_with_terminal_indices() {
        {
            let k_tuple = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .terminal_string(&[1])
                .build()
                .unwrap();
            let t = term(&[1], 1, 1);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: 1,
            };
            assert_eq!(CompiledTerminal::default(), t.get(1));
            assert_eq!(CompiledTerminal::default(), t.get(MAX_K - 1));
            assert_eq!(expected, k_tuple, "[1]");
        }
        {
            let k_tuple = KTupleBuilder::new()
                .k(MAX_K)
                .max_terminal_index(10)
                .terminal_string(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
                .build()
                .unwrap();
            let t = term(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], MAX_K, 10);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: MAX_K,
            };
            assert_eq!(CompiledTerminal(1), t.get(0));
            assert_eq!(CompiledTerminal(10), t.get(MAX_K - 1));
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]");
        }
        {
            let k_tuple = KTupleBuilder::new()
                .k(5)
                .max_terminal_index(10)
                .terminal_string(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
                .build()
                .unwrap();
            let t = term(&[1, 2, 3, 4, 5], 5, 10);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: 5,
            };
            assert_eq!(CompiledTerminal(1), t.get(0));
            assert_eq!(CompiledTerminal(5), t.get(4));
            assert_eq!(CompiledTerminal::default(), t.get(5));
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5]");
        }
    }

    #[test]
    fn check_from_slice() {
        {
            let k_tuple = KTuple::from_slice(&[CompiledTerminal(1)], 1, 1);
            let t = term(&[1], 1, 1);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: 1,
            };
            assert_eq!(CompiledTerminal::default(), t.get(1));
            assert_eq!(CompiledTerminal::default(), t.get(MAX_K - 1));
            assert_eq!(expected, k_tuple, "[1]");
        }
        {
            let k_tuple = KTuple::from_slice(
                &[
                    CompiledTerminal(1),
                    CompiledTerminal(2),
                    CompiledTerminal(3),
                    CompiledTerminal(4),
                    CompiledTerminal(5),
                    CompiledTerminal(6),
                    CompiledTerminal(7),
                    CompiledTerminal(8),
                    CompiledTerminal(9),
                    CompiledTerminal(10),
                ],
                MAX_K,
                10,
            );
            let t = term(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], MAX_K, 10);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: MAX_K,
            };
            assert_eq!(CompiledTerminal(1), t.get(0));
            assert_eq!(CompiledTerminal(10), t.get(MAX_K - 1));
            assert_eq!(expected, k_tuple);
        }
        {
            let k_tuple = KTuple::from_slice(
                &[
                    CompiledTerminal(1),
                    CompiledTerminal(2),
                    CompiledTerminal(3),
                    CompiledTerminal(4),
                    CompiledTerminal(5),
                    CompiledTerminal(6),
                    CompiledTerminal(7),
                    CompiledTerminal(8),
                    CompiledTerminal(9),
                    CompiledTerminal(10),
                ],
                5,
                10,
            );
            let t = term(&[1, 2, 3, 4, 5], 5, 10);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: 5,
            };
            assert_eq!(CompiledTerminal(1), t.get(0));
            assert_eq!(CompiledTerminal(5), t.get(4));
            assert_eq!(CompiledTerminal::default(), t.get(5));
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5]");
        }
    }

    #[test]
    fn check_k_tuple_of() {
        {
            let k = 1;
            let mut t = Terminals::new(1);
            t.extend([1]);
            let k_tuple = KTuple::of(t, k);
            let mut t2 = Terminals::new(1);
            t2.extend([1]);
            let expected = KTuple {
                terminals: TerminalString::Complete(t2),
                k,
            };
            assert_eq!(CompiledTerminal::default(), t.get(1));
            assert_eq!(CompiledTerminal::default(), t.get(MAX_K - 1));
            assert_eq!(expected, k_tuple, "[1]");
        }
        {
            let k = MAX_K;
            let mut t = Terminals::new(11);
            t.extend([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            let k_tuple = KTuple::of(t, k);
            assert_eq!(MAX_K, k_tuple.len());
            let mut t2 = Terminals::new(11);
            t2.extend([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            let expected = KTuple {
                terminals: TerminalString::Complete(t2),
                k,
            };
            assert_eq!(CompiledTerminal(1), t.get(0));
            assert_eq!(CompiledTerminal(10), t.get(MAX_K - 1));
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]");
        }
        {
            let k = 5;
            let mut t = Terminals::new(11);
            t.extend([1, 2, 3, 4, 5]);

            let k_tuple = KTuple::of(t, k);
            let mut t2 = Terminals::new(11);
            t2.extend([1, 2, 3, 4, 5]);
            let expected = KTuple {
                terminals: TerminalString::Complete(t2),
                k,
            };
            assert_eq!(CompiledTerminal(1), t.get(0));
            assert_eq!(CompiledTerminal(5), t.get(4));
            assert_eq!(CompiledTerminal::default(), t.get(5));
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5]");
        }
    }

    #[test]
    fn check_k_concat() {
        {
            let tuple1 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .eps()
                .unwrap();
            let tuple2 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .eps()
                .unwrap();
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .eps()
                .unwrap();
            assert_eq!(expected, result, "1: [ε] + [ε] = [ε]");
        }
        {
            let tuple1 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .terminal_string(&[1])
                .build()
                .unwrap();
            let tuple2 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .eps()
                .unwrap();
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .terminal_string(&[1])
                .build()
                .unwrap();
            assert_eq!(expected, result, "1: [a] + [ε] = [a]");
        }
        {
            let tuple1 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .eps()
                .unwrap();
            let tuple2 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .terminal_string(&[1])
                .build()
                .unwrap();
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .terminal_string(&[1])
                .build()
                .unwrap();
            assert_eq!(expected, result, "1: [ε] + [a] = [a]");
        }
        {
            let tuple1 = KTupleBuilder::new()
                .k(2)
                .max_terminal_index(2)
                .terminal_string(&[1])
                .build()
                .unwrap();
            let tuple2 = KTupleBuilder::new()
                .k(2)
                .max_terminal_index(2)
                .terminal_string(&[2])
                .build()
                .unwrap();
            let result = tuple1.k_concat(&tuple2, 2);
            let expected = KTupleBuilder::new()
                .k(2)
                .max_terminal_index(1)
                .terminal_string(&[1, 2])
                .build()
                .unwrap();
            assert_eq!(expected, result, "2: [a] + [b] = [ab]");
        }
    }

    #[test]
    fn check_term() {
        {
            let terminals = Terminals::new(4);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(0, terminals.k_len(1));
            assert_eq!(0, terminals.k_len(2));

            assert!(terminals.is_k_complete(0));
            assert!(!terminals.is_k_complete(1));
            assert!(!terminals.is_k_complete(2));
            assert!(!terminals.is_k_complete(3));
        }
        {
            let terminals = term(&[1], 1, 4);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(1, terminals.k_len(2));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(!terminals.is_k_complete(2));
            assert!(!terminals.is_k_complete(3));
        }
        {
            let terminals = term(&[1, 2], 2, 4);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(2, terminals.k_len(2));
            assert_eq!(2, terminals.k_len(3));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(terminals.is_k_complete(2));
            assert!(!terminals.is_k_complete(3));
        }
        {
            let terminals = term(&[1, EOI], 2, 4);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(2, terminals.k_len(2));
            assert_eq!(2, terminals.k_len(3));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(terminals.is_k_complete(2));
            assert!(terminals.is_k_complete(3));
        }
        {
            let terminals = term(
                &[
                    1, EOI, 1, // This constellation is actually illegal!
                ],
                3,
                1,
            );
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(2, terminals.k_len(2));
            assert_eq!(2, terminals.k_len(3));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(terminals.is_k_complete(2));
            assert!(terminals.is_k_complete(3));

            let terminals2 = term(&[3], 1, 1);
            let result = terminals.k_concat(&terminals2, 3);
            let expected = term(&[1, EOI, 1], 3, 1);
            assert_eq!(expected, result);
        }
    }

    #[test]
    fn test_iteration_of_terminals() {
        let terminals = term(&[1, 2, 3, 4, 5], 5, 11);
        let mut iter = terminals.iter();
        assert_eq!(Some(1), iter.next());
        assert_eq!(Some(2), iter.next());
        assert_eq!(Some(3), iter.next());
        assert_eq!(Some(4), iter.next());
        assert_eq!(Some(5), iter.next());
        assert_eq!(None, iter.next());
    }
}
