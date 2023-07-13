# ALIENTEK STM2F103 Examples

![board](docs/board.jpg)

## Build & Flash

- Install Rust.
- Install `cargo-flash` and [configure it](https://probe.rs/docs/getting-started/probe-setup/) for your platform.

```txt
cargo install cargo-flash
```

- Build and flash.

```txt
cargo flash --chip STM32F103RB -p <package> [--release]
```
