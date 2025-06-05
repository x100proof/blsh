# Development

## Logging

To just turn on `blsh` level debugging do

```
$ BLSH_DEBUG=1 cargo run
```

To turn on logging for all `env_logger` enabled dependency crates

```
$ RUST_LOG=debug BLSH_DEBUG=1 cargo run
```