use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_count() -> usize {
    1
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GenerationRequest {
    /// This defines the starting symbol for generation, which is typically the main non-terminal in
    /// the grammar. For example, if your grammar defines a language with a starting symbol "S",
    /// you would set this field to "S". The generator will then use this symbol as the entry point
    /// to produce random strings based on the production rules defined in the grammar.
    start_symbol: String,
    /// The BNF grammar itself, provided as a string. This should be a valid BNF grammar that defines
    /// the structure of the language you want to generate strings from.
    grammar: String,
    /// The number of random strings to generate from the grammar. This field specifies how many
    /// unique strings the generator should produce based on the provided grammar and starting symbol.
    #[serde(default = "default_count")]
    count: usize,
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
