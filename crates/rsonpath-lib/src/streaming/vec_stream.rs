use crate::input::error::Infallible;
use crate::input::{Input, InputBlock, InputBlockIterator, SliceSeekable};
use crate::result::InputRecorder;
use crate::BLOCK_SIZE;
use rsonpath_syntax::prelude::JsonString;
use std::cell::RefCell;
use std::ops::Deref;
use std::slice;

impl<'i, 'r, I, R> InputBlockIterator<'_> for FakeIterator<'i, 'r, I, R>
where
    I: Iterator<Item = u8>,
    R: InputRecorder<ByteStreamBlock>,
{
    type Block = ByteStreamBlock;
    type Error = Infallible;

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Block>, Self::Error> {
        let mut iter = self.stream.borrow_mut();

        let block = iter.next()?;
        match block {
            None => Ok(None),
            Some(block) => {
                self.recorder.record_block_start(block.clone());
                Ok(Some(block))
            }
        }
    }

    #[inline]
    fn get_offset(&self) -> usize {
        let iter = self.stream.borrow();
        iter.get_offset()
    }

    #[inline]
    fn offset(&mut self, count: isize) {
        let mut iter = self.stream.borrow_mut();
        iter.offset(count)
    }

    #[inline(always)]
    fn release_memory(&mut self) {
        self.stream.borrow_mut().release_memory();
    }
}
pub struct InnerByteStream<I: Iterator<Item = u8>> {
    pub iter: I,
    pub data: RefCell<Vec<ByteStreamBlock>>,
    pub idx: usize,
    pub released: usize,
}

