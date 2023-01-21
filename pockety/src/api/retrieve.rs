use crate::{
    error,
    models::{
        ContentType,
        DetailType,
        ItemId,
        ItemImage,
        ItemStatus,
        ItemVideo,
        PocketItem,
        Sort,
        State,
        Tag,
        Timestamp,
    },
    pockety::Pockety,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct RetrieveResponse {
    pub list: Vec<PocketItem>,
    pub status: u16,
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

    pub fn search(mut self, search: &str) -> Self {
        self.body.search = Some(search.to_string());
        self
    }

    pub fn domain(mut self, domain: &str) -> Self {
        self.body.domain = Some(domain.to_string());
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

    pub fn since(mut self, since: Timestamp) -> Self {
        self.body.since = Some(since);
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

    pub async fn execute(self) -> Result<Vec<PocketItem>, error::Error> {
        let response: RetrieveResponse =
            self.pockety.post("/get", Some(&self.body)).await?;
        Ok(response.list)
    }
}
