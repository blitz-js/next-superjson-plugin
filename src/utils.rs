use swc_common::DUMMY_SP;

use swc_plugin::ast::*;

pub fn superjson_import_decl(superjson_import_name: &str) -> ModuleItem {
    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        asserts: None,
        span: DUMMY_SP,
        type_only: false,
        specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
            local: Ident {
                sym: format!("_{superjson_import_name}").into(),
                span: DUMMY_SP,
                optional: false,
            },
            span: DUMMY_SP,
            imported: Some(ModuleExportName::Ident(Ident {
                //sym: superjson_import_name.into(),
                sym: superjson_import_name.into(),
                span: DUMMY_SP,
                optional: false,
            })),
            is_type_only: false,
        })],
        src: Str {
            span: DUMMY_SP,
            value: "next-superjson-plugin/tools".into(),
            raw: None,
        },
    }))
}
