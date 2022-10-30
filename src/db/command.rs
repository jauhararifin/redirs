use crate::value::{Bytes, Value};

use super::Session;

pub type CommandFlag = &'static str;

pub const COMMAND_FLAG_READONLY: CommandFlag = "readonly";
pub const COMMAND_FLAG_RANDOM: CommandFlag = "random";
pub const COMMAND_FLAG_STRING: CommandFlag = "string";
pub const COMMAND_FLAG_SLOW: CommandFlag = "slow";
pub const COMMAND_FLAG_FAST: CommandFlag = "fast";
pub const COMMAND_FLAG_WRITE: CommandFlag = "write";
pub const COMMAND_FLAG_CONNECTION: CommandFlag = "connection";

pub struct CommandSpec<'a> {
    pub name: String,
    pub args_len: i64,
    pub flags: Vec<CommandFlag>,
    pub first_key: i64,
    pub last_key: i64,
    pub key_step: i64,
    pub handler: fn(&mut Session<'a>, Vec<Value>) -> Result<Value, String>,
}

const ERR_DB_INDEX: &str = "invalid DB index";
const ERR_DB_OUTOFRANGE: &str = "DB index is out of range";
const ERR_INVALID_KEY: &str = "invalid key";
const ERR_INVALID_VAL: &str = "invalid value";

pub fn get_commands<'a>() -> Vec<CommandSpec<'a>> {
    vec![
        CommandSpec {
            name: "COMMAND".to_string(),
            args_len: 1,
            flags: vec![COMMAND_FLAG_READONLY, COMMAND_FLAG_RANDOM],
            first_key: 1,
            last_key: 1,
            key_step: 1,
            handler: handle_command,
        },
        CommandSpec {
            name: "SELECT".to_string(),
            args_len: 1,
            flags: vec![COMMAND_FLAG_FAST, COMMAND_FLAG_CONNECTION],
            first_key: 0,
            last_key: 0,
            key_step: 0,
            handler: handle_select,
        },
        CommandSpec {
            name: "GET".to_string(),
            args_len: 2,
            flags: vec![COMMAND_FLAG_READONLY, COMMAND_FLAG_RANDOM],
            first_key: 1,
            last_key: 1,
            key_step: 1,
            handler: handle_get,
        },
        CommandSpec {
            name: "SET".to_string(),
            args_len: 2,
            flags: vec![COMMAND_FLAG_WRITE, COMMAND_FLAG_STRING, COMMAND_FLAG_SLOW],
            first_key: 1,
            last_key: 1,
            key_step: 1,
            handler: handle_set,
        },
    ]
}

fn handle_command(session: &mut Session, _: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Array(
        session
            .handlers
            .values()
            .map(|spec| {
                Value::Array(vec![
                    Value::Simple(Bytes::from(&spec.name)),
                    Value::Number(spec.args_len),
                    Value::Array(
                        spec.flags
                            .iter()
                            .map(|flag| Value::Simple(Bytes::from(*flag)))
                            .collect(),
                    ),
                    Value::Number(spec.first_key),
                    Value::Number(spec.last_key),
                    Value::Number(spec.key_step),
                ])
            })
            .collect(),
    ))
}

fn handle_select(session: &mut Session, args: Vec<Value>) -> Result<Value, String> {
    let mut args = args.into_iter();

    let target_db = args
        .next()
        .ok_or("wrong number of arguments for 'select' command")?;

    let target_db = match target_db {
        Value::Number(n) => n,
        Value::Simple(s) | Value::Blob(s) => s
            .into_string()
            .map_err(|_| ERR_DB_OUTOFRANGE.to_string())?
            .parse::<i64>()
            .map_err(|_| ERR_DB_INDEX.to_string())?,
        _ => return Err(ERR_DB_INDEX.to_string()),
    };

    session.selected_db = session.db.get(target_db).ok_or(ERR_DB_OUTOFRANGE)?;

    Ok(Value::Simple("OK".into()))
}

fn handle_get(session: &mut Session, args: Vec<Value>) -> Result<Value, String> {
    let mut args = args.into_iter();
    let key = args
        .next()
        .ok_or("wrong number of arguments for 'get' command")?;

    let key = match key {
        Value::Simple(s) | Value::Blob(s) => s,
        _ => return Err(ERR_INVALID_KEY.to_string()),
    };

    Ok(session
        .selected_db
        .read()
        .unwrap()
        .storage
        .get(&key)
        .map(|v| Value::Blob(v.clone()))
        .unwrap_or(Value::Null))
}

fn handle_set(session: &mut Session, args: Vec<Value>) -> Result<Value, String> {
    let mut args = args.into_iter();
    let key = args
        .next()
        .ok_or("wrong number of arguments for 'set' command")?;

    let key = match key {
        Value::Simple(s) | Value::Blob(s) => s,
        _ => return Err(ERR_INVALID_KEY.to_string()),
    };

    let value = args
        .next()
        .ok_or("wrong number of arguments for 'set' command")?;

    let value = match value {
        Value::Simple(s) | Value::Blob(s) => s,
        _ => return Err(ERR_INVALID_VAL.to_string()),
    };

    session
        .selected_db
        .write()
        .unwrap()
        .storage
        .insert(key, value);

    Ok(Value::Simple("OK".into()))
}
