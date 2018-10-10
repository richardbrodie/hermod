# Hermod
A futures-based RSS-reading library for rust.

## To use

```rust
extern crate hermod;

use hermod::models::Feed;
use hermod::functions::fetch_feed;

fn get_a_feed(url: &str) {
  fetch_feed(url)
    .and_then(|feed| {
      let channel = feed.channel;
      let title = channel.title;
    });
}
```
