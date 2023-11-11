<div align="center">
  <h1><code>ubs</code></h1>
  <p><strong>University at Buffalo Schedule</strong></p>
  <p>
    <a href="https://github.com/ok-nick/ubs/actions/workflows/test.yml"><img src="https://github.com/ok-nick/ubs/actions/workflows/test.yml/badge.svg" alt="test" /></a>
    <a href="https://docs.rs/ubs-lib"><img src="https://docs.rs/ubs-lib/badge.svg" alt="docs" /></a>
    <a href="https://crates.io/crates/ubs-lib"><img src="https://img.shields.io/crates/v/ubs-lib" alt="crates" /></a>
    <a href="https://discord.gg/w9Bc6xH7uC"><img src="https://img.shields.io/discord/834969350061424660?label=discord" alt="discord" /></a>
  </p>
</div>

`ubs` is a library designed to provide real-time access to University at Buffalo class schedules, offering a wealth of information on each class. This includes: `class open/closed`, `start/end date`, `start/end time`, `class type`, `class id`, `section`, `room`, `instructor`, `seats open/closed`, etc.

<img src="https://user-images.githubusercontent.com/25470747/258559272-32e79831-eda7-41b5-aba5-87c3d8fc363f.gif">

## Installation
### Cargo
```bash
$ cargo install ubs
```


## Examples
Below is a snippet of using the high-level API with [tokio](https://github.com/tokio-rs/tokio) for fetching live class information.
```rust
use futures::stream::TryStreamExt;
use ubs_lib::{Career, Course, Semester};

#[tokio::main]
async fn main() -> Result<(), ubs_lib::Error> {
    let mut schedule_iter = ubs_lib::schedule_iter(
        Course::Cse115Llr,
        Semester::Spring2024,
    ).await?;

    while let Some(schedule) = schedule_iter.try_next().await? {
        for group in schedule?.group_iter()? {
            for class in group.class_iter() {
                // do stuff
            }
        }
    }

    Ok(())
}
```

## Implementations
### [ubs-bot](https://github.com/ok-nick/ubs-bot/)
A Discord bot for querying and listening to changes in class schedules. Supports caching!

## FAQ
### How does it work?
The process involves sending a precisely tailored sequence of network requests directed towards the [target URL](https://www.pub.hub.buffalo.edu/). Upon receiving the requests, the resulting HTML is cached until the user requests specific information, at which point it is parsed to specification and the inner values are extracted using Regex.

### Could I use this library from other languages?
Yes. While direct access to the core library may not be possible from other languages (yet), the library does provide a command-line interface that can output data in the desired format.

### How stable is this library?
Sort of stable. While using this library, it's important to note that there is a possibility that the underlying API may change in the future. Therefore, it may not be advisable to depend on this library for critical code. However, the library does have a comprehensive continuous integration system that runs daily, catching potential issues early on. In the event that the API does change and `ubs` ceases to function properly, users are encouraged to report the issue so that it can be resolved.

### What do you mean by real-time, it's so slow?
Existing methods of fetching University at Buffalo class schedules operate **nightly** (if they even still exist). In contrast, `ubs` operates in real-time, fetching the latest and most up to date information.

### Does this library operate on private information?
No, this library operates exclusively on public information that is readily accessible to anyone. There are no proprietary or confidential data sources involved.
