
use crate::handle;

pub fn unwrap_value(handle: &handle::Handle, v: &rpc::Value) -> i64 {
    match v {
        rpc::Value::Number(num) => *num,
        rpc::Value::Identifier(id) => unwrap_value(
            handle,
            handle.module.constants.get(id).expect("Was set")
        ),
    }
}

