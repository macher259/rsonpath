use super::*;
use crate::input::{error::InputErrorConvertible, InputBlockIterator};

pub(crate) struct Constructor;

impl MemmemImpl for Constructor {
    type Classifier<'i, 'b, 'r, I, R>
        = SequentialMemmemClassifier<'i, 'b, 'r, I, R>
    where
        I: Input + 'i,
        <I as Input>::BlockIterator<'i, 'r, R>: 'b,
        R: InputRecorder<<I as Input>::Block<'i>> + 'r,
        'i: 'r;

    fn memmem<'i, 'b, 'r, I, R>(
        input: &'i I,
        iter: &'b mut <I as Input>::BlockIterator<'i, 'r, R>,
    ) -> Self::Classifier<'i, 'b, 'r, I, R>
    where
        I: Input,
        R: InputRecorder<<I as Input>::Block<'i>>,
        'i: 'r,
    {
        Self::Classifier { input, iter }
    }
}

pub(crate) struct SequentialMemmemClassifier<'i, 'b, 'r, I, R>
where
    I: Input,
    R: InputRecorder<I::Block<'i>> + 'r,
{
    input: &'i I,
    iter: &'b mut I::BlockIterator<'i, 'r, R>,
}

impl<'i, 'r, I, R> SequentialMemmemClassifier<'i, '_, 'r, I, R>
where
    I: Input,
    R: InputRecorder<I::Block<'i>> + 'r,
{
    #[inline]
    fn find_label_sequential(
        &mut self,
        label: &JsonString,
        mut offset: usize,
    ) -> Result<Option<(usize, I::Block<'i>)>, InputError> {
        let label_size = label.quoted().len();
        let first_c = if label.unquoted().is_empty() {
            b'"'
        } else {
            label.unquoted().as_bytes()[0]
        };

        while let Some(block) = self.iter.next().e()? {
            for (i, c) in block.iter().copied().enumerate() {
                let j = offset + i;

                if c == first_c && j > 0 && self.input.is_member_match(j - 1, j + label_size - 1, label).e()? {
                    self.iter.release_memory();
                    return Ok(Some((j - 1, block)));
                }
                self.iter.release_memory();
            }

            offset += block.len();
        }

        Ok(None)
    }
}

impl<'i, 'b, 'r, I, R> Memmem<'i, 'b, 'r, I> for SequentialMemmemClassifier<'i, 'b, 'r, I, R>
where
    I: Input,
    R: InputRecorder<I::Block<'i>> + 'r,
{
    // Output the relative offsets
    fn find_label(
        &mut self,
        first_block: Option<I::Block<'i>>,
        start_idx: usize,
        label: &JsonString,
    ) -> Result<Option<(usize, I::Block<'i>)>, InputError> {
        if let Some(b) = first_block {
            if let Some(res) = shared::find_label_in_first_block(self.input, b, start_idx, label)? {
                return Ok(Some(res));
            }
        }
        let next_block_offset = self.iter.get_offset();

        self.find_label_sequential(label, next_block_offset)
    }
}
