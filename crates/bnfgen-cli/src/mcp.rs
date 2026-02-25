mod generate;
mod resource;

use crate::app::App;
use crate::mcp::generate::{GenerationRequest, GenerationResponse};
use crate::mcp::resource::BnfgenResources;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    ErrorCode, ListResourcesResult, PaginatedRequestParams, ReadResourceRequestParams,
    ReadResourceResult, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_handler, tool_router, Json, RoleServer, ServerHandler};
use typed_builder::TypedBuilder;

pub struct BnfgenMCP {
    tool_router: ToolRouter<Self>,
    settings: BnfgenSettings,
    resource: BnfgenResources,
}

#[derive(TypedBuilder, Clone)]
pub struct BnfgenSettings {
    /// The maximum number of generation attempts before giving up (default: 100)
    #[builder(default=Some(100))]
    pub max_attempts: Option<usize>,
}

#[tool_router]
impl BnfgenMCP {
    pub fn new(settings: BnfgenSettings) -> Self {
        Self {
            tool_router: Self::tool_router(),
            settings,
            resource: BnfgenResources::new(),
        }
    }

    #[tool(
        description = "Generates random strings based on a provided BNF grammar and a starting symbol."
    )]
    async fn generate(
        &self,
        Parameters(req): Parameters<GenerationRequest>,
    ) -> Result<Json<GenerationResponse>, String> {
        let app = App::new(req.grammar);

        let raw = app.parse().map_err(|e| e.to_string())?;
        let checked = app.lint(raw).map_err(|e| e.to_string())?;

        let outputs = app
            .generate(
                checked,
                req.start_symbol,
                req.count,
                req.seed,
                req.max_depth,
                self.settings.max_attempts,
            )
            .map_err(|e| e.to_string())?;

        Ok(Json(GenerationResponse {
            generated_strings: outputs,
        }))
    }
}

#[tool_handler]
impl ServerHandler for BnfgenMCP {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("This server provides tools for generating random strings based on BNF grammars. Use the 'generate' tool to create strings from a specified grammar and starting symbol.".to_string()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(self.resource.list_resources())
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let content = match self.resource.get_resource_contents(&request.uri) {
            Some(content) => content,
            None => {
                return Err(McpError::new(
                    ErrorCode::RESOURCE_NOT_FOUND,
                    format!("Resource not found: '{}'", request.uri),
                    None,
                ))
            }
        };

        Ok(ReadResourceResult {
            contents: vec![content],
        })
    }
}
