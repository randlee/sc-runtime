//! Exercise the actual MCP input-schema boundary with the canonical DTO.
fn main() {
    let schema = rmcp::handler::server::common::schema_for_input::<sc_cron_commands::CronRequest>()
        .expect("canonical request accepted by RMCP");
    let tool = rmcp::model::Tool::new("cron", "standard cron operations", schema);
    assert_eq!(tool.input_schema["type"], "object");
    assert_eq!(tool.input_schema["oneOf"].as_array().unwrap().len(), 3);
}
