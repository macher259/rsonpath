pub mod contagious_deque;
pub mod linked_list_stream;
pub mod raw_vec_deque;
pub mod skip_streaming;
pub mod vec_stream;

pub use contagious_deque::InputStream as ContagiousInputStream;
pub use linked_list_stream::StreamInput as LinkedListStream;
pub use raw_vec_deque::StreamInput;
pub use skip_streaming::StreamInput as SkipStream;
pub use vec_stream::InputStream as VecStream;

pub trait StreamingInput {}

impl<I: Iterator<Item = u8>> StreamingInput for raw_vec_deque::StreamInput<I> {}

impl<I: Iterator<Item = u8>> StreamingInput for contagious_deque::InputStream<I> {}

impl<I: Iterator<Item = u8>> StreamingInput for vec_stream::InputStream<I> {}

impl<I: Iterator<Item = u8>> StreamingInput for linked_list_stream::StreamInput<I> {}

impl<I: Iterator<Item = u8>> StreamingInput for skip_streaming::StreamInput<I> {}
