use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{config::Config, value::Bytes, value::Value};

use super::command::{get_commands, CommandSpec};

pub struct Database {
    dbs: Vec<Arc<RwLock<InternalDb>>>,
}

impl Database {
    pub fn new(config: &Config) -> Self {
        let n = config.databases.clamp(1, 16);
        let dbs: Vec<Arc<RwLock<InternalDb>>> = (0..n)
            .map(|_| Arc::new(RwLock::new(InternalDb::new())))
            .collect();
        Self { dbs }
    }

    pub fn get(&self, index: i64) -> Option<Arc<RwLock<InternalDb>>> {
        Some(self.dbs.get(index as usize)?.clone())
    }
}

pub struct InternalDb {
    pub storage: HashMap<Bytes, Bytes>,
}

impl InternalDb {
    fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

pub struct Session<'a> {
    pub handlers: HashMap<String, CommandSpec<'a>>,
    pub db: &'a Database,
    pub selected_db: Arc<RwLock<InternalDb>>,
}

pub struct SessionFactory {
    database: Database,
}

impl SessionFactory {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn create_session(&self) -> Session {
        let mut handlers = HashMap::new();
        for command in get_commands() {
            handlers.insert(command.name.clone().to_uppercase(), command);
        }

        Session {
            db: &self.database,
            selected_db: self.database.dbs.first().unwrap().clone(),
            handlers,
        }
    }
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

        let command = command.to_uppercase();
        let handler = match self.handlers.get(&command) {
            Some(v) => v,
            None => {
                return Value::err(format!(
                    "unknown command `{}`, with args beginning with: {}",
                    command,
                    args.get(0).unwrap_or(&Value::Null)
                ))
            }
        };

        (handler.handler)(self, args).unwrap_or_else(|s| Value::err(s))
    }
}
