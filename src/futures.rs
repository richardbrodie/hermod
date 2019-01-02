extern crate futures;

use self::futures::future::{err, ok, result, Future};
use self::futures::stream::Stream;
use atom_syndication;
use hyper::{Body, Client};
use hyper_tls::HttpsConnector;
use log::{debug, error};
use quick_xml::events::Event;
use quick_xml::Reader;
use rss;
use std::io::BufReader;
use std::str;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_current_thread::spawn;
use tokio_timer::Interval;

use super::models::{Channel, Error, Feed, FeedType, Item, ItemType};

////////////////////////
/// Future sequences ///
////////////////////////

pub fn fetch_feed(url: String) -> impl Future<Item = Feed, Error = Error> {
    fetch_feed_data(&url)
        .and_then(|data| identify_fetched_data(&data))
        .and_then(move |data| consume_feed_data(&data, &url))
        .and_then(|(new_feed, mut item_type)| {
            consume_item_types(&mut item_type).and_then(|items| {
                ok(Feed {
                    channel: new_feed,
                    items: items,
                })
            })
        })
        .map_err(|_| Error::FetchError)
}

pub fn start_fetch_loop<F: 'static + Send + Sync + Copy + Clone>(
    state: Arc<Mutex<Vec<String>>>,
    interval: u64,
    mut func: F,
) -> impl Future<Item = (), Error = ()>
where
    F: FnMut(Feed),
{
    Interval::new_interval(Duration::from_secs(interval))
        .for_each(move |_| {
            let urls = state.lock().unwrap().clone();
            urls.into_iter().for_each(|url| {
                let work = fetch_feed(url)
                    .and_then(move |feed| ok(func(feed)))
                    .map_err(|e| error!("error: {:?}", e));
                spawn(work);
            });
            ok(())
        })
        .map_err(|_| println!("timer error"))
}

/////////////////////////
/// Future components ///
/////////////////////////

fn fetch_feed_data(url: &str) -> impl Future<Item = Vec<u8>, Error = Error> {
    let url = url.to_owned();
    let https = HttpsConnector::new(2).expect("TLS initialization failed");
    let client = Client::builder().build::<_, Body>(https);
    let local = url.to_owned();
    client
        .get(url.parse().unwrap())
        .map_err(move |_err| Error::FetchError)
        .and_then(move |res| {
            res.into_body()
                .concat2()
                .map_err(|_err| Error::FetchError)
                .and_then(move |body| {
                    debug!("collected body: {}", local);
                    ok(body.to_vec())
                })
        })
}

fn identify_fetched_data(string: &[u8]) -> impl Future<Item = FeedType, Error = Error> {
    let mut buf = Vec::new();
    let mut reader = Reader::from_str(str::from_utf8(string).unwrap());
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"rss" => {
                    debug!("found rss");
                    match rss::Channel::read_from(BufReader::new(string)) {
                        Ok(channel) => return ok(FeedType::RSS(channel)),
                        Err(_) => return err(Error::ParseError),
                    }
                }
                b"feed" => {
                    debug!("found atom");
                    match atom_syndication::Feed::read_from(BufReader::new(string)) {
                        Ok(feed) => return ok(FeedType::Atom(feed)),
                        Err(_) => return err(Error::ParseError),
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

fn consume_feed_data(
    parsed: &FeedType,
    url: &str,
) -> impl Future<Item = (Channel, ItemType), Error = Error> {
    match parsed {
        FeedType::RSS(feed) => {
            let new_feed = Channel::from_rss(&feed, url);
            let new_items = ItemType::Item(feed.items().to_vec());
            ok((new_feed, new_items))
        }
        FeedType::Atom(feed) => {
            let new_feed = Channel::from_atom(&feed, url);
            let new_items = ItemType::Entry(feed.entries().to_vec());
            ok((new_feed, new_items))
        }
    }
}

fn consume_item_types(parsed: &mut ItemType) -> impl Future<Item = Vec<Item>, Error = Error> {
    result(match parsed {
        ItemType::Item(i) => process_items(i),
        ItemType::Entry(i) => process_entries(i),
    })
}

///////////////////
/// Synchronous ///
///////////////////

fn process_items<'a>(feed_items: &mut Vec<rss::Item>) -> Result<Vec<Item>, Error> {
    let items: Result<Vec<Item>, Error> = feed_items
        .iter()
        .map(|item| Item::from_item(item))
        .collect();
    items
}

fn process_entries<'a>(feed_items: &mut Vec<atom_syndication::Entry>) -> Result<Vec<Item>, Error> {
    let items: Result<Vec<Item>, Error> = feed_items
        .iter()
        .map(|entry| Item::from_entry(entry))
        .collect();
    items
}
