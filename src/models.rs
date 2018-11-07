use atom_syndication;
use serde_derive::Serialize;
use chrono::{DateTime, Utc};
use rss;
use std::str;

///////////
// Enums //
///////////

pub enum FeedType {
    RSS(rss::Channel),
    Atom(atom_syndication::Feed),
}
pub enum ItemType {
    Item(Vec<rss::Item>),
    Entry(Vec<atom_syndication::Entry>),
}

#[derive(Debug)]
pub enum Error {
    FetchError,
    ParseError,
}
/////////////
// Channel //
/////////////

pub struct Feed {
    pub channel: Channel,
    pub items: Vec<Item>,
}

/////////////
// Channel //
/////////////

#[derive(Debug, Serialize)]
pub struct Channel {
    pub title: String,
    pub description: Option<String>,
    pub site_link: String,
    pub feed_link: String,
    pub updated_at: DateTime<Utc>,
}
impl Channel {
    pub fn from_rss(feed: &rss::Channel, url: &str) -> Self {
        Self {
            title: feed.title().to_string(),
            site_link: feed.link().to_string(),
            feed_link: url.to_string(),
            description: Some(feed.description().to_string()),
            updated_at: Utc::now(),
        }
    }

    pub fn from_atom(feed: &atom_syndication::Feed, url: &str) -> Self {
        Self {
            title: feed.title().to_string(),
            site_link: feed.links()[0].href().to_string(),
            feed_link: url.to_string(),
            description: feed.subtitle().and_then(|s| Some(s.to_owned())),
            updated_at: Utc::now(),
        }
    }
}

//////////
// Item //
//////////

#[derive(Debug)]
pub struct Item {
    pub guid: String,
    pub link: String,
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
impl Item {
    pub fn from_item(item: &rss::Item) -> Result<Item, Error> {
        Ok(Item {
            guid: item.guid().ok_or(Error::ParseError)?.value().to_owned(),
            title: item.title().expect("no title!").to_owned(),
            link: item.link().expect("no link!").to_owned(),
            summary: item.description().and_then(|s| Some(s.to_owned())),
            content: item.content().and_then(|s| Some(s.to_owned())),
            published_at: item.pub_date().and_then(|d| parse_date(d)),
            updated_at: item.pub_date().and_then(|d| parse_date(d)),
        })
    }
    pub fn from_entry(item: &atom_syndication::Entry) -> Result<Item, Error> {
        Ok(Item {
            guid: item.id().to_owned(),
            title: item.title().to_owned(),
            link: item.links()[0].href().to_owned(),
            summary: item.summary().and_then(|s| Some(s.to_owned())),
            content: item
                .content()
                .and_then(|o| o.value().and_then(|s| Some(s.to_owned()))),
            published_at: item.published().and_then(|d| parse_date(d)),
            updated_at: parse_date(item.updated()),
        })
    }
}

fn parse_date(date: &str) -> Option<DateTime<Utc>> {
    match DateTime::parse_from_rfc2822(date) {
        Ok(d) => Some(d.with_timezone(&Utc)),
        Err(_) => date.parse::<DateTime<Utc>>().ok(),
    }
}
