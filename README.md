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

`ubs` is a library designed to provide real-time access to University at Buffalo (UB) class schedules, offering a wealth of information on each class. This includes data on class openings, start and end dates and times, class types such as recitation, lab, lecture, and seminar, class ID and section number, room number, instructor name, and even the number of open and closed seats available for each class.

## Examples
Below is a snippet of using the high-level API with [tokio](https://github.com/tokio-rs/tokio) for fetching live class information.
```rust
use futures::stream::TryStreamExt;
use ubs::Course;

#[tokio::main]
async fn main() -> Result<(), ubs::Error> {
    let mut schedule_iter = ubs::schedule_iter(Course::Cse115).await?;
    while let Some(schedule) = schedule_iter.try_next().await? {
        for group in schedule?.group_iter() {
            for class in group.class_iter() {
                // do stuff
            }
        }
    }

    Ok(())
}
```

## FAQ

### How does it work?
The process involves sending a precisely tailored sequence of network requests directed towards the [target URL](https://www.pub.hub.buffalo.edu/). Upon receiving the requests, the resulting HTML is cached until the user requests specific information, at which point it is parsed to specification and the values are extracted using Regex.

### Could I use this library from other languages?
Yes. While direct access to the core library may not be possible, the library does provide a command-line interface (CLI) that can output data in the desired format.

### How stable is this library?
Sort of stable. While using this library, it's important to note that there is a possibility that the underlying API may change in the future. Therefore, it may not be advisable to depend on this library for critical code. However, the library does have a comprehensive continuous integration (CI) system that runs daily, catching potential issues early on. In the event that the API does change and `ubs` ceases to function properly, users are encouraged to report the issue so that it can be resolved.

### Does this library operate on private information?
No, this library operates exclusively on public information that is readily accessible to anyone. There are no proprietary or confidential data sources involved.
