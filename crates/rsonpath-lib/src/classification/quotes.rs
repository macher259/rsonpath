//! Classification of bytes withing JSON quote sequences.
//!
//! Provides the [`QuoteClassifiedBlock`] struct and [`QuoteClassifiedIterator`] trait
//! that allow effectively enriching JSON inputs with quote sequence information.
//!
//! The output of quote classification is an iterator of [`QuoteClassifiedBlock`]
//! which contain bitmasks whose lit bits signify characters that are within quotes
//! in the source document. These characters need to be ignored.
//!
//! Note that the actual quote characters are not guaranteed to be classified
//! as "within themselves" or otherwise. In particular the current implementation
//! marks _opening_ quotes with lit bits, but _closing_ quotes are always unmarked.
//! This behavior should not be presumed to be stable, though, and can change
//! without a major semver bump.
use crate::{
    input::{error::InputError, InputBlock, InputBlockIterator},
    FallibleIterator, MaskType, BLOCK_SIZE,
};

/// Result of the [`FallibleIterator`] for quote classification,
/// and of the [`offset`](`QuoteClassifiedIterator::offset`) function.
pub type QuoteIterResult<I, M> = Result<Option<QuoteClassifiedBlock<I, M>>, InputError>;

/// Input block with a bitmask signifying which characters are within quotes.
///
/// Characters within quotes in the input are guaranteed to have their corresponding
/// bit in `within_quotes_mask` set. The $0$-th bit of the mask corresponds to the
/// last character in `block`, the $1$-st bit to the second-to-last character, etc.
///
/// There is no guarantee on how the boundary quote characters are classified,
/// their bits might be lit or not lit depending on the implementation.
pub struct QuoteClassifiedBlock<B, M> {
    /// The block that was classified.
    pub block: B,
    /// Mask marking characters within a quoted sequence.
    pub within_quotes_mask: M,
}

/// Result of resuming quote classification, the resulting iterator
/// and optionally the first block (already quote classified).
pub struct ResumedQuoteClassifier<Q, B, M> {
    /// Resumed iterator.
    pub classifier: Q,
    /// Optional first quote classified block.
    pub first_block: Option<QuoteClassifiedBlock<B, M>>,
}

/// Trait for quote classifier iterators, i.e. finite iterators
/// enriching blocks of input with quote bitmasks.
/// Iterator is allowed to hold a reference to the JSON document valid for `'a`.
pub trait QuoteClassifiedIterator<'i, I: InputBlockIterator<'i>, M>:
    FallibleIterator<Item = QuoteClassifiedBlock<I::Block, M>, Error = InputError>
{
    /// Get the total offset in bytes from the beginning of input.
    fn get_offset(&self) -> usize;

    /// Move the iterator `count` blocks forward.
    ///
    /// # Errors
    /// At least one new block is read from the underlying
    /// [`InputBlockIterator`] implementation, which can fail.
    fn offset(&mut self, count: isize) -> QuoteIterResult<I::Block, M>;

    /// Flip the bit representing whether the last block ended with a nonescaped quote.
    ///
    /// This should be done only in very specific circumstances where the previous-block
    /// state could have been damaged due to stopping and resuming the classification at a later point.
    fn flip_quotes_bit(&mut self);

    /// Release any unneeded memory.
    ///
    /// This is a no-op for most implementations, but can be used
    /// to release memory in streaming implementations.
    fn release_memory(&mut self);
}

/// Higher-level classifier that can be consumed to retrieve the inner
/// [`Input::BlockIterator`](crate::input::Input::BlockIterator).
pub trait InnerIter<I> {
    /// Consume `self` and return the wrapped [`Input::BlockIterator`](crate::input::Input::BlockIterator).
    fn into_inner(self) -> I;
}

impl<'i, B, M> QuoteClassifiedBlock<B, M>
where
    B: InputBlock<'i>,
{
    /// Returns the length of the classified block.
    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.block.len()
    }

    /// Whether the classified block is empty.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.block.is_empty()
    }
}

pub(crate) mod nosimd;
pub(crate) mod shared;

#[cfg(target_arch = "x86")]
pub(crate) mod avx2_32;
#[cfg(target_arch = "x86_64")]
pub(crate) mod avx2_64;
#[cfg(target_arch = "x86")]
pub(crate) mod sse2_32;
#[cfg(target_arch = "x86_64")]
pub(crate) mod sse2_64;

pub(crate) trait QuotesImpl {
    type Classifier<'i, I>: QuoteClassifiedIterator<'i, I, MaskType> + InnerIter<I>
    where
        I: InputBlockIterator<'i>;

    fn new<'i, I>(iter: I) -> Self::Classifier<'i, I>
    where
        I: InputBlockIterator<'i>;

    fn resume<'i, I>(
        iter: I,
        first_block: Option<I::Block>,
    ) -> ResumedQuoteClassifier<Self::Classifier<'i, I>, I::Block, MaskType>
    where
        I: InputBlockIterator<'i>;
}
