use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::slice;
use rsonpath_syntax::prelude::JsonString;
use crate::{FallibleIterator, BLOCK_SIZE};
use crate::input::error::{Infallible, InputError};
use crate::input::{repr_align_block_size, BackwardSeekable, Input, InputBlock, InputBlockIterator, SliceSeekable};
use crate::result::InputRecorder;

pub struct FakeIterator<'r, I: Iterator<Item=[u8; BLOCK_SIZE]>, R: InputRecorder<ByteStreamBlock>> {
    pub stream: Rc<RefCell<ByteStream<I>>>,
    pub recorder: &'r R,
}

impl<'r, I: Iterator<Item=[u8; BLOCK_SIZE]>, R: InputRecorder<ByteStreamBlock>> InputBlockIterator<'_> for FakeIterator<'r, I, R> {
    type Block = ByteStreamBlock;
    type Error = Infallible;

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Block>, Self::Error> {
        let block = self.stream.borrow_mut().next()?;
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
        self.stream.borrow().get_offset()
    }

    #[inline]
    fn offset(&mut self, count: isize) {
        self.stream.borrow_mut().offset(count)
    }
}
#[derive(Clone)]
pub struct ByteStream<I: Iterator<Item=[u8; BLOCK_SIZE]>> {
    pub iter: I,
    pub data: Vec<ByteStreamBlock>,
    pub idx: usize,
}

impl<I: Iterator<Item=[u8; BLOCK_SIZE]>> ByteStream<I> {
    #[inline(always)]
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            data: Vec::new(),
            idx: 0,
        }
    }

    #[inline]
    fn get_block(&mut self, idx: usize) -> Option<&ByteStreamBlock> {
        while idx >= self.data.len() {
            match self.iter.next() {
                None => return None,
                Some(block) => self.data.push(ByteStreamBlock(block))
            }
        }
        Some(&self.data[idx])
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        let len = self.data.len() * BLOCK_SIZE;
        let ptr = self.data.as_slice().as_ptr().cast();

        unsafe { slice::from_raw_parts(ptr, len) }
    }
}

#[derive(Clone)]
#[repr(C, align(64))]
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

impl<'a, I: Iterator<Item=[u8; BLOCK_SIZE]>> InputBlockIterator<'a> for ByteStream<I> {
    type Block = ByteStreamBlock; // or [u8; BLOCK_SIZE]
    type Error = Infallible;

    #[inline(always)]
    fn next(&mut self) -> Result<Option<Self::Block>, Self::Error> {
        let block = self.get_block(self.idx).cloned();
        self.idx += 1;
        Ok(block)
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



pub struct InputStream<I: Iterator<Item=[u8; BLOCK_SIZE]>> {
    pub iter: Rc<RefCell<ByteStream<I>>>,
}


#[derive(Clone)]
pub struct FakeIter {
    pub iter: Rc<RefCell<dyn Iterator<Item = ByteStreamBlock>>>,
}

impl Iterator for FakeIter {
    type Item = ByteStreamBlock;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.borrow_mut().next()
    }
}

impl<I: Iterator<Item = [u8; 64]>> InputStream<I> {
    #[inline(always)]
    pub fn new(iter: I) -> Self {
        Self {
            iter: Rc::new(
                RefCell::new(
                    ByteStream::new(
                        iter
                    )
                )
            )
        }
    }
}

impl<I: Iterator<Item=[u8; BLOCK_SIZE]>> Input for InputStream<I> {
    type BlockIterator<'i, 'r, R> = FakeIterator<'r, I, R>
    where
        Self: 'i,
        R: InputRecorder<Self::Block<'i>> + 'r;
    type Error = Infallible;
    type Block<'i> = ByteStreamBlock
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
        R: InputRecorder<Self::Block<'i>>
    {
        FakeIterator {
            stream: self.iter.clone(),
            recorder,
        }
    }

    #[inline]
    fn seek_forward<const N: usize>(&self, from: usize, needles: [u8; N]) -> Result<Option<(usize, u8)>, Self::Error> {
        let mut iter = self.iter.borrow_mut();

        let mut from_idx = from / BLOCK_SIZE;
        let _ = iter.get_block(from_idx + 1);

        let mut moving_from = from;
        loop {
            let res = iter.as_slice().seek_forward(moving_from, needles);
            moving_from += BLOCK_SIZE;
            from_idx += 1;
            if res.is_some() {
                return Ok(res);
            } else {
                let b= iter.get_block(from_idx);
                if b.is_none() {
                    return Ok(None);
                }
            }
        }
    }

    #[inline]
    fn seek_non_whitespace_forward(&self, from: usize) -> Result<Option<(usize, u8)>, Self::Error> {
        let mut iter = self.iter.borrow_mut();

        let mut from_idx = from / BLOCK_SIZE;
        let _ = iter.get_block(from_idx + 1);

        let mut moving_from = from;
        loop {
            let res = iter.as_slice().seek_non_whitespace_forward(moving_from);
            moving_from += BLOCK_SIZE;
            from_idx += 1;
            if res.is_some() {
                return Ok(res);
            } else {
                let b= iter.get_block(from_idx);
                if b.is_none() {
                    return Ok(None);
                }
            }
        }
    }

    #[inline]
    fn is_member_match(&self, from: usize, to: usize, member: &JsonString) -> Result<bool, Self::Error> {
        let mut iter = self.iter.borrow_mut();
        let to_idx = to / BLOCK_SIZE;
        match iter.get_block(to_idx) {
            None => return Ok(false),
            Some(_) => {}
        }

        let slice = iter.as_slice();
        let bytes = &slice[from..to];
        Ok(member.quoted().as_bytes() == bytes && (from == 0 || slice[from - 1] != b'\\'))
    }
}

impl<I: Iterator<Item=[u8; BLOCK_SIZE]>> BackwardSeekable for InputStream<I> {

    #[inline]
    fn seek_backward(&self, from: usize, needle: u8) -> Option<usize> {
        let mut iter = self.iter.borrow_mut();
        iter.get_block(from);
        let slice = iter.as_slice();
        slice.seek_backward(from, needle)
    }

    #[inline]
    fn seek_non_whitespace_backward(&self, from: usize) -> Option<(usize, u8)> {
        let mut iter = self.iter.borrow_mut();
        iter.get_block(from);
        let slice = iter.as_slice();
        slice.seek_non_whitespace_backward(from)
    }
}
