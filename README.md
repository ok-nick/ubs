<div align="center">
  <h1><code>ubs</code></h1>
  <p><strong>University at Buffalo Schedule</strong></p>
  <p>
    <a href="https://docs.rs/ubs/0.0.0/ubs/"><img src="https://img.shields.io/readthedocs/ubs" alt="docs" /></a>
    <a href="https://crates.io/crates/ubs"><img src="https://img.shields.io/crates/v/ubs" alt="crates" /></a>
    <a href="https://discord.gg/w9Bc6xH7uC"><img src="https://img.shields.io/discord/834969350061424660?label=discord" alt="discord" /></a>
  </p>
</div>

> **Warning**
> This library is still a work in progress. Do not expect full functionality.

`ubs` is a library for fetching University at Buffalo (UB) class schedules in real-time. This includes class openings, start/end date/time, class types (Recitation, Lab, Lecture, Seminar), class id, class section, room number, instructor, open/closed seats, etc.

The goal of this project is to provide an easy and descriptive interface for retrieving class schedule information. It also makes use of idiomatic techniques, such as async/await and lazy evaluation to additionally attribute high-performance code.

## Examples
Below is a snippet of using the high-level API for fetching live class information.
```rust
use futures::stream::TryStreamExt;

const CSE115_ID: &str = "004544";

#[tokio::main]
async fn main() {
    let schedules = ubs::schedule_iter(CSE115_ID);
    while let Some(schedule) = schedules.try_next().await {
        for group in schedule.unwrap().group_iter() {
            for class in group.class_iter() {
                // do stuff
            }
        }
    }
}
```

## FAQ

### How does it work?
It works by first creating network requests that are directly sent to UB servers from [here](https://www.pub.hub.buffalo.edu/). The backend API was reversed-engineered, meaning there is no use of browser emulation, making it super fast and low footprint. The result of the requests are parsed in a series of two stages: XML then HTML. The XML is parsed first for its inner HTML, which is then parsed, cached, and stored internally. As the class-like structs are called, the parsed HTML is lazily searched for class information and returned to the caller.

### Could I use this library from other languages?
No, not yet at least. If you desire to use `ubs` from other languages, please leave an issue so I can see the demand. In the case that it's decided to be cross-language, a C API will be made so that languages with FFI support could be available.

### How stable is this library?
Not very; there is no guarantee UB will change the API in the future. Do not depend on this library for critical code. If the API does change and `ubs` stops working, please leave an issue for it to be resolved.

