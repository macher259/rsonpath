use std::cell::{RefCell};
use std::ops::Deref;
use std::slice;
use rsonpath_syntax::prelude::JsonString;
use crate::{FallibleIterator, BLOCK_SIZE};
use crate::input::error::{Infallible, InputError};
use crate::input::{BackwardSeekable, Input, InputBlock, InputBlockIterator, SliceSeekable};
use crate::result::InputRecorder;

impl<'i, 'r, I, R, E> InputBlockIterator<'_> for FakeIterator<'i, 'r, I, R, E>
where
    I: Iterator<Item=[u8; BLOCK_SIZE]>,
    R: InputRecorder<ByteStreamBlock>,
    InputError: From<E>,
{
    type Block = ByteStreamBlock;
    type Error = E;

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
}
pub struct InnerByteStream<I: Iterator<Item=[u8; BLOCK_SIZE]>, E> {
    pub iter: I,
    pub data: Vec<ByteStreamBlock>,
    pub idx: usize,
    _phantom: std::marker::PhantomData<E>,
}

impl<I: Iterator<Item=[u8; BLOCK_SIZE]>, E> InnerByteStream<I, E> {
    #[inline(always)]
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            data: Vec::new(),
            idx: 0,
            _phantom: std::marker::PhantomData,
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

impl<'a, I, E> InputBlockIterator<'a> for InnerByteStream<I, E>
where
    I: Iterator<Item=[u8; BLOCK_SIZE]>,
    InputError: From<E>
{
    type Block = ByteStreamBlock; // or [u8; BLOCK_SIZE]
    type Error = E;

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



pub struct InputStream<I: Iterator<Item=[u8; BLOCK_SIZE]>, E> {
    pub iter: RefCell<InnerByteStream<I, E>>,
    _phantom: std::marker::PhantomData<E>,
}

impl<I: Iterator<Item = [u8; BLOCK_SIZE]>, E> InputStream<I, E> {
    #[inline(always)]
    pub fn new(iter: I) -> Self {
        Self {
            iter:
                RefCell::new(
                    InnerByteStream::new(
                        iter
                    )
                ),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct FakeIterator<'i, 'r, I: Iterator<Item=[u8; BLOCK_SIZE]>, R: InputRecorder<ByteStreamBlock>, E> {
    stream: &'i RefCell<InnerByteStream<I, E>>,
    recorder: &'r R,
    _phantom: std::marker::PhantomData<E>,
}

impl<I, E> Input for InputStream<I, E>
where
    I: Iterator<Item=[u8; BLOCK_SIZE]>,
    InputError: From<E>,
{
    type BlockIterator<'i, 'r, R> = FakeIterator<'i, 'r, I, R, E>
    where
        Self: 'i,
        R: InputRecorder<Self::Block<'i>> + 'r;
    type Error = E;
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
            stream: &self.iter,
            recorder,
            _phantom: std::marker::PhantomData,
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

impl<I: Iterator<Item=[u8; BLOCK_SIZE]>, E> BackwardSeekable for InputStream<I, E> {

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
