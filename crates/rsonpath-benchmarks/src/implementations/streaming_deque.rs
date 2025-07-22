use crate::framework::implementation::Implementation;
use crate::implementations::rsonpath::RsonpathError;
use ouroboros::self_referencing;
use rsonpath::engine::main::MainEngine;
use rsonpath::engine::{Compiler, Engine};
use rsonpath::streaming::StreamInput;
use rsonpath_syntax::JsonPathQuery;
use std::fs;
use std::io::Read;
use std::vec::IntoIter;

pub struct StreamingDeque {}

#[self_referencing()]
pub struct StreamingDequeQuery {
    query: JsonPathQuery,
    #[borrows(query)]
    #[not_covariant]
    engine: MainEngine<'this>,
}
/*
impl Implementation for StreamingDeque {
    type Query = StreamingDequeQuery;

    type File = StreamInput<Box<dyn Iterator<Item = u8>>>;

    type Error = RsonpathError;

    type Result<'a> = &'static str;

    fn id() -> &'static str {
        "StreamingDeque"
    }

    fn new() -> Result<Self, Self::Error> {
        Ok(StreamingDeque {})
    }

    fn load_file(&self, file_path: &str) -> Result<Self::File, Self::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let iter = Box::new(reader.bytes().filter_map(|r| r.ok()));
        Ok(StreamInput::new(iter))
    }

    fn compile_query(&self, query: &str) -> Result<Self::Query, Self::Error> {
        let query = rsonpath_syntax::parse(query).unwrap();

        let rsonpath = StreamingDequeQuery::try_new(query, |query| {
            MainEngine::compile_query(query).map_err(RsonpathError::CompilerError)
        })?;

        Ok(rsonpath)
    }

    fn run(&self, query: &Self::Query, file: &Self::File) -> Result<Self::Result<'_>, Self::Error> {
        query
            .with_engine(|engine| engine.matches(file, &mut crate::implementations::rsonpath::VoidSink))
            .map_err(RsonpathError::EngineError)?;

        Ok("[not collected]")
    }
}
*/

impl Implementation for StreamingDeque {
    type Query = StreamingDequeQuery;

    type File = StreamInput<IntoIter<u8>>;

    type Error = RsonpathError;

    type Result<'a> = &'static str;

    fn id() -> &'static str {
        "StreamingDeque"
    }

    fn new() -> Result<Self, Self::Error> {
        Ok(StreamingDeque {})
    }

    fn load_file(&self, file_path: &str) -> Result<Self::File, Self::Error> {
        let file = fs::read(file_path)?.into_iter();
        Ok(StreamInput::new(file))
    }

    fn compile_query(&self, query: &str) -> Result<Self::Query, Self::Error> {
        let query = rsonpath_syntax::parse(query).unwrap();

        let rsonpath = StreamingDequeQuery::try_new(query, |query| {
            MainEngine::compile_query(query).map_err(RsonpathError::CompilerError)
        })?;

        Ok(rsonpath)
    }

    fn run(&self, query: &Self::Query, file: &Self::File) -> Result<Self::Result<'_>, Self::Error> {
        query
            .with_engine(|engine| engine.count(file))
            .map_err(RsonpathError::EngineError)?;

        Ok("[not collected]")
    }
}
