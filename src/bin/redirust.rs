use redirust::server::{Config, Server};

fn main() {
    let config = Config {
        host: "0.0.0.0".to_string(),
        port: 5101,
    };
    let mut server = Server::new(&config);
    server.run().unwrap();
}
