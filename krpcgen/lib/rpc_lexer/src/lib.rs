#[cfg(test)]
mod test;
mod matcher;

use rpc::token;
use lexer::{
    Lexer,
    generic::Lexer as GenericLexer,
};

pub fn lexer() -> impl Lexer<token::Token> {
    let mut out = GenericLexer::new();

    out.with_skip(|| char::is_whitespace)
        .push_back(matcher::separator_matcher)
        .push_back(matcher::bracket_matcher)
        .push_back(matcher::literal_matcher)
        .push_back(matcher::keyword_matcher)
        .push_back(matcher::type_matcher)
        .push_back(matcher::operator_matcher)
        .push_back(matcher::identifier_matcher)
        .push_back(matcher::comment_matcher);

    out
}

