use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use serde::{de, Deserialize, Serialize};
use time::OffsetDateTime;

use crate::error;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn now() -> Self {
        Self(OffsetDateTime::now_utc().unix_timestamp())
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(date_time: DateTime<Utc>) -> Self {
        let timestamp = date_time.timestamp();
        Self(timestamp)
    }
}

impl TryFrom<i64> for Timestamp {
    type Error = error::Error;

    fn try_from(timestamp: i64) -> Result<Self, Self::Error> {
        let date_time = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| {
                Self::Error::Parse(format!(
                    "failed to parse timestamp to datetime. error: {e:?}",
                ))
            })?;
        let timestamp = date_time.unix_timestamp();

        Ok(Self(timestamp))
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        i64::deserialize(deserializer)
            .and_then(|op| Timestamp::try_from(op).map_err(de::Error::custom))
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tags(pub Vec<String>);

impl<'de> Deserialize<'de> for Tags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let tags = Vec::<String>::deserialize(deserializer)?;
        Ok(Self(tags))
    }
}

impl Serialize for Tags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub struct ItemId(pub u32);

impl<'de> Deserialize<'de> for ItemId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = u32::deserialize(deserializer)?;
        Ok(Self(id))
    }
}

impl Serialize for ItemId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
pub struct ItemImage {
    pub item_id: ItemId,
    pub image_id: ItemId,
    pub src: String,
    pub width: u32,
    pub height: u32,
    pub caption: String,
    pub credit: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct ItemVideo {
    pub item_id: ItemId,
    pub video_id: ItemId,
    pub src: String,
    pub width: u32,
    pub height: u32,
    pub length: Option<u32>,
    pub vid: String,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, Copy,
)]
pub enum ItemStatus {
    #[serde(rename = "0")]
    Normal,
    #[serde(rename = "1")]
    Archived,
    #[serde(rename = "2")]
    Deleted,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, Copy,
)]
pub enum ItemHas {
    #[serde(rename = "0")]
    No,
    #[serde(rename = "1")]
    Yes,
    #[serde(rename = "2")]
    Is,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub enum DetailType {
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "complete")]
    Complete,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub enum Sort {
    #[serde(rename = "newest")]
    Newest,
    #[serde(rename = "oldest")]
    Oldest,
    #[serde(rename = "title")]
    Title,
    #[serde(rename = "site")]
    Site,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone)]
pub enum State {
    #[serde(rename = "unread")]
    Unread,
    #[serde(rename = "archive")]
    Archive,
    #[serde(rename = "all")]
    All,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone)]
pub enum Tag {
    #[serde(rename = "_untagged_")]
    Untagged,
    #[serde(rename = "tag_name")]
    TagName,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone)]
pub enum ContentType {
    #[serde(rename = "article")]
    Article,
    #[serde(rename = "video")]
    Video,
    #[serde(rename = "image")]
    Image,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct PocketItem {
    /// A unique identifier matching the saved item. This id must be used to
    /// perform any actions through the v3/modify endpoint.
    pub item_id: ItemId,
    /// A unique identifier similar to the item_id but is unique to the actual
    /// url of the saved item. The resolved_id identifies unique urls. For
    /// examples a direct link to a New York Times article and a link that
    /// redirects (ex a shortened bit.ly url) to the same article will share
    /// the same resolved_id. If this value is 0, it means that Pocket has not
    /// processed the item. Normally this happens within seconds but is possible
    /// you may request the item before it has been resolved.
    pub resolved_id: ItemId,
    /// The actual url that was saved with the item. This url should be used if
    /// the user wants to view the item.
    pub given_url: String,
    /// The title that was saved along with the item.
    pub given_title: String,
    /// 0 or 1 - 1 If the item is favorited
    pub favorite: bool,
    /// 0, 1, 2 - 1 if the item is archived - 2 if the item should be deleted
    pub status: ItemStatus,
    pub time_added: Option<Timestamp>,
    pub time_updated: Option<Timestamp>,
    pub time_read: Option<Timestamp>,
    pub time_favorited: Option<Timestamp>,
    /// The final url of the item. For examples if the item was a shortened
    /// bit.ly link, this will be the actual article the url linked to.
    pub resolved_url: String,
    /// The title that Pocket found for the item when it was parsed
    pub resolved_title: String,
    /// The first few lines of the item (articles only)
    pub excerpt: String,
    /// 0 or 1 - 1 if the item is an article
    pub is_article: bool,
    /// 0, 1, or 2 - 1 if the item has images in it - 2 if the item is an image
    pub has_image: ItemHas,
    /// 0, 1, or 2 - 1 if the item has videos in it - 2 if the item is a video
    pub has_video: ItemHas,
    /// How many words are in the article
    pub word_count: u32,
    /// A JSON object of the user tags associated with the item
    pub tags: String,
    /// A JSON object listing all of the authors associated with the item
    pub authors: String,
    /// A JSON object listing all of the images associated with the item
    pub images: Option<Vec<ItemImage>>,
    /// A JSON object listing all of the videos associated with the item
    pub videos: Option<Vec<ItemVideo>>,
}
