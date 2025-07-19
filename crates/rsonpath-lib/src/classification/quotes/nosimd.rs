use super::*;
use crate::{debug, input::error::InputErrorConvertible};
use std::marker::PhantomData;

pub(crate) struct Constructor;

impl QuotesImpl for Constructor {
    type Classifier<'i, I>
        = SequentialQuoteClassifier<'i, I>
    where
        I: InputBlockIterator<'i>;

    #[inline(always)]
    #[allow(dead_code)]
    fn new<'i, I>(iter: I) -> Self::Classifier<'i, I>
    where
        I: InputBlockIterator<'i>,
    {
        SequentialQuoteClassifier {
            iter,
            escaped: false,
            in_quotes: false,
            phantom: PhantomData,
        }
    }

    fn resume<'i, I>(
        iter: I,
        first_block: Option<I::Block>,
    ) -> ResumedQuoteClassifier<Self::Classifier<'i, I>, I::Block, MaskType>
    where
        I: InputBlockIterator<'i>,
    {
        let mut s = SequentialQuoteClassifier {
            iter,
            escaped: false,
            in_quotes: false,
            phantom: PhantomData,
        };

        let block = first_block.map(|b| s.classify_block(b));

        ResumedQuoteClassifier {
            classifier: s,
            first_block: block,
        }
    }
}

pub(crate) struct SequentialQuoteClassifier<'i, I>
where
    I: InputBlockIterator<'i>,
{
    iter: I,
    escaped: bool,
    in_quotes: bool,
    phantom: PhantomData<&'i ()>,
}

impl<'i, I> SequentialQuoteClassifier<'i, I>
where
    I: InputBlockIterator<'i>,
{
    fn classify_block(&mut self, block: I::Block) -> QuoteClassifiedBlock<I::Block, MaskType> {
        let mut mask: MaskType = 0;
        let mut idx_mask = 1;

        for character in block.iter().copied() {
            if !self.escaped && character == b'"' {
                self.in_quotes = !self.in_quotes;
            }

            if character == b'\\' {
                self.escaped = !self.escaped;
            } else {
                self.escaped = false;
            }

            if self.in_quotes {
                mask |= idx_mask;
            }

            idx_mask <<= 1;
        }

        QuoteClassifiedBlock {
            block,
            within_quotes_mask: mask,
        }
    }
}

impl<'i, I> FallibleIterator for SequentialQuoteClassifier<'i, I>
where
    I: InputBlockIterator<'i>,
{
    type Item = QuoteClassifiedBlock<I::Block, MaskType>;
    type Error = InputError;

    #[inline(always)]
    fn next(&mut self) -> Result<Option<Self::Item>, InputError> {
        match self.iter.next().e()? {
            Some(block) => Ok(Some(self.classify_block(block))),
            None => Ok(None),
        }
    }
}

impl<'i, I> InnerIter<I> for SequentialQuoteClassifier<'i, I>
where
    I: InputBlockIterator<'i>,
{
    fn into_inner(self) -> I {
        self.iter
    }
}

impl<'i, I> QuoteClassifiedIterator<'i, I, MaskType> for SequentialQuoteClassifier<'i, I>
where
    I: InputBlockIterator<'i>,
{
    fn get_offset(&self) -> usize {
        self.iter.get_offset() - BLOCK_SIZE
    }

    fn offset(&mut self, count: isize) -> QuoteIterResult<I::Block, MaskType> {
        debug_assert!(count > 0);
        debug!("Offsetting by {count}");

        for _ in 0..count - 1 {
            self.iter.next().e()?;
        }

        self.next()
    }

    fn flip_quotes_bit(&mut self) {
        self.in_quotes = !self.in_quotes;
    }
}
