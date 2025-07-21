use crate::input::error::Infallible;
use crate::input::{Input, InputBlock, InputBlockIterator, SliceSeekable};
use crate::result::InputRecorder;
use crate::BLOCK_SIZE;
use rsonpath_syntax::prelude::JsonString;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::ops::Deref;

const FILL_BYTE: u8 = b' ';

#[repr(transparent)]
#[derive(Clone)]
pub struct ByteBlock(pub(crate) [u8; BLOCK_SIZE]);

impl Deref for ByteBlock {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl<'i> InputBlock<'i> for ByteBlock {
    #[inline(always)]
    fn halves(&self) -> (&[u8], &[u8]) {
        self.0.split_at(BLOCK_SIZE / 2)
    }
}

impl<I: Iterator<Item = u8>> SliceSeekable for InnerStream<I> {
    #[inline]
    fn is_member_match(&self, from: usize, to: usize, member: &JsonString) -> bool {
        let list = self.data.borrow();
        let expected = member.quoted().as_bytes();
        debug_assert!(to < list.len() * BLOCK_SIZE, "Index out of bounds");
        if to - from < member.quoted().len() {
            return false;
        }
        let mut cursor = list.cursor_front();
        for (i, &b) in expected.iter().enumerate() {
            let abs_idx = from + i;
            let block_idx = abs_idx / BLOCK_SIZE;
            let offset = abs_idx % BLOCK_SIZE;

            while cursor.index().map(|idx| idx < block_idx).unwrap_or(false) {
                cursor.move_next();
            }
            if cursor.current().map(|block| block[offset]) != Some(b) {
                return false;
            }
        }

        if from == 0 {
            true
        } else {
            let escape_idx = from - 1;
            let block_idx = escape_idx / BLOCK_SIZE;
            let offset = escape_idx % BLOCK_SIZE;
            list.iter().nth(block_idx).map(|block| block.0[offset]) != Some(b'\\')
        }
    }

    #[inline]
    fn seek_backward(&self, from: usize, needle: u8) -> Option<usize> {
        let list = self.data.borrow();
        let total_len = list.len() * BLOCK_SIZE;

        assert!(from < total_len, "Index out of bounds");

        let mut idx = from;
        let mut cursor = list.cursor_front();
        loop {
            let block_idx = idx / BLOCK_SIZE;
            let offset = idx % BLOCK_SIZE;
            while cursor.index().map(|i| i < block_idx).unwrap_or(false) {
                cursor.move_next();
            }
            if cursor.current().map(|block| block.0[offset]) == Some(needle) {
                return Some(idx);
            }

            if idx == 0 {
                return None;
            }

            idx -= 1;
        }
    }

    #[inline]
    fn seek_forward<const N: usize>(&self, from: usize, needles: [u8; N]) -> Option<(usize, u8)> {
        assert!(N > 0);

        let list = self.data.borrow();
        let total_len = list.len() * BLOCK_SIZE;

        if from >= total_len {
            return None;
        }

        let mut idx = from;
        let mut cursor = list.cursor_front();
        loop {
            let block_idx = idx / BLOCK_SIZE;
            let offset = idx % BLOCK_SIZE;

            while cursor.index().map(|i| i < block_idx).unwrap_or(false) {
                cursor.move_next();
            }
            let b = cursor.current()?.0[offset];
            if needles.contains(&b) {
                return Some((idx, b));
            }

            idx += 1;
            if idx == total_len {
                return None;
            }
        }
    }

    #[inline]
    fn seek_non_whitespace_forward(&self, from: usize) -> Option<(usize, u8)> {
        let list = self.data.borrow();
        let total_len = list.len() * BLOCK_SIZE;
        if from >= total_len {
            return None;
        }
        let mut idx = from;
        if idx >= total_len {
            return None;
        }

        let mut cursor = list.cursor_front();
        loop {
            let block_idx = idx / BLOCK_SIZE;
            let offset = idx % BLOCK_SIZE;

            while cursor.index().map(|i| i < block_idx).unwrap_or(false) {
                cursor.move_next();
            }
            let b = cursor.current()?.0[offset];
            if !b.is_ascii_whitespace() {
                return Some((idx, b));
            }

            idx += 1;
            if idx == total_len {
                return None;
            }
        }
    }

