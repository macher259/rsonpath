use super::*;
use crate::classification::{quotes::QuoteClassifiedBlock, ResumeClassifierBlockState};
use crate::debug;

pub(crate) struct Constructor;

impl StructuralImpl for Constructor {
    type Classifier<'i, I, Q>
        = SequentialClassifier<'i, I, Q>
    where
        I: InputBlockIterator<'i>,
        Q: QuoteClassifiedIterator<'i, I, MaskType>;

    #[inline(always)]
    #[allow(dead_code)]
    fn new<'i, I, Q>(iter: Q) -> Self::Classifier<'i, I, Q>
    where
        I: InputBlockIterator<'i>,
        Q: QuoteClassifiedIterator<'i, I, MaskType>,
    {
        Self::Classifier {
            iter,
            block: None,
            are_colons_on: false,
            are_commas_on: false,
        }
    }
}

struct Block<'i, I>
where
    I: InputBlockIterator<'i>,
{
    quote_classified: QuoteClassifiedBlock<I::Block, MaskType>,
    idx: usize,
    are_colons_on: bool,
    are_commas_on: bool,
}

impl<'i, I> Block<'i, I>
where
    I: InputBlockIterator<'i>,
{
    fn new(
        quote_classified_block: QuoteClassifiedBlock<I::Block, MaskType>,
        are_colons_on: bool,
        are_commas_on: bool,
    ) -> Self {
        Self {
            quote_classified: quote_classified_block,
            idx: 0,
            are_colons_on,
            are_commas_on,
        }
    }

    fn from_idx(
        quote_classified_block: QuoteClassifiedBlock<I::Block, MaskType>,
        idx: usize,
        are_colons_on: bool,
        are_commas_on: bool,
    ) -> Self {
        Self {
            quote_classified: quote_classified_block,
            idx,
            are_colons_on,
            are_commas_on,
        }
    }
}

impl<'i, I> Iterator for Block<'i, I>
where
    I: InputBlockIterator<'i>,
{
    type Item = Structural;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.quote_classified.block.len() {
            let character = self.quote_classified.block[self.idx];
            let idx_mask = 1 << self.idx;
            let is_quoted = (self.quote_classified.within_quotes_mask & idx_mask) == idx_mask;

            let structural = match character {
                _ if is_quoted => None,
                b':' if self.are_colons_on => Some(Colon(self.idx)),
                b'{' => Some(Opening(BracketType::Curly, self.idx)),
                b'[' => Some(Opening(BracketType::Square, self.idx)),
                b',' if self.are_commas_on => Some(Comma(self.idx)),
                b'}' => Some(Closing(BracketType::Curly, self.idx)),
                b']' => Some(Closing(BracketType::Square, self.idx)),
                _ => None,
            };

            self.idx += 1;

            if structural.is_some() {
                return structural;
            };
        }

        None
    }
}

pub(crate) struct SequentialClassifier<'i, I, Q>
where
    I: InputBlockIterator<'i>,
{
    iter: Q,
    block: Option<Block<'i, I>>,
    are_colons_on: bool,
    are_commas_on: bool,
}

impl<'i, I, Q> SequentialClassifier<'i, I, Q>
where
    I: InputBlockIterator<'i>,
    Q: QuoteClassifiedIterator<'i, I, MaskType>,
{
    #[inline]
    fn reclassify(&mut self, idx: usize) {
        if let Some(block) = self.block.take() {
            let quote_classified_block = block.quote_classified;
            let relevant_idx = idx + 1;
            let block_idx = (idx + 1) % BLOCK_SIZE;
            debug!("relevant_idx is {relevant_idx}.");

            if block_idx != 0 || relevant_idx == self.iter.get_offset() {
                let new_block = Block::from_idx(
                    quote_classified_block,
                    block_idx,
                    self.are_colons_on,
                    self.are_commas_on,
                );
                self.block = Some(new_block);
            }
        }
    }
}

impl<'i, I, Q> FallibleIterator for SequentialClassifier<'i, I, Q>
where
    I: InputBlockIterator<'i>,
    Q: QuoteClassifiedIterator<'i, I, MaskType>,
{
    type Item = Structural;
    type Error = InputError;

    #[inline(always)]
    fn next(&mut self) -> Result<Option<Structural>, InputError> {
        let mut item = self.block.as_mut().and_then(Iterator::next);

        while item.is_none() {
            match self.iter.next()? {
                Some(block) => {
                    let mut block = Block::new(block, self.are_colons_on, self.are_commas_on);
                    item = block.next();
                    self.block = Some(block);
                }
                None => return Ok(None),
            }
        }

        Ok(item.map(|x| x.offset(self.iter.get_offset())))
    }
}

impl<'i, I, Q> StructuralIterator<'i, I, Q, MaskType> for SequentialClassifier<'i, I, Q>
where
    I: InputBlockIterator<'i>,
    Q: QuoteClassifiedIterator<'i, I, MaskType>,
{
    fn turn_colons_and_commas_on(&mut self, idx: usize) {
        if !self.are_commas_on && !self.are_colons_on {
            self.are_commas_on = true;
            self.are_colons_on = true;
            debug!("Turning both commas and colons on at {idx}.");

            self.reclassify(idx);
        } else if !self.are_commas_on {
            self.turn_commas_on(idx);
        } else if !self.are_colons_on {
            self.turn_colons_on(idx);
        }
    }

    fn turn_colons_and_commas_off(&mut self) {
        if self.are_commas_on && self.are_colons_on {
            self.are_commas_on = false;
            self.are_colons_on = false;
            debug!("Turning both commas and colons off.");
        } else if self.are_commas_on {
            self.turn_commas_off();
        } else if self.are_colons_on {
            self.turn_colons_off();
        }
    }

    fn turn_commas_on(&mut self, idx: usize) {
        if !self.are_commas_on {
            self.are_commas_on = true;
            debug!("Turning commas on at {idx}.");

            self.reclassify(idx);
        }
    }

    fn turn_commas_off(&mut self) {
        self.are_commas_on = false;
        debug!("Turning commas off.");
    }

    fn turn_colons_on(&mut self, idx: usize) {
        if !self.are_colons_on {
            self.are_colons_on = true;
            debug!("Turning colons on at {idx}.");

            self.reclassify(idx);
        }
    }

    fn turn_colons_off(&mut self) {
        self.are_colons_on = false;
        debug!("Turning colons off.");
    }

    fn stop(self) -> ResumeClassifierState<'i, I, Q, MaskType> {
        let block = self.block.map(|b| ResumeClassifierBlockState {
            block: b.quote_classified,
            idx: b.idx,
        });
        ResumeClassifierState {
            iter: self.iter,
            block,
            are_colons_on: self.are_colons_on,
            are_commas_on: self.are_commas_on,
        }
    }

    fn resume(state: ResumeClassifierState<'i, I, Q, MaskType>) -> Self {
        Self {
            iter: state.iter,
            block: state.block.map(|b| Block {
                quote_classified: b.block,
                idx: b.idx,
                are_commas_on: state.are_commas_on,
                are_colons_on: state.are_colons_on,
            }),
            are_commas_on: state.are_commas_on,
            are_colons_on: state.are_colons_on,
        }
    }
}
