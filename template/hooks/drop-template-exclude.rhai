// Parent-approved C1 declaration prerequisite; C3 owns later generator changes.
if variable::get("cron_commands") && !variable::get("cron") {
    throw "cron_commands requires cron=true";
}
// The template Cargo.toml excludes crates whose manifests are not valid
// Cargo until this expansion finishes. Replace it with the expanded member
// glob, prefix package names with the project name, then drop this hook so
// it is not part of the generated project.
file::delete("Cargo.toml");
file::rename("workspace.template.toml", "Cargo.toml");
let project = variable::get("project-name");
let db = variable::get("db");
// Success with empty stdout returns (), and a non-zero exit throws.
system::command("python3", ["hooks/prefix_packages.py", project, ".", db]);
// C2 initial policy handoff; C3 owns subsequent generator evolution.
system::command("python3", ["boundaries/render.py", "."]);
file::delete("boundaries/render.py");
file::delete("hooks/prefix_packages.py");
file::delete("hooks/drop-template-exclude.rhai");
file::delete("hooks");
