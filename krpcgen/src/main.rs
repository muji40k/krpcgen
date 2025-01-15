
use lexer::Lexer;

fn main() {
    let config = rpc_generator::config::Config::new();

    let file = std::fs::File::open("./spec.x").expect("tests");
    let reader = std::io::BufReader::new(file);

    let tokens = rpc_lexer::lexer().parse(reader)
        .try_fold(Vec::new(), |mut out, t| t.map(|t| {
            out.push(t);
            out
        })).expect("tests");

    let defs = rpc_parser::parse(tokens.into_iter()).expect("tests");

    rpc_generator::testies(defs.definitions.into_iter(), Some(config)).expect("tests");
}

