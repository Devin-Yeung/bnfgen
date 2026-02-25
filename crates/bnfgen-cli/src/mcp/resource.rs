use gray_matter::engine::YAML;
use gray_matter::Matter;
use include_dir::{include_dir, Dir};
use rmcp::model::{ListResourcesResult, RawResource, Resource, ResourceContents};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

static RESOURCE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/resource");

#[derive(Serialize, Deserialize, Debug)]
pub struct BnfgenResources {
    resources: HashMap<String, BnfgenResource>,
}

impl BnfgenResources {
    pub fn new() -> Self {
        let mut resources = HashMap::new();

        for dir in RESOURCE_DIR.dirs() {
            for file in dir.files() {
                let path = file.path().to_str().unwrap();

                if path.ends_with(".md") {
                    let markdown_content = file
                        .contents_utf8()
                        .expect("Failed to read markdown resource as UTF-8");
                    let resource = BnfgenResource::from_markdown(markdown_content);
                    resources.insert(resource.meta.uri.clone(), resource);
                }
            }
        }

        BnfgenResources { resources }
    }

    pub fn list_resources(&self) -> ListResourcesResult {
        let resources = self
            .resources
            .values()
            .map(|res| res.mcp_resource())
            .collect();

        ListResourcesResult {
            meta: None,
            next_cursor: None,
            resources,
        }
    }

    pub fn get_resource_contents(&self, uri: &str) -> Option<ResourceContents> {
        self.resources
            .get(uri)
            .map(|res| ResourceContents::text(res.content.clone(), res.meta.uri.clone()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BnfgenResource {
    meta: ResourceMeta,
    content: String,
}

impl BnfgenResource {
    pub fn mcp_resource(&self) -> Resource {
        Resource {
            raw: RawResource {
                uri: self.meta.uri.clone(),
                name: self.meta.name.clone(),
                title: None,
                description: None,
                mime_type: Some("text".to_string()),
                size: None,
                icons: None,
                meta: None,
            },
            annotations: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceMeta {
    uri: String,
    name: String,
}

impl BnfgenResource {
    pub fn from_markdown<S: AsRef<str>>(source: S) -> Self {
        let matter = Matter::<YAML>::new();
        let entity = matter
            .parse::<ResourceMeta>(source.as_ref())
            .expect("Failed to parse markdown resource");

        BnfgenResource {
            meta: entity.data.expect("Missing metadata in markdown resource"),
            content: entity.content,
        }
    }
}
