use crate::{
    error::{ApiError::MissingAccessToken, Error},
    models::{ItemId, Tags, Timestamp},
    pockety::Pockety,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum PocketAction {
    Add(Add),
    Archive(Update),
    Readd(Update),
    Favorite(Update),
    Unfavorite(Update),
    Delete(Update),
    TagsAdd(TagsAdd),
    TagsRemove(TagsRemove),
    TagsReplace(TagsReplace),
    TagsClear(TagsClear),
    TagRename(TagRename),
    TagDelete(TagDelete),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "action", rename = "add")]
pub struct Add {
    pub item_id: ItemId,
    pub ref_id: Option<u32>,
    pub tags: Option<String>,
    pub time: Option<Timestamp>,
    pub title: Option<String>,
    pub url: Option<String>,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq, Eq,
)]
#[serde(tag = "action", rename = "archive")]
pub struct Archive {
    pub item_id: ItemId,
    pub time: Timestamp,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq, Eq,
)]
#[serde(rename_all = "snake_case")]
pub enum UpdateName {
    Archive,
    Readd,
    Favorite,
    Unfavorite,
    Delete,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq, Eq,
)]
pub struct Update {
    pub action: UpdateName,
    pub item_id: ItemId,
    pub time: Timestamp,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "action", rename = "tags_add")]
pub struct TagsAdd {
    pub item_id: ItemId,
    pub tags: Tags,
    pub time: Option<Timestamp>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "action", rename = "tags_replace")]
pub struct TagsReplace {
    item_id: ItemId,
    tags: Tags,
    time: Option<Timestamp>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "action", rename = "tags_remove")]
pub struct TagsRemove {
    pub item_id: ItemId,
    pub tags: Tags,
    pub time: Option<Timestamp>,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Copy, Clone, PartialEq, Eq,
)]
#[serde(tag = "action", rename = "tags_clear")]
pub struct TagsClear {
    pub item_id: ItemId,
    pub time: Option<Timestamp>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "action", rename = "tag_rename")]
pub struct TagRename {
    pub old_tag: String,
    pub new_tag: String,
    pub time: Option<Timestamp>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "action", rename = "tag_delete")]
pub struct TagDelete {
    pub tag: String,
    pub time: Option<Timestamp>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct ModifyRequestBody {
    pub consumer_key: String,
    pub access_token: String,
    pub actions: Vec<PocketAction>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct ModifyResponse {
    pub status: u16,
    pub action_results: Vec<bool>,
}

#[derive(Debug)]
pub struct ModifyHandler<'po> {
    pockety: &'po Pockety,
    actions: Vec<PocketAction>,
}

impl<'po> ModifyHandler<'po> {
    pub fn new(pockety: &'po Pockety) -> Self {
        Self {
            pockety,
            actions: Vec::new(),
        }
    }

    pub fn push(&mut self, action: PocketAction) {
        self.actions.push(action);
    }

    pub async fn send(self) -> Result<Vec<bool>, Error> {
        if let Some(ref access_token) =
            *self.pockety.auth.access_token.lock().await
        {
            let body = ModifyRequestBody {
                consumer_key: self.pockety.auth.consumer_key.clone(),
                access_token: access_token.clone(),
                actions: self.actions,
            };

            let res: ModifyResponse =
                self.pockety.post("/send", Some(&body)).await?;

            Ok(res.action_results)
        } else {
            Err(Error::Api(MissingAccessToken))
        }
    }
}
