use rsonpath::{
    engine::{Compiler, Engine, RsonpathEngine},
    input::MmapInput,
    result::MatchWriter,
};
use std::{env, error::Error, fs, io, process::ExitCode};
use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;
use rsonpath::input::error::Infallible;
use rsonpath::streaming::ByteStreamBlock;

fn string_to_iter(s: String) -> impl Iterator<Item = [u8; 64]> {
    let bytes = s.into_bytes();
    let len = bytes.len();
    let mut pos = 0;
    std::iter::from_fn(move || {
        if pos >= len {
            None
        } else {
            let mut array = [0u8; 64];
            let end = (pos + 64).min(len);
            array[..(end - pos)].copy_from_slice(&bytes[pos..end]);
            pos += 64;
            Some(array)
        }
    })
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        eprintln!("provide exactly 1 arguments, file path");
        return Ok(ExitCode::FAILURE);
    }

    let file_path = &args[1];

    let query = rsonpath_syntax::parse("$..search_metadata..count")?;
    let mut file = fs::File::open(file_path)?;
    let mut s = String::new();
    let n = file.read_to_string(&mut s)?;
    assert_eq!(n > 0, true);
    let input = string_to_iter(s);
    let stdout_lock = io::stdout().lock();
    let mut sink = MatchWriter::from(stdout_lock);

    let engine = RsonpathEngine::compile_query(&query)?;

    engine.matches_streaming::<_, _, Infallible>(input, &mut sink)?;
    print!("Finished processing");

    Ok(ExitCode::SUCCESS)
}