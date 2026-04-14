pub mod discover;
mod registry;
pub mod remote;

pub use discover::{
    compute_skill_hash, discover_skills, files_and_hash, normalize_tag_filter, read_skill_manifest,
    read_skill_manifest_version, resolve_skills_root, SkillManifestData,
};
pub use registry::{
    generate_registry, generate_registry_merged, DeprecatedEntry, SkillMetadata, SkillsRegistry,
};
