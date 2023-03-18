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

`ubs` is a library for fetching University at Buffalo (UB) class schedules in real-time. This includes class openings, start/end date/time, class types (recitation, lab, lecture, seminar), class id, class section, room number, instructor, open/closed seats, etc.

The API exposes a high-level and low-level interface, giving users accessibility for either convenience or performance (or both). This is enabled by the use of `async`/`await` syntax and the ability to lazily parse data.

## Examples
Below is a snippet of using the high-level API with [tokio](https://github.com/tokio-rs/tokio) for fetching live class information.
```rust
use futures::stream::TryStreamExt;
use ubs::Course;

#[tokio::main]
async fn main() -> Result<(), ubs::Error> {
    let mut schedule_iter = ubs::schedule_iter(Course::CSE115).await?;
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
It works by first sending a carefully crafted series of network requests directly to [this url](https://www.pub.hub.buffalo.edu/). The result of the requests are then parsed in two stages: first the XML then the HTML. The HTML is nested inside of the XML, thus the XML is parsed first and cached internally. As the user requests the desired information, the HTML is parsed to spec, following up with regex to parse the values.

### Could I use this library from other languages?
No...at least not yet. If you'd like to use `ubs` from other languages, please leave an issue to generate demand. There are a few ways of going about this and I'd love to discuss potential opportunities.

### How stable is this library?
Not very; there is no guarantee UB will change the API in the future. Do not depend on this library for critical code. However, for the most part, issues will be caught early on due to the CI that is ran daily. Though, if the API does change and `ubs` stops working, please leave an issue for it to be resolved.

### Does this library operate on private information?
No, this library operates on public information that is accessible by anyone.
