use crate::{
    error::{ApiError::MissingAccessToken, PocketyError},
    models::{ItemHas, ItemId, ItemImage, ItemVideo, Tags, Timestamp},
    pockety::Pockety,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct AddRequestBody {
    consumer_key: String,
    access_token: String,
    url:          String,
    title:        Option<String>,
    tags:         Option<Tags>,
    tweet_id:     Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct AddResponse {
    /// A unique identifier for the added item
    pub item_id:          ItemId,
    /// The original url for the added item
    pub normal_url:       String,
    /// A unique identifier for the resolved item
    pub resolved_id:      ItemId,
    /// The resolved url for the added item. The easiest way to think about the
    /// resolved_url - if you add a bit.ly link, the resolved_url will be the
    /// url of the page the bit.ly link points to
    pub resolved_url:     String,
    /// A unique identifier for the domain of the resolved_url
    pub domain_id:        ItemId,
    /// A unique identifier for the domain of the normal_url
    pub origin_domain_id: ItemId,
    /// The response code received by the Pocket parser when it tried to access
    /// the item
    pub response_code:    String,
    ///  The MIME type returned by the item
    pub mime_type:        String,
    /// The content length of the item
    pub content_length:   u32,
    /// The encoding of the item
    pub encoding:         String,
    /// The date the item was resolved
    pub date_resolved:    Timestamp,
    /// The date the item was published (if the parser was able to find one)
    pub date_published:   Timestamp,
    /// The title of the resolved_url
    pub title:            String,
    /// The excerpt of the resolved_url
    pub excerpt:          String,
    /// For an article, the number of words
    pub word_count:       u32,
    /// 0: no image; 1: has an image in the body of the article; 2: is an image
    pub has_image:        ItemHas,
    /// 0: no video; 1: has a video in the body of the article; 2: is a video
    pub has_video:        ItemHas,
    /// 0 or 1; If the parser thinks this item is an index page it will be set
    /// to 1
    pub is_index:         bool,
    /// 0 or 1; If the parser thinks this item is an article it will be set to 1
    pub is_article:       bool,
    /// Array of author data (if author(s) were found)
    pub authors:          Vec<String>,
    // TODO: Should be ItemAuthor
    /// Array of image data (if image(s) were found)
    pub images:           Vec<ItemImage>,
    /// Array of video data (if video(s) were found)
    pub videos:           Vec<ItemVideo>,
}

pub struct AddHandler<'po> {
    pockety: &'po Pockety,
    body:    AddRequestBody,
}

impl<'po> AddHandler<'po> {
    pub fn new(pockety: &'po Pockety) -> Self {
        Self {
            pockety,
            body: Default::default(),
        }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.body.url = url.to_string();
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.body.title = Some(title.to_string());
        self
    }

    pub fn tags(mut self, tags: Tags) -> Self {
        self.body.tags = Some(tags);
        self
    }

    pub fn tweet_id(mut self, tweet_id: &str) -> Self {
        self.body.tweet_id = Some(tweet_id.to_string());
        self
    }

    pub async fn send(self) -> Result<AddResponse, PocketyError> {
        if let Some(access_token) = self.pockety.auth.access_token.clone() {
            let body = AddRequestBody {
                consumer_key: self.pockety.auth.consumer_key.clone(),
                access_token: access_token.clone(),
                url:          self.body.url,
                title:        self.body.title,
                tags:         self.body.tags,
                tweet_id:     self.body.tweet_id,
            };

            let res: AddResponse =
                self.pockety.post("/send", Some(&body)).await?;

            Ok(res)
        } else {
            Err(PocketyError::Api(MissingAccessToken))
        }
    }
}
