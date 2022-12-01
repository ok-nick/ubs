<div align="center">
  <h1><code>ubs</code></h1>
  <p><strong>University at Buffalo Schedule</strong></p>
  <p>
    <a href="https://docs.rs/ubs/0.0.0/ubs/"><img src="https://img.shields.io/readthedocs/ubs" alt="docs" /></a>
    <a href="https://crates.io/crates/ubs"><img src="https://img.shields.io/crates/v/ubs" alt="crates" /></a>
    <a href="https://discord.gg/w9Bc6xH7uC"><img src="https://img.shields.io/discord/834969350061424660?label=discord" alt="discord" /></a>
  </p>
</div>

`ubs` is a library for fetching University at Buffalo (UB) class schedules in real-time. This includes class openings, start/end date/time, class types (Recitation, Lab, Lecture, Seminar), class id, class section, room number, instructor, open/closed seats, etc.

The goal of this project is to provide an easy and descriptive interface for retrieving class schedule information. It will also make use of idiomatic techniques, such as async/await and lazy evaluation to attribute high-performance and low-memory code.

## Examples
Below is a snippet of fetching live class information.
```rust
// TODO
```

## FAQ

### How does it work?
Network requests are called directly and were reverse-engineered from [here](https://www.pub.hub.buffalo.edu/). This means there is no use of browser emulation, making it super fast and low footprint. The result of the requests is parsed in two stages, XML and HTML. The XML is first to be parsed, followed by HTML, which is then cached and stored. As operations are performed on the `Class`-like structs, the HTML is lazily-parsed.

### Could I use this library from other languages?
No, not yet at least. If you desire to use `ubs` from other languages, please leave an issue so I can see the demand. In the case that it's decided to be cross-language, a C API will be made so that languages with FFI support would be available.

### How stable is this library?
Not very; there is no guarantee UB will change the API in the future. Do not depend on this library for critical code. If the API does change and `ubs` stops working, please leave an issue for it to be resolved.

