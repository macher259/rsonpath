use crate::framework::implementation::Implementation;
use crate::implementations::rsonpath::RsonpathError;
use ouroboros::self_referencing;
use rsonpath::engine::main::MainEngine;
use rsonpath::engine::{Compiler, Engine};
use rsonpath::streaming::VecStream;
use rsonpath_syntax::JsonPathQuery;
use std::fs;
use std::io::Read;
use std::vec::IntoIter;

pub struct StreamingVec {}

#[self_referencing()]
pub struct StreamingVecQuery {
    query: JsonPathQuery,
    #[borrows(query)]
    #[not_covariant]
    engine: MainEngine<'this>,
}

impl Implementation for StreamingVec {
    type Query = StreamingVecQuery;

    type File = VecStream<IntoIter<u8>>;

    type Error = RsonpathError;

    type Result<'a> = &'static str;

    fn id() -> &'static str {
        "StreamingVec"
    }

    fn new() -> Result<Self, Self::Error> {
        Ok(StreamingVec {})
    }

    fn load_file(&self, file_path: &str) -> Result<Self::File, Self::Error> {
        let file = fs::read(file_path)?.into_iter();
        Ok(VecStream::new(file))
    }

    fn compile_query(&self, query: &str) -> Result<Self::Query, Self::Error> {
        let query = rsonpath_syntax::parse(query).unwrap();

        let rsonpath = StreamingVecQuery::try_new(query, |query| {
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
