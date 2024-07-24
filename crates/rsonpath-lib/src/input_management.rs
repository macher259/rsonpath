use std::cell::RefCell;

use rsonpath_syntax::prelude::JsonString;

use crate::FallibleIterator;
use crate::input::{BasicInput, InputBlockIterator};
use crate::result::InputRecorder;

struct InternalManager<'input, 'recorder, I: BasicInput + 'input, R: InputRecorder<I::Block<'input, N>> + 'recorder, const N: usize> {
    input_iterator: I::BlockIterator<'input, 'recorder, R, N>,
    blocks: Vec<I::Block<'input, N>>,
}

impl<'input, 'recorder, 'a, I, R, const N: usize> InternalManager<'input, 'recorder, I, R, N>
where
    I: BasicInput<Block<'input, N>=&'a [u8]> + 'input,
    R: InputRecorder<I::Block<'input, N>> + 'recorder,
{
    pub fn new(input: &'input I, recorder: &'recorder R) -> Self {
        Self {
            input_iterator: input.iter_blocks(recorder),
            blocks: Vec::new(),
        }
    }

    pub fn get(&mut self, idx: usize) -> Result<Option<I::Block<'input, N>>, I::Error> {
        while idx < self.blocks.len() {
            let block = self.input_iterator.next()?;
            match block {
                Some(block) => self.blocks.push(block),
                None => return Ok(None)
            }
        }
        let b = self.blocks[idx];
        Ok(Some(b))
    }
}

pub struct BlockManager<'input, 'recorder, I: BasicInput + 'input, R: InputRecorder<I::Block<'input, N>>, const N: usize> {
    input: &'input I,
    inner: RefCell<InternalManager<'input, 'recorder, I, R, N>>,
}

impl<'input, 'recorder, 'a, I, R, const N: usize> BlockManager<'input, 'recorder, I, R, N>
where
    I: BasicInput<Block<'input, N>=&'a [u8]> + 'input,
    R: InputRecorder<I::Block<'input, N>> + 'recorder,
{
    pub fn new(input: &'input I, recorder: &'recorder R) -> Self {
        Self {
            input,
            inner: RefCell::new(InternalManager::new(input, recorder)),
        }
    }
}


impl<'input, 'recorder, I, R, const N: usize> BasicInput for BlockManager<'input, 'recorder, I, R, N>
where
    I: BasicInput + 'input,
    Self: 'input,
    R: InputRecorder<I::Block<'input, N>>,
{
    type BlockIterator<'input2, 'recorder2, R2, const N2: usize> =
    BlockManagerIterator<'input2, 'recorder2, I, R2, N2>
    where
        Self: 'input2,
        R2: InputRecorder<<Self as BasicInput>::Block<'input2, N2>>;
    type Error = I::Error;
    type Block<'input2, const N2: usize> = I::Block<'input2, N2>
    where
        Self: 'input2;

    fn len_hint(&self) -> Option<usize> {
        self.input.len_hint()
    }

    fn leading_padding_len(&self) -> usize {
        self.input.leading_padding_len()
    }

    fn trailing_padding_len(&self) -> usize {
        self.input.trailing_padding_len()
    }

    fn iter_blocks<'i, 'r, R2, const N2: usize>(&'i self, _: &'r R2) -> Self::BlockIterator<'i, 'r, R2, N2>
    where
        R2: InputRecorder<Self::Block<'i, N2>>,
    {
        BlockManagerIterator {
            internal_manager: unsafe {
                std::mem::transmute::<&RefCell<InternalManager<'input, 'recorder, _, R, N>>,
                    &RefCell<InternalManager<'i, 'r, _, R2, N2>>>(&self.inner)
            },
            current_block: 0,
        }
    }

    fn is_member_match(&self, from: usize, to: usize, member: &JsonString) -> Result<bool, Self::Error> {
        self.input.is_member_match(from, to, member)
    }
}

pub struct BlockManagerIterator<'input, 'recorder, I: BasicInput + 'input, R: InputRecorder<I::Block<'input, N>>, const N: usize> {
    internal_manager: &'input RefCell<InternalManager<'input, 'recorder, I, R, N>>,
    current_block: usize,
}


impl<'input, 'recorder, 'a, I, R, const N: usize> FallibleIterator for BlockManagerIterator<'input, 'recorder, I, R, N>
where
    'input: 'a,
    I: BasicInput<Block<'input, N>=&'a [u8]> + 'input,
    R: InputRecorder<I::Block<'input, N>> + 'recorder,
    <I as BasicInput>::Error: std::error::Error,
{
    type Item = I::Block<'input, N>;
    type Error = I::Error;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        let block = self.internal_manager.borrow_mut().get(self.current_block);
        self.current_block += 1;
        block
    }
}
