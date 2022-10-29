use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{config::Config, value::Value};

pub struct Database {
    dbs: Vec<Arc<Mutex<InternalDb>>>,
}

impl Database {
    pub fn new(config: &Config) -> Self {
        let n = if config.databases == 0 {
            16
        } else {
            config.databases
        };

        let dbs: Vec<Arc<Mutex<InternalDb>>> = (0..n)
            .map(|_| Arc::new(Mutex::new(InternalDb::new())))
            .collect();

        Self { dbs }
    }

    fn get(&self, index: i64) -> Option<Arc<Mutex<InternalDb>>> {
        Some(self.dbs.get(index as usize)?.clone())
    }

    pub fn create_session(&self) -> Session {
        let selected_db = self.dbs.first().unwrap().clone();
        Session {
            db: self,
            selected_db,
        }
    }
}

pub struct Session<'a> {
    db: &'a Database,
    selected_db: Arc<Mutex<InternalDb>>,
}

impl<'a> Session<'a> {
    pub fn handle_request(&mut self, request: Value) -> Value {
        let request = match request {
            Value::Array(v) => v,
            _ => return Value::err(format!("Invalid request from client: {:?}", request)),
        };

        if request.is_empty() {}

        let mut request = request.into_iter();
        let command = match request.next() {
            Some(command) => command,
            None => return Value::err(format!("Invalid request from client: {:?}", request)),
        };

        let command = match command {
            Value::Simple(v) => v,
            Value::Blob(v) => v,
            _ => return Value::err(format!("Invalid command from client: {:?}", command)),
        };

        let command = match command.into_string() {
            Ok(v) => v,
            Err(_) => return Value::err(format!("Invalid command from client")),
        };

        let args: Vec<Value> = request.collect();

        match command.to_uppercase().as_str() {
            "COMMAND" => self.handle_command_command(args),
            "SELECT" => self.handle_select_command(args),
            cmd @ _ => Value::err(format!(
                "unknown command `{}`, with args beginning with: {}",
                cmd,
                args.get(0).unwrap_or(&Value::Null),
            )),
        }
    }

    fn handle_command_command(&mut self, _: Vec<Value>) -> Value {
        Value::Array(vec![
            Value::Array(vec![
                Value::Simple("COMMAND".into()),
                Value::Number(1),
                Value::Array(vec![
                    Value::Simple("readonly".into()),
                    Value::Simple("random".into()),
                ]),
                Value::Number(1),
                Value::Number(1),
                Value::Number(1),
            ]),
            Value::Array(vec![
                Value::Simple("GET".into()),
                Value::Number(2),
                Value::Array(vec![
                    Value::Simple("readonly".into()),
                    Value::Simple("random".into()),
                ]),
                Value::Number(1),
                Value::Number(1),
                Value::Number(1),
            ]),
            Value::Array(vec![
                Value::Simple("SET".into()),
                Value::Number(2),
                Value::Array(vec![
                    Value::Simple("write".into()),
                    Value::Simple("string".into()),
                    Value::Simple("slow".into()),
                ]),
                Value::Number(1),
                Value::Number(1),
                Value::Number(1),
            ]),
            Value::Array(vec![
                Value::Simple("SELECT".into()),
                Value::Number(2),
                Value::Array(vec![
                    Value::Simple("fast".into()),
                    Value::Simple("connection".into()),
                ]),
                Value::Number(0),
                Value::Number(0),
                Value::Number(0),
            ]),
        ])
    }

    fn handle_select_command(&mut self, args: Vec<Value>) -> Value {
        let mut args = args.into_iter();
        let target_db = match args.next() {
            Some(n) => n,
            None => return Value::err("wrong number of arguments for 'select' command"),
        };

        let target_db = match target_db {
            Value::Number(n) => n,
            Value::Simple(s) | Value::Blob(s) => match s.into_string() {
                Ok(s) => match s.parse::<i64>() {
                    Ok(n) => n,
                    _ => return Value::err("invalid DB index"),
                },
                _ => return Value::err("invalid DB index"),
            },
            _ => return Value::err("invalid DB index"),
        };

        let selected_db = match self.db.get(target_db) {
            Some(db) => db,
            None => return Value::err("DB index is out of range"),
        };
        self.selected_db = selected_db;

        Value::Simple("OK".into())
    }
}

pub struct InternalDb {
    storage: HashMap<String, Value>,
}

impl InternalDb {
    fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}
