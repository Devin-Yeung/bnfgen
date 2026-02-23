mod generate;

use crate::app::App;
use crate::mcp::generate::{GenerationRequest, GenerationResponse};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router, Json, ServerHandler};

#[derive(Clone)]
pub struct BnfgenMCP {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl BnfgenMCP {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
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
}
