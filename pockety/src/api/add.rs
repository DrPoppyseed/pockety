use serde::{Deserialize, Serialize};

use crate::{
    models::{ItemHas, ItemId, ItemImage, ItemVideo, Tags, Timestamp},
    ApiResult, Pockety,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AddRequestBody {
    pub consumer_key: String,
    pub access_token: String,
    pub url: String,
    pub title: Option<String>,
    pub tags: Option<Tags>,
    pub tweet_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AddResponse {
    /// A unique identifier for the added item
    pub item_id: ItemId,
    /// The original url for the added item
    pub normal_url: String,
    /// A unique identifier for the resolved item
    pub resolved_id: ItemId,
    /// The resolved url for the added item. The easiest way to think about the
    /// resolved_url - if you add a bit.ly link, the resolved_url will be the
    /// url of the page the bit.ly link points to
    pub resolved_url: String,
    /// A unique identifier for the domain of the resolved_url
    pub domain_id: ItemId,
    /// A unique identifier for the domain of the normal_url
    pub origin_domain_id: ItemId,
    /// The response code received by the Pocket parser when it tried to access
    /// the item
    pub response_code: String,
    ///  The MIME type returned by the item
    pub mime_type: String,
    /// The content length of the item
    pub content_length: u32,
    /// The encoding of the item
    pub encoding: String,
    /// The date the item was resolved
    pub date_resolved: Timestamp,
    /// The date the item was published (if the parser was able to find one)
    pub date_published: Timestamp,
    /// The title of the resolved_url
    pub title: String,
    /// The excerpt of the resolved_url
    pub excerpt: String,
    /// For an article, the number of words
    pub word_count: u32,
    /// 0: no image; 1: has an image in the body of the article; 2: is an image
    pub has_image: ItemHas,
    /// 0: no video; 1: has a video in the body of the article; 2: is a video
    pub has_video: ItemHas,
    /// 0 or 1; If the parser thinks this item is an index page it will be set
    /// to 1
    pub is_index: bool,
    /// 0 or 1; If the parser thinks this item is an article it will be set to 1
    pub is_article: bool,
    /// Array of author data (if author(s) were found)
    pub authors: Vec<String>,
    // TODO: Should be ItemAuthor
    /// Array of image data (if image(s) were found)
    pub images: Vec<ItemImage>,
    /// Array of video data (if video(s) were found)
    pub videos: Vec<ItemVideo>,
}

#[derive(Debug)]
pub struct AddHandler<'po> {
    pockety: &'po Pockety,
    body: AddRequestBody,
}

impl<'po> AddHandler<'po> {
    pub fn new(pockety: &'po Pockety) -> Self {
        Self {
            pockety,
            body: Default::default(),
        }
    }

    pub fn access_token(mut self, access_token: String) -> Self {
        self.body.access_token = access_token;
        self
    }

    pub fn url(mut self, url: String) -> Self {
        self.body.url = url;
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.body.title = Some(title);
        self
    }

    pub fn tags(mut self, tags: Tags) -> Self {
        self.body.tags = Some(tags);
        self
    }

    pub fn tweet_id(mut self, tweet_id: String) -> Self {
        self.body.tweet_id = Some(tweet_id);
        self
    }

    pub async fn send(self) -> ApiResult<AddResponse> {
        let body = AddRequestBody {
            consumer_key: self.pockety.consumer_key.clone(),
            ..self.body
        };

        self.pockety
            .post::<AddRequestBody, AddResponse>("/send", Some(&body))
            .await
    }
}
