use redirust::{config::Config, server::Server, db::Database};
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

    let mut database = Database::new(&config);
    let server = Server::new(&config, &mut database);
    server.run()
}
