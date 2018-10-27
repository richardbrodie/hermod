# Hermod
A futures-based RSS-reading library for rust.

## To use

To fetch a feed

```rust
extern crate hermod;

use hermod::models::Feed;
use hermod::futures::fetch_feed;

fn get_a_feed(url: &str) {
  fetch_feed(url)
    .and_then(|feed| {
      let channel = feed.channel;
      let title = channel.title;
    });
}
```

To start a loop that will fetch several feeds, and run a custom func for each feed with the resulting `Feed`

```rust
extern crate hermod;

use std::sync::{Arc, Mutex};

use hermod::models::Feed;
use hermod::futures::start_fetch_loop;

fn automatically_fetch_feeds() {
  let interval = 300; // seconds
  let feeds = vec![
    "https://lorem-rss.herokuapp.com/feed".to_owned(),
    "https://feeds.feedburner.com/cyclingtipsblog/TJog".to_owned(),
  ];
  let feed_state = Arc::new(Mutex::new(feeds)); // thread-safe Vec of strings
  
  let func = |feed: Feed| println!("updated feed: {}", feed.channel.title); // func to run for each updated feed
  
  let work = start_fetch_loop(feed_state, interval, func);
  tokio::run(work);
}
```
