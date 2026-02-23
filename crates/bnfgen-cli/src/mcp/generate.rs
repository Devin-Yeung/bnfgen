use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_count() -> usize {
    1
}

fn default_max_steps() -> Option<usize> {
    Some(1000)
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GenerationRequest {
    /// This defines the starting symbol for generation, which is typically the main non-terminal in
    /// the grammar. For example, if your grammar defines a language with a starting symbol "S",
    /// you would set this field to "S". The generator will then use this symbol as the entry point
    /// to produce random strings based on the production rules defined in the grammar.
    pub start_symbol: String,
    /// The BNF grammar itself, provided as a string. This should be a valid BNF grammar that defines
    /// the structure of the language you want to generate strings from.
    pub grammar: String,
    /// The number of random strings to generate from the grammar. This field specifies how many
    /// unique strings the generator should produce based on the provided grammar and starting symbol.
    #[serde(default = "default_count")]
    pub count: usize,
    /// An optional random seed for reproducible generation. If provided, the generator will produce
    /// the same set of random strings for the same input grammar and starting symbol across different runs
    pub seed: Option<u64>,
    /// An optional maximum depth for generation attempts. This limits the number of steps the generator
    /// will take when trying to generate a string before it gives up and retries with a fresh attempt.
    #[serde(default = "default_max_steps")]
    pub max_depth: Option<usize>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GenerationResponse {
    /// A list of generated strings that conform to the provided BNF grammar and starting symbol.
    pub generated_strings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use crate::mcp::generate::GenerationRequest;

    #[test]
    fn test_request_schema() {
        let schema = schemars::schema_for!(GenerationRequest);
        insta::assert_json_snapshot!(schema);
    }
}
