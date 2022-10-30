use redirs::{config::Config, db::{Database, SessionFactory}, server::Server};
use std::io;
use stderrlog::{self, LogLevelNum};

fn main() -> io::Result<()> {
    stderrlog::new()
        .verbosity(LogLevelNum::Trace)
        .init()
        .unwrap();

    let config = Config {
        host: "0.0.0.0".to_string(),
        port: 5101,
        databases: 16,
    };

    let database = Database::new(&config);
    let mut session_factory = SessionFactory::new(database);
    let server = Server::new(&config, &mut session_factory);
    server.run()
}
