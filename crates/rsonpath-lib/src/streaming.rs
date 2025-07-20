pub mod contagious_deque;
pub mod raw_vec_deque;
pub mod vec_stream;

pub use contagious_deque::InputStream as ContagiousInputStream;
pub use raw_vec_deque::StreamInput;
pub use vec_stream::InputStream as VecStream;

pub trait StreamingInput {}

impl<I: Iterator<Item = u8>> StreamingInput for raw_vec_deque::StreamInput<I> {}

impl<I: Iterator<Item = u8>> StreamingInput for contagious_deque::InputStream<I> {}

impl<I: Iterator<Item = u8>> StreamingInput for vec_stream::InputStream<I> {}
