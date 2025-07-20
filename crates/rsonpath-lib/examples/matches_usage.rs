use rsonpath::engine::{Compiler, Engine, RsonpathEngine};
use rsonpath::input::OwnedBytes;
use rsonpath::result::MatchWriter;
use std::error::Error;
use std::io::Read;
use std::process::ExitCode;
use std::{env, fs, io};

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() != 3 {
        eprintln!("provide exactly 2 arguments, file path and query");
        return Ok(ExitCode::FAILURE);
    }

    let file_path = &args[1];

    let query = rsonpath_syntax::parse(&args[2])?;
    let engine = RsonpathEngine::compile_query(&query)?;
    let input = OwnedBytes::from(fs::read(file_path)?);

    let stdout_lock = io::stdout().lock();
    let mut sink = MatchWriter::from(stdout_lock);

    engine.matches(&input, &mut sink)?;
    print!("Finished processing");

    Ok(ExitCode::SUCCESS)
}
