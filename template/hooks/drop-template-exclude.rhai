// The template Cargo.toml excludes crates whose manifests are not valid
// Cargo until this expansion finishes. Replace it with the expanded member
// glob, then drop this hook so it is not part of the generated project.
file::delete("Cargo.toml");
file::rename("workspace.template.toml", "Cargo.toml");
file::delete("hooks/drop-template-exclude.rhai");
file::delete("hooks");
