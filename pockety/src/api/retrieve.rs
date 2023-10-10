use std::collections::HashMap;

use futures::TryFutureExt;
use serde::{Deserialize, Serialize};

use crate::{
    models::{ContentType, DetailType, PocketItem, Sort, State, Tag, Timestamp},
    ApiResult, Pockety, PocketyResponse,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RetrieveRequestBody {
    pub consumer_key: String,
    pub access_token: String,
    pub search: Option<String>,
    pub domain: Option<String>,
    pub tag: Option<Tag>,
    pub state: Option<State>,
    pub content_type: Option<ContentType>,
    pub detail_type: Option<DetailType>,
    pub favorite: Option<bool>,
    pub since: Option<Timestamp>,
    pub sort: Option<Sort>,
    pub count: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RetrieveResponse {
    pub list: HashMap<String, PocketItem>,
    pub status: u16,
    pub complete: u16,
    pub error: Option<String>,
    pub since: Option<i64>,
    // search_meta isn't documented in the API docs, but it's in the response.
    // we're not too sure what we want to do with this, so we're just going to
    // leave it as a serde_json::Value for now.
    pub search_meta: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct RetrieveHandler<'po> {
    pockety: &'po Pockety,
    body: RetrieveRequestBody,
}

impl<'po> RetrieveHandler<'po> {
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

    pub fn search(mut self, search: String) -> Self {
        self.body.search = Some(search);
        self
    }

    pub fn domain(mut self, domain: String) -> Self {
        self.body.domain = Some(domain);
        self
    }

    pub fn tag(mut self, tag: Tag) -> Self {
        self.body.tag = Some(tag);
        self
    }

    pub fn state(mut self, state: State) -> Self {
        self.body.state = Some(state);
        self
    }

    pub fn content_type(mut self, content_type: ContentType) -> Self {
        self.body.content_type = Some(content_type);
        self
    }

    pub fn detail_type(mut self, detail_type: DetailType) -> Self {
        self.body.detail_type = Some(detail_type);
        self
    }

    pub fn favorite(mut self, fav: bool) -> Self {
        self.body.favorite = Some(fav);
        self
    }

    pub fn since(mut self, since: impl Into<Timestamp>) -> Self {
        self.body.since = Some(since.into());
        self
    }

    pub fn sort(mut self, sort: Sort) -> Self {
        self.body.sort = Some(sort);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.body.offset = Some(offset);
        self
    }

    pub fn count(mut self, count: u32) -> Self {
        self.body.count = Some(count);
        self
    }

    pub async fn execute(self) -> ApiResult<Vec<PocketItem>> {
        let body = RetrieveRequestBody {
            consumer_key: self.pockety.consumer_key.clone(),
            ..self.body
        };

        self.pockety
            .post::<RetrieveRequestBody, RetrieveResponse>("/get", Some(&body))
            .map_ok(|res| PocketyResponse {
                rate_limits: res.rate_limits,
                data: res.data.list.into_values().collect(),
            })
            .await
    }
}
