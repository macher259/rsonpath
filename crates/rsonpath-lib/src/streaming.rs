use std::cell::RefCell;
use std::ops::Deref;
use rsonpath_syntax::prelude::JsonString;
use crate::{FallibleIterator, BLOCK_SIZE};
use crate::input::error::{Infallible, InputError};
use crate::input::{Input, InputBlock, InputBlockIterator, SliceSeekable};
use crate::result::InputRecorder;


#[derive(Clone)]
pub struct ByteStream<I: Iterator<Item=ByteStreamBlock> + Clone> {
    pub iter: I,
    pub data: Vec<u8>,
    pub idx: usize,
}

#[derive(Clone)]
pub struct ByteStreamBlock([u8; BLOCK_SIZE]);

impl Deref for ByteStreamBlock {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl<'i> InputBlock<'i> for ByteStreamBlock {
    fn halves(&self) -> (&[u8], &[u8]) {
        todo!()
    }
}

impl<'a, I: Iterator<Item=ByteStreamBlock> + Clone> InputBlockIterator<'a> for ByteStream<I> {
    type Block = ByteStreamBlock; // or [u8; BLOCK_SIZE]
    type Error = Infallible;

    fn next(&mut self) -> Result<Option<Self::Block>, Self::Error> {
        todo!()
    }

    fn get_offset(&self) -> usize {
        todo!()
    }

    fn offset(&mut self, count: isize) {
        todo!()
    }
}



pub struct InputStream<I: Iterator<Item=ByteStreamBlock> + Clone> {
    pub iter: ByteStream<I>,
}

impl<I: Iterator<Item=ByteStreamBlock> + Clone> Input for InputStream<I> {
    type BlockIterator<'i, 'r, R>
    where
        Self: 'i,
        R: InputRecorder<Self::Block<'i>> + 'r
    = ByteStream<I>;
    type Error = Infallible;
    type Block<'i>
    where
        Self: 'i
    = ByteStreamBlock;

    fn leading_padding_len(&self) -> usize {
        todo!()
    }

    fn trailing_padding_len(&self) -> usize {
        todo!()
    }

    fn iter_blocks<'i, 'r, R>(&'i self, recorder: &'r R) -> Self::BlockIterator<'i, 'r, R>
    where
        R: InputRecorder<Self::Block<'i>>
    {
        self.iter.clone()
    }

    fn seek_forward<const N: usize>(&self, from: usize, needles: [u8; N]) -> Result<Option<(usize, u8)>, Self::Error> {
        todo!()
    }

    fn seek_non_whitespace_forward(&self, from: usize) -> Result<Option<(usize, u8)>, Self::Error> {
        todo!()
    }

    fn is_member_match(&self, from: usize, to: usize, member: &JsonString) -> Result<bool, Self::Error> {
        todo!()
    }
}
