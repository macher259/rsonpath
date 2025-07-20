pub mod raw_vec_deque;

pub use raw_vec_deque::StreamInput;

pub trait StreamingInput {}

impl<I: Iterator<Item = u8>> StreamingInput for raw_vec_deque::StreamInput<I> {}
