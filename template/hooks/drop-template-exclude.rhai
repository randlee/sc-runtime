// The template Cargo.toml excludes crates whose manifests are not valid
// Cargo until this expansion finishes. Replace it with the expanded member
// glob, prefix package names with the project name, then drop this hook so
// it is not part of the generated project.
file::delete("Cargo.toml");
file::rename("workspace.template.toml", "Cargo.toml");
let project = variable::get("project-name");
system::command("python3", ["hooks/prefix_packages.py", project, "."]);
file::delete("hooks/prefix_packages.py");
file::delete("hooks/drop-template-exclude.rhai");
file::delete("hooks");