    #[inline]
    fn seek_non_whitespace_backward(&self, from: usize) -> Option<(usize, u8)> {
        let list = self.data.borrow();
        let total_len = list.len() * BLOCK_SIZE;
        debug_assert!(from < total_len);
        if from >= total_len {
            return None;
        }

        let mut idx = from;

        let mut cursor = list.cursor_front();
        loop {
            let block_idx = idx / BLOCK_SIZE;
            let offset = idx % BLOCK_SIZE;

            while cursor.index().map(|i| i < block_idx).unwrap_or(false) {
                cursor.move_next();
            }
            let b = cursor.current()?.0[offset];
            if !b.is_ascii_whitespace() {
                return Some((idx, b));
            }

            if idx == 0 {
                return None;
            }
            idx -= 1;
        }
    }
}

struct InnerStream<I: Iterator<Item = u8>> {
    iter: I,
    data: RefCell<LinkedList<ByteBlock>>,
    released: usize,
}

impl<I: Iterator<Item = u8>> InnerStream<I> {
    #[inline(always)]
    fn new(iter: I) -> Self {
        Self {
            iter,
            data: RefCell::new(LinkedList::new()),
            released: 0,
        }
    }

    #[inline(always)]
    fn get_block(&mut self, idx: usize) -> Option<ByteBlock> {
        self.read_block(idx);
        self.data.borrow().iter().nth(idx).cloned()
    }

    #[inline(always)]
    fn read_block(&mut self, idx: usize) -> bool {
        while idx >= self.data.borrow().len() {
            let next_block = match self.iter.next_chunk::<BLOCK_SIZE>() {
                Ok(block) => Some(block),
                Err(remainder) => {
                    let mut remainder = remainder.collect::<Vec<u8>>();
                    if remainder.is_empty() {
                        return false;
                    } else {
                        remainder.resize(BLOCK_SIZE, FILL_BYTE);
                        Some(remainder.try_into().expect("slice with incorrect length"))
                    }
                }
            };
            next_block
                .into_iter()
                .for_each(|block| self.data.borrow_mut().push_back(ByteBlock(block)))
        }
        true
    }

    #[inline(always)]
    fn release_blocks(&mut self, last_idx_to_remove: usize) {
        self.released += last_idx_to_remove + 1;
        let mut list = self.data.borrow_mut();
        for _ in 0..=last_idx_to_remove {
            list.pop_front();
        }
    }
}

pub struct StreamInput<I: Iterator<Item = u8>> {
    inner: RefCell<InnerStream<I>>,
}

impl<I: Iterator<Item = u8>> StreamInput<I> {
    #[inline(always)]
    pub fn new(iter: I) -> Self {
        Self {
            inner: RefCell::new(InnerStream::new(iter)),
        }
    }
}

impl<I: Iterator<Item = u8>> Input for StreamInput<I> {
    type BlockIterator<'i, 'r, R>
        = StreamIterator<'i, 'r, I, R>
    where
        Self: 'i,
        R: InputRecorder<Self::Block<'i>> + 'r;
    type Error = Infallible;
    type Block<'i>
        = ByteBlock
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

    #[inline(always)]
    fn iter_blocks<'i, 'r, R>(&'i self, recorder: &'r R) -> Self::BlockIterator<'i, 'r, R>
    where
        R: InputRecorder<Self::Block<'i>>,
    {
        StreamIterator::new(&self.inner, recorder)
    }

    #[inline]
    fn seek_backward(&self, from: usize, needle: u8) -> Option<usize> {
        let mut inner = self.inner.borrow_mut();
        let from = from - inner.released * BLOCK_SIZE;
        let from_idx = from / BLOCK_SIZE;
        inner.read_block(from_idx);
        inner
            .seek_backward(from, needle)
            .map(|idx| idx + inner.released * BLOCK_SIZE)
    }

