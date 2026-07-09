use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    title: String,
    subtitle: Option<String>,
    description: Option<String>,
    keywords: Vec<String>,
}
