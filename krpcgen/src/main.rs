
use clap::Parser;

use lexer::Lexer;

enum Error {
    FS(std::io::Error),
    Lexer(lexer::Error),
    Parser(rpc_parser::Error),
}

/// Program for generating minimal linux kernel RPC modules for client and
/// server based on given rpcl specification
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to workspace
    #[arg(short, long, default_value_t = String::from("."))]
    path: String,

    /// Path to rpcl specificaion file
    #[arg(short, long, default_value_t = String::from("spec.x"))]
    specification: String,

    /// Constant value for maximum variable lenght array size
    #[arg(short, long, default_value_t = 1024)]
    vla_limit: usize,
}

fn args_to_config<'a>(args: &'a Args) -> rpc_generator::config::Config<'a> {
    let mut out = rpc_generator::config::Config::new();

    out.path = Some(std::path::Path::new(&args.path));
    out.vla_limit = Some(args.vla_limit);

    out
}

fn main() -> Result<(), Error>{
    let args = Args::parse();
    let config = args_to_config(&args);

    let file = std::fs::File::open(&args.specification)?;
    let reader = std::io::BufReader::new(file);

    let tokens = rpc_lexer::lexer().parse(reader)
        .try_fold(Vec::new(), |mut out, t| t.map(|t| {
            out.push(t);
            out
        }))?;

    let defs = rpc_parser::parse(tokens.into_iter())?;

    rpc_generator::generate(defs.definitions.into_iter(), Some(config))?;

    Ok(())
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FS(error) => write!(f,
                "Filesystem error: {error}"
            ),
            Error::Lexer(error) => write!(f,
                "Lexer error: {error}"
            ),
            Error::Parser(error) => write!(f,
                "Parser error: {error}"
            ),
        }
    }
}

impl From<lexer::Error> for Error {
    fn from(value: lexer::Error) -> Self {
        Self::Lexer(value)
    }
}

impl From<rpc_parser::Error> for Error {
    fn from(value: rpc_parser::Error) -> Self {
        Self::Parser(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::FS(value)
    }
}

