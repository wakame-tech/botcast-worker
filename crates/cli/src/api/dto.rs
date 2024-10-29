use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Script {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) template: Value,
    result: Value,
    user_id: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct UpdateScript {
    id: Uuid,
    title: String,
    // json
    template: String,
}

impl UpdateScript {
    pub(crate) fn new(id: Uuid, title: String, template: Value) -> Self {
        Self {
            id,
            title,
            template: serde_json::to_string(&template).unwrap(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct NewScript {
    title: String,
    // json
    template: String,
}

impl NewScript {
    pub(crate) fn new(title: String) -> Self {
        let default_template = serde_json::to_string(&json!({
            "$eval": "1+1"
        }))
        .unwrap();

        Self {
            title,
            template: default_template,
        }
    }
}