    #[inline]
    fn seek_forward<const N: usize>(&self, from: usize, needles: [u8; N]) -> Result<Option<(usize, u8)>, Self::Error> {
        let mut inner = self.inner.borrow_mut();
        let from = from - inner.released * BLOCK_SIZE;
        let mut from_idx = from / BLOCK_SIZE;
        inner.read_block(from_idx + 1);

        let mut moving_from = from;
        loop {
            let res = inner.seek_forward(moving_from, needles);
            moving_from += BLOCK_SIZE;
            from_idx += 1;
            if res.is_some() {
                return Ok(res.map(|(idx, b)| (idx + inner.released * BLOCK_SIZE, b)));
            } else if !inner.read_block(from_idx) {
                return Ok(None);
            }
        }
    }

    #[inline]
    fn seek_non_whitespace_forward(&self, from: usize) -> Result<Option<(usize, u8)>, Self::Error> {
        let mut inner = self.inner.borrow_mut();
        let from = from - inner.released * BLOCK_SIZE;
        let mut from_idx = from / BLOCK_SIZE;
        inner.read_block(from_idx + 1);
        let mut moving_from = from;
        loop {
            let res = inner.seek_non_whitespace_forward(moving_from);
            moving_from += BLOCK_SIZE;
            from_idx += 1;
            if res.is_some() {
                return Ok(res.map(|(idx, byte)| (idx + inner.released * BLOCK_SIZE, byte)));
            } else if !inner.read_block(from_idx) {
                return Ok(None);
            }
        }
    }

    #[inline]
    fn seek_non_whitespace_backward(&self, from: usize) -> Option<(usize, u8)> {
        let mut inner = self.inner.borrow_mut();
        let from = from - inner.released * BLOCK_SIZE;
        let from_idx = from / BLOCK_SIZE;
        inner.read_block(from_idx);
        inner
            .seek_non_whitespace_backward(from)
            .map(|(idx, byte)| (idx + inner.released * BLOCK_SIZE, byte))
    }

    #[inline]
    fn is_member_match(&self, from: usize, to: usize, member: &JsonString) -> Result<bool, Self::Error> {
        let mut inner = self.inner.borrow_mut();
        let from = from - inner.released * BLOCK_SIZE;
        let to = to - inner.released * BLOCK_SIZE;
        let to_idx = to / BLOCK_SIZE;
        if !inner.read_block(to_idx) {
            return Ok(false);
        }
        Ok(inner.is_member_match(from, to, member))
    }
}

pub struct StreamIterator<'i, 'r, I: Iterator<Item = u8>, R: InputRecorder<ByteBlock>> {
    inner: &'i RefCell<InnerStream<I>>,
    recorder: &'r R,
    idx: usize,
}

impl<'i, 'r, I, R> StreamIterator<'i, 'r, I, R>
where
    I: Iterator<Item = u8>,
    R: InputRecorder<ByteBlock>,
{
    #[inline(always)]
    fn new(inner: &'i RefCell<InnerStream<I>>, recorder: &'r R) -> Self {
        Self {
            inner,
            recorder,
            idx: 0,
        }
    }
}

impl<'i, 'r, I, R> InputBlockIterator<'i> for StreamIterator<'i, 'r, I, R>
where
    I: Iterator<Item = u8>,
    R: InputRecorder<ByteBlock>,
{
    type Block = ByteBlock;
    type Error = Infallible;

    #[inline(always)]
    fn next(&mut self) -> Result<Option<Self::Block>, Self::Error> {
        let mut inner = self.inner.borrow_mut();
        let released = inner.released;
        let block = inner.get_block(self.idx - released);

        self.idx += 1;
        match block {
            None => Ok(None),
            Some(b) => {
                self.recorder.record_block_start(b.clone());
                Ok(Some(b))
            }
        }
    }

    #[inline(always)]
    fn get_offset(&self) -> usize {
        self.idx * BLOCK_SIZE
    }

    #[inline(always)]
    fn offset(&mut self, count: isize) {
        self.idx = self.idx.saturating_add_signed(count);
    }

    #[inline(always)]
    fn release_memory(&mut self) {
        let mut inner = self.inner.borrow_mut();
        if self.idx - inner.released < 3 {
            return;
        }
        let last_idx_to_remove = self.idx - 1 - inner.released - 2;

        inner.release_blocks(last_idx_to_remove);
    }
}
