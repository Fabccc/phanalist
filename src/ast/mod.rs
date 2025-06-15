use std::{
    collections::HashMap,
    path::{self, Path},
};

use mago_interner::ThreadedInterner;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_project::module::Module;
use mago_source::SourceManager;
use mago_syntax::ast::Program;
use walkdir::WalkDir;

pub(crate) struct AST {
    /// The tree that stores filepath to (Module, Program) pairs.
    pub tree: HashMap<String, (Module, Program)>,
}

/// Load all source files from the given workspace path into a SourceManager.
/// The workspace path is expected to be a directory containing source files.
pub(crate) fn load(
    workspace_path: &String,
    interner: &ThreadedInterner,
) -> Result<SourceManager, String> {
    let source_manager = SourceManager::new(interner.clone());

    let iter = WalkDir::new(workspace_path).into_iter().filter_entry(|e| {
        if let Some(file_name) = e.file_name().to_str() {
            !file_name.starts_with(".")
        } else {
            true // Include entries with non-UTF-8 filenames
        }
    });

    for entry in iter {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                let entry_path = entry.path();
                let name = match entry_path.strip_prefix(workspace_path) {
                    Ok(rel_path) => rel_path.display().to_string(),
                    Err(_) => entry_path.display().to_string(),
                };
                source_manager.insert_path(
                    name,
                    entry.into_path(),
                    mago_source::SourceCategory::UserDefined,
                );
            }
        }
    }

    Ok(source_manager)
}

pub(crate) fn build_ast(
    threaded_interner: &ThreadedInterner,
    source_manager: &SourceManager,
    php_version: PHPVersion,
) -> Result<AST, String> {
    let source_ids =
        source_manager.source_ids_for_category(mago_source::SourceCategory::UserDefined);

    let mut ast = AST {
        tree: HashMap::new(),
    };
    for source_id in source_ids {
        let source = match source_manager.load(&source_id) {
            Ok(source) => source,
            Err(err) => {
                eprintln!("Error loading source {}: {}", source_id.0, err);
                continue;
            }
        };
        let (module, program) = mago_project::module::Module::build_with_ast(
            &threaded_interner,
            php_version,
            source,
            mago_project::module::ModuleBuildOptions {
                reflect: true,
                validate: false,
            },
        );
        ast.tree.insert(
            threaded_interner.lookup(&source_id.0).to_string(),
            (module, program),
        );
    }

    Ok(ast)
}
