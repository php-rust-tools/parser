use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::lexer::token::Span;
use crate::node::Node;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CommaSeparated<T> {
    pub inner: Vec<T>,
    pub commas: Vec<Span>, // `,`
}

impl<T: Node> Node for CommaSeparated<T> {
    fn children(&self) -> Vec<&dyn Node> {
        self.inner.iter().map(|x| x as &dyn Node).collect()
    }
}
