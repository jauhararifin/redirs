use redirust::{config::Config, server};

fn main() {
    let config = Config {
        host: "0.0.0.0".to_string(),
        port: 5101,
    };
    server::run(&config).unwrap();
}