impl<I: Iterator<Item = u8>> InnerByteStream<I> {
    #[inline(always)]
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            data: RefCell::new(Vec::new()),
            idx: 0,
            released: 0,
        }
    }

    #[inline]
    fn get_block(&mut self, idx: usize) -> Option<ByteStreamBlock> {
        while idx >= self.data.borrow().len() {
            let next_block = match self.iter.next_chunk::<BLOCK_SIZE>() {
                Ok(block) => Some(block),
                Err(remainder) => {
                    let mut remainder = remainder.collect::<Vec<u8>>();
                    if remainder.is_empty() {
                        return None;
                    } else {
                        remainder.resize(BLOCK_SIZE, b' ');
                        Some(remainder.try_into().expect("slice with incorrect length"))
                    }
                }
            };
            next_block.into_iter().for_each(|block| {
                self.data.borrow_mut().push(ByteStreamBlock(block));
            });
        }
        self.data.borrow().get(idx).map(Clone::clone)
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        let mut data = self.data.borrow_mut();
        let len = data.len() * BLOCK_SIZE;
        let ptr = data.as_ptr().cast();

        unsafe { slice::from_raw_parts(ptr, len) }
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct ByteStreamBlock(pub [u8; BLOCK_SIZE]);

impl Deref for ByteStreamBlock {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl<'i> InputBlock<'i> for ByteStreamBlock {
    #[inline(always)]
    fn halves(&self) -> (&[u8], &[u8]) {
        self.0.split_at(BLOCK_SIZE / 2)
    }
}

impl<'a, I> InputBlockIterator<'a> for InnerByteStream<I>
where
    I: Iterator<Item = u8>,
{
    type Block = ByteStreamBlock;
    type Error = Infallible;

    #[inline(always)]
    fn next(&mut self) -> Result<Option<Self::Block>, Self::Error> {
        let block = self.get_block(self.idx - self.released);
        self.idx += 1;
        Ok(block)
    }

    #[inline(always)]
    fn release_memory(&mut self) {
        if self.idx - self.released < 3 {
            return;
        }
        let last_idx_to_remove = self.idx // id of the next block to be read
            - 1 // now we have id of the current block
            - self.released // id of the actual block
            - 2; // save the last block

        self.data.borrow_mut().drain(0..=last_idx_to_remove);
        self.released += last_idx_to_remove + 1;
    }

    #[inline(always)]
    fn get_offset(&self) -> usize {
        self.idx * BLOCK_SIZE
    }

    #[inline(always)]
    fn offset(&mut self, count: isize) {
        self.idx = self.idx.saturating_add_signed(count);
    }
}

pub struct InputStream<I: Iterator<Item = u8>> {
    pub iter: RefCell<InnerByteStream<I>>,
}

impl<I: Iterator<Item = u8>> InputStream<I> {
    #[inline(always)]
    pub fn new(iter: I) -> Self {
        Self {
            iter: RefCell::new(InnerByteStream::new(iter)),
        }
    }
}

pub struct FakeIterator<'i, 'r, I: Iterator<Item = u8>, R: InputRecorder<ByteStreamBlock>> {
    stream: &'i RefCell<InnerByteStream<I>>,
    recorder: &'r R,
}

impl<I> Input for InputStream<I>
where
    I: Iterator<Item = u8>,
{
    type BlockIterator<'i, 'r, R>
        = FakeIterator<'i, 'r, I, R>
    where
        Self: 'i,
        R: InputRecorder<Self::Block<'i>> + 'r;
    type Error = Infallible;
    type Block<'i>
        = ByteStreamBlock
    where
        Self: 'i;

    #[inline(always)]
    fn leading_padding_len(&self) -> usize {
        0
    }

    #[inline(always)]
    fn trailing_padding_len(&self) -> usize {
        0
    }

    #[inline]
    fn iter_blocks<'i, 'r, R>(&'i self, recorder: &'r R) -> Self::BlockIterator<'i, 'r, R>
    where
        R: InputRecorder<Self::Block<'i>>,
    {
        FakeIterator {
            stream: &self.iter,
            recorder,
        }
    }

    #[inline]
    fn seek_backward(&self, from: usize, needle: u8) -> Option<usize> {
        let mut iter = self.iter.borrow_mut();
        let from = from - iter.released * BLOCK_SIZE;
        let from_idx = from / BLOCK_SIZE;
        iter.get_block(from_idx);
        let slice = iter.as_slice();
        slice
            .seek_backward(from, needle)
            .map(|res| res + iter.released * BLOCK_SIZE)
    }

    #[inline]
    fn seek_forward<const N: usize>(&self, from: usize, needles: [u8; N]) -> Result<Option<(usize, u8)>, Self::Error> {
        let mut iter = self.iter.borrow_mut();
        let from = from - iter.released * BLOCK_SIZE;

        let mut from_idx = from / BLOCK_SIZE;
        let _ = iter.get_block(from_idx + 1);

        let mut moving_from = from;
        loop {
            let res = iter.as_slice().seek_forward(moving_from, needles);
            moving_from += BLOCK_SIZE;
            from_idx += 1;
            if res.is_some() {
                return Ok(res.map(|(idx, byte)| (idx + iter.released * BLOCK_SIZE, byte)));
            } else {
                let b = iter.get_block(from_idx + 1);
                if b.is_none() {
                    return Ok(None);
                }
            }
        }
    }

    #[inline]
    fn seek_non_whitespace_forward(&self, from: usize) -> Result<Option<(usize, u8)>, Self::Error> {
        let mut iter = self.iter.borrow_mut();
        let from = from - iter.released * BLOCK_SIZE;
        let mut from_idx = from / BLOCK_SIZE;
        let _ = iter.get_block(from_idx + 1);

        let mut moving_from = from;
        loop {
            let res = iter.as_slice().seek_non_whitespace_forward(moving_from);
            moving_from += BLOCK_SIZE;
            from_idx += 1;
            if res.is_some() {
                return Ok(res.map(|(idx, byte)| (idx + iter.released * BLOCK_SIZE, byte)));
            } else {
                let b = iter.get_block(from_idx + 1);
                if b.is_none() {
                    return Ok(None);
                }
            }
        }
    }

    #[inline]
    fn seek_non_whitespace_backward(&self, from: usize) -> Option<(usize, u8)> {
        let mut iter = self.iter.borrow_mut();
        let from = from - iter.released * BLOCK_SIZE;
        let from_idx = from / BLOCK_SIZE;
        iter.get_block(from_idx);
        let slice = iter.as_slice();
        slice
            .seek_non_whitespace_backward(from)
            .map(|(res, b)| (res + iter.released * BLOCK_SIZE, b))
    }

    #[inline]
    fn is_member_match(&self, from: usize, to: usize, member: &JsonString) -> Result<bool, Self::Error> {
        let mut iter = self.iter.borrow_mut();
        let from = from - iter.released * BLOCK_SIZE;
        let to = to - iter.released * BLOCK_SIZE;

        let to_idx = to / BLOCK_SIZE;
        match iter.get_block(to_idx) {
            None => return Ok(false),
            Some(_) => {}
        }

        let slice = iter.as_slice();
        let bytes = &slice[from..to];

        let matched = member.quoted().as_bytes() == bytes && (from == 0 || slice[from - 1] != b'\\');

        Ok(matched)
    }
}
