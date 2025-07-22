use crate::framework::implementation::Implementation;
use crate::implementations::rsonpath::RsonpathError;
use ouroboros::self_referencing;
use rsonpath::engine::main::MainEngine;
use rsonpath::engine::{Compiler, Engine};
use rsonpath::streaming::ContagiousInputStream;
use rsonpath_syntax::JsonPathQuery;
use std::fs;
use std::vec::IntoIter;

pub struct StreamingContagiousDeque {}

#[self_referencing()]
pub struct ContagiousDequeQuery {
    query: JsonPathQuery,
    #[borrows(query)]
    #[not_covariant]
    engine: MainEngine<'this>,
}

impl Implementation for StreamingContagiousDeque {
    type Query = ContagiousDequeQuery;

    type File = ContagiousInputStream<IntoIter<u8>>;

    type Error = RsonpathError;

    type Result<'a> = &'static str;

    fn id() -> &'static str {
        "ContagiousStreamingDeque"
    }

    fn new() -> Result<Self, Self::Error> {
        Ok(StreamingContagiousDeque {})
    }

    fn load_file(&self, file_path: &str) -> Result<Self::File, Self::Error> {
        let file = fs::read(file_path)?.into_iter();
        Ok(ContagiousInputStream::new(file))
    }

    fn compile_query(&self, query: &str) -> Result<Self::Query, Self::Error> {
        let query = rsonpath_syntax::parse(query).unwrap();

        let rsonpath = ContagiousDequeQuery::try_new(query, |query| {
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
