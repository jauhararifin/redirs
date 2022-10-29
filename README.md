# Redirust

A redis server implementation written in Rust. Currently only supports 3 commands: SELECT, GET and SET.

## Running

```bash
cargo run --bin redirust
```

Redirust runs on port 5101

## Testing

```
> redis-cli -h 127.0.0.1 -p 5101

127.0.0.1:5101> GET jauhar
(nil)
127.0.0.1:5101> SET jauhar arifin
OK
127.0.0.1:5101> GET jauhar
"arifin"
```

