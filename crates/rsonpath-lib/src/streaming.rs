pub mod contagious_deque;
pub mod raw_vec_deque;

pub use contagious_deque::InputStream as ContagiousInputStream;
pub use raw_vec_deque::StreamInput;

pub trait StreamingInput {}

impl<I: Iterator<Item = u8>> StreamingInput for raw_vec_deque::StreamInput<I> {}

impl<I: Iterator<Item = u8>> StreamingInput for contagious_deque::InputStream<I> {}
