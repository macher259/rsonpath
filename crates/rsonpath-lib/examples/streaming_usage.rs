use rsonpath::input::error::Infallible;
use rsonpath::{
    engine::{Compiler, Engine, RsonpathEngine},
    result::MatchWriter,
};
use std::io::{repeat, Read};
use std::{env, error::Error, fs, io, process::ExitCode};

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() != 3 {
        eprintln!("provide exactly 2 arguments, file path and query");
        return Ok(ExitCode::FAILURE);
    }

    let file_path = &args[1]; // File has to have length being multiple of 64

    let query = rsonpath_syntax::parse(&args[2])?;
    let mut file = fs::File::open(file_path)?;
    let mut s = Vec::new();
    let n = file.read_to_end(&mut s)?;
    assert_eq!(n > 0, true);
    let pad_len = 64 - (n % 64);
    s.resize(n + pad_len, ' ' as u8);
    //assert_eq!(n % 64, 0, "File length must be multiple of 64");
    let chunks = s.chunks_exact(64).map(|chunk| {
        let mut arr = [' ' as u8; 64];
        arr.copy_from_slice(chunk);
        arr
    });

    let engine = RsonpathEngine::compile_query(&query)?;

    let count = engine.count_streaming::<_, Infallible>(chunks)?;
    println!("{}", count);
    print!("Finished processing");

    Ok(ExitCode::SUCCESS)
}
