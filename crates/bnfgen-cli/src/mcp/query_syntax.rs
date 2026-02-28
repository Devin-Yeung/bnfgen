use include_dir::{include_dir, Dir};
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static SYNTAX_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/resource/syntax");

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct QuerySyntaxRequest {
    /// The syntax topic to query. Available topics:
    /// - "core": Core BNF grammar structure
    /// - "regex": Regex symbol syntax (re("..."))
    /// - "limit": Invoke limits syntax ({min,max})
    /// - "weight": Alternative weights syntax
    pub topic: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct QuerySyntaxResponse {
    /// The syntax documentation content for the requested topic
    pub content: String,
    /// The name of the syntax topic
    pub topic: String,
    /// Available topics for navigation
    pub available_topics: Vec<String>,
}

/// Get the syntax content for a given topic
pub fn get_syntax_content(topic: &str) -> Option<String> {
    let filename = format!("{}.md", topic);
    SYNTAX_DIR.get_file(&filename).map(|file| {
        file.contents_utf8()
            .expect("Failed to read syntax resource as UTF-8")
            .to_string()
    })
}

/// List all available syntax topics
pub fn list_available_topics() -> Vec<String> {
    SYNTAX_DIR
        .find("*.md")
        .map(|files| {
            files
                .into_iter()
                .filter_map(|file| {
                    file.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::mcp::query_syntax::{get_syntax_content, list_available_topics};

    #[test]
    fn test_get_syntax_content() {
        let content = get_syntax_content("core");
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("## Core grammar"));
    }

    #[test]
    fn test_list_available_topics() {
        let topics = list_available_topics();
        assert!(topics.contains(&"core".to_string()));
        assert!(topics.contains(&"regex".to_string()));
    }

    #[test]
    fn test_get_nonexistent_topic() {
        let content = get_syntax_content("nonexistent");
        assert!(content.is_none());
    }
}
