
use super::*;

const PING_PROGR: &str =
"/*
 * Simple ping program
 */
program PING_PROG {
    version PING_VERS_PINGBACK {
        void         
        PINGPROC_NULL(void) = 0;
        /*
         * ping the caller, return the round-trip time
         * in milliseconds. Return a minus one (-1) if
         * operation times-out
         */
        int
        PINGPROC_PINGBACK(void) = 1;
        /* void - above is an argument to the call */
    } = 2;
/*
 * Original version
 */
    version PING_VERS_ORIG {
        void
        PINGPROC_NULL(void) = 0;
    } = 1;
} = 200000; 
const PING_VERS = 2; /* latest version */";

#[test]
fn ping() {
    let mut l = lexer();
    let mut tokens = l.parse_str(PING_PROGR);

    assert_eq!(
        token::Token::Comment("\n * Simple ping program\n ".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Program),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("PING_PROG".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::LeftCurly),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Version),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("PING_VERS_PINGBACK".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::LeftCurly),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Type(token::Type::Void),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("PINGPROC_NULL".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::Left),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Type(token::Type::Void),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::Right),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(0)),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Comment(String::from(
         "
         * ping the caller, return the round-trip time
         * in milliseconds. Return a minus one (-1) if
         * operation times-out
         "
        )),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Type(token::Type::Integer),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("PINGPROC_PINGBACK".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::Left),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Type(token::Type::Void),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::Right),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(1)),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Comment(String::from(
            " void - above is an argument to the call "
        )),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::RightCurly),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(2)),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Comment(String::from(
            "\n * Original version\n "
        )),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Version),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("PING_VERS_ORIG".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::LeftCurly),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Type(token::Type::Void),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("PINGPROC_NULL".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::Left),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Type(token::Type::Void),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::Right),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(0)),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::RightCurly),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(1)),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::RightCurly),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(200000)),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("PING_VERS".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(2)),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Comment(String::from(
            " latest version "
        )),
        tokens.next().unwrap().unwrap()
    );
    assert!(tokens.next().is_none());
}

const BAKERY_PROGR: &str =
"const REGISTER = 0;
const ACCESS   = 1;
const GET      = 2;
const STATUS   = 3;
const OP_MAX   = 4;

const STATUS_FREE         = 0;
const STATUS_REGISTERED   = 1;
const STATUS_ACCESSING    = 2;
const STATUS_READY_FOR_CR = 3;

const ERROR_WRONG_ID_RPC           = -1;
const ERROR_INCOMPATIBLE_HANLE_RPC = -2;
const ERROR_WRONG_STATUS_RPC       = -3;
const ERROR_REJECT_ACCESS_RPC      = -4;
const ERROR_WRONG_OP_RPC           = -5;

struct BAKERY
{
    int op;
    int id;
    int num;
    int result;
    struct BAKERY *not_supposed_2_be_here;
};

program BAKERY_PROG
{
    version BAKERY_VER
    {
        struct BAKERY BAKERY_PROC(struct BAKERY) = 1;
    } = 1;
} = 0x20000001;";

#[test]
fn bakery() {
    let mut l = lexer();
    let mut tokens = l.parse_str(BAKERY_PROGR);

    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("REGISTER".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(0)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("ACCESS".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(1)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("GET".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(2)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("STATUS".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(3)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("OP_MAX".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(4)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("STATUS_FREE".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(0)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("STATUS_REGISTERED".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(1)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("STATUS_ACCESSING".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(2)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("STATUS_READY_FOR_CR".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(3)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("ERROR_WRONG_ID_RPC".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(-1)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("ERROR_INCOMPATIBLE_HANLE_RPC".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(-2)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("ERROR_WRONG_STATUS_RPC".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(-3)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("ERROR_REJECT_ACCESS_RPC".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(-4)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Const),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("ERROR_WRONG_OP_RPC".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(-5)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Struct),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("BAKERY".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::LeftCurly),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Integer),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("op".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Integer),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("id".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Integer),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("num".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Integer),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("result".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Struct),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("BAKERY".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Pointer),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("not_supposed_2_be_here".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::RightCurly),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Program),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("BAKERY_PROG".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::LeftCurly),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Version),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("BAKERY_VER".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::LeftCurly),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Type(token::Type::Struct),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("BAKERY".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("BAKERY_PROC".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::Left),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Type(token::Type::Struct),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Identifier("BAKERY".to_string()),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::Right),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(1)),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::RightCurly),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(1)),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::RightCurly),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(0x20000001)),
        tokens.next().unwrap().unwrap()
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap()
    );
    assert!(tokens.next().is_none());
}

const UNION_AND_ENUM_TYPE: &str =
"enum cases
{
    NUMBERS = 1,
    NAME,
};

union test switch (enum cases value)
{
    case NUMBERS:
        unsigned int values[10];
    case NAME:
        string name<>;
    default:
        void;
};";

#[test]
fn union_enum() {
    let mut l = lexer();
    let mut tokens = l.parse_str(UNION_AND_ENUM_TYPE);

    assert_eq!(
        token::Token::Type(token::Type::Enum),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("cases".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::LeftCurly),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("NUMBERS".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Operator(token::Operator::Assign),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(1)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Comma),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("NAME".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Comma),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::RightCurly),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );

    assert_eq!(
        token::Token::Type(token::Type::Union),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("test".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Switch),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::Left),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Enum),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("cases".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("value".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::Right),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::LeftCurly),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Case),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("NUMBERS".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Colon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Unsigned),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Integer),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("values".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::LeftSquare),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Literal(token::Literal::Integer(10)),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::RightSquare),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Case),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("NAME".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Colon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::String),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Identifier("name".to_string()),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::LeftTriangle),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::RightTriangle),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Keyword(token::Keyword::Default),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Colon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Type(token::Type::Void),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Bracket(token::Bracket::RightCurly),
        tokens.next().unwrap().unwrap(),
    );
    assert_eq!(
        token::Token::Separator(token::Separator::Semicolon),
        tokens.next().unwrap().unwrap(),
    );

    assert!(tokens.next().is_none());
}

