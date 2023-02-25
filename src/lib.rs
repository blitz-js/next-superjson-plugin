use std::path::{Component, Path};

use serde::Deserialize;
use swc_core::{
    ecma::{ast::*, visit::*},
    plugin::{
        metadata::TransformPluginMetadataContextKind, plugin_transform,
        proxies::TransformPluginProgramMetadata,
    },
};

use app::*;
use page::*;

pub mod app;
pub mod page;
mod utils;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub excluded: Vec<String>,
}

pub enum DirType {
    Page,
    App,
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    let raw_cwd = _metadata
        .get_context(&TransformPluginMetadataContextKind::Cwd)
        .unwrap();

    let raw_path = _metadata
        .get_context(&TransformPluginMetadataContextKind::Filename)
        .unwrap();

    // Windows path separator -> Unix path separator
    let cwd = &raw_cwd.replace('\\', "/");
    let path = &raw_path.replace('\\', "/");

    if let Some(relative_path) = path.strip_prefix(cwd) {
        let mut is_page = false;

        for component in Path::new(relative_path).components() {
            match component {
                Component::Normal(str) => match str.to_str().unwrap_or_default() {
                    // skip non-source stuff
                    "node_modules" => {
                        return program;
                    }
                    "pages" => {
                        is_page = true;
                        break;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // consider server components outside the app directory
        let dir_type = if is_page { DirType::Page } else { DirType::App };

        let config = serde_json::from_str::<Config>(
            &_metadata
                .get_transform_plugin_config()
                .unwrap_or_else(|| "{}".to_string()),
        )
        .expect("Failed to parse plugin config");

        match dir_type {
            DirType::Page => program.fold_with(&mut as_folder(transform_page(config))),
            DirType::App => program.fold_with(&mut as_folder(transform_app(config))),
        }
    } else {
        program
    }
}
