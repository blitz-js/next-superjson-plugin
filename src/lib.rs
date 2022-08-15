use std::{ops::IndexMut, path::{Path, Component}};

use serde::Deserialize;
use swc_plugin::{
    ast::*,
    metadata::{TransformPluginMetadataContextKind, TransformPluginProgramMetadata},
    plugin_transform,
    syntax_pos::DUMMY_SP,
    utils::{prepend_stmt, take::Take, ExprFactory},
};

use utils::*;

mod utils;

static SSG_EXPORTS: &[&str; 2] = &["getStaticProps", "getServerSideProps"];

struct NextSuperJsonTransformer {
    excluded: Vec<String>,

    ssg_prop_export_pos: Option<usize>,
    ssg_prop_export_decl_pos: Option<usize>,
    ssg_prop_export_spec_pos: Option<usize>,

    ssg_prop_ident_pos: Option<usize>,
    ssg_prop_ident_decl_pos: Option<usize>,
    ssg_prop_ident_spec_pos: Option<usize>,

    skip_ssg_prop: bool,

    page_pos: Option<usize>,
    page_spec_pos: Option<usize>,

    skip_page: bool,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub excluded: Vec<String>,
}

pub fn plugin(config: Config) -> impl VisitMut + Fold {
    as_folder(transform(config))
}

pub fn transform(config: Config) -> impl VisitMut {
    NextSuperJsonTransformer {
        excluded: config.excluded,

        ssg_prop_export_pos: Default::default(),
        ssg_prop_export_decl_pos: Default::default(),
        ssg_prop_export_spec_pos: Default::default(),

        ssg_prop_ident_pos: Default::default(),
        ssg_prop_ident_decl_pos: Default::default(),
        ssg_prop_ident_spec_pos: Default::default(),

        skip_ssg_prop: Default::default(),

        page_pos: Default::default(),
        page_spec_pos: Default::default(),

        skip_page: Default::default(),
    }
}

impl Fold for NextSuperJsonTransformer {}

impl VisitMut for NextSuperJsonTransformer {
    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        self.find_ssg_prop(items);

        if self.ssg_prop_export_pos.is_none() {
            return;
        }

        self.find_page(items);

        if self.page_pos.is_none() {
            return;
        }

        let mut new_items = vec![];

        for (pos, item) in items.iter_mut().enumerate() {
            if self.ssg_prop_ident_pos.is_some()
                && pos == self.ssg_prop_ident_pos.unwrap()
                && !self.skip_ssg_prop
            {
                match item {
                    // gSSP = ..
                    // =>
                    // gSSP = wrap(.., excluded)
                    ModuleItem::Stmt(Stmt::Expr(ExprStmt { expr, .. })) => {
                        let assign_expr = expr.as_mut_assign().unwrap();

                        assign_expr.right = Box::new(Expr::Call(CallExpr {
                            args: vec![assign_expr.right.take().as_arg(), self.excluded_expr()],
                            callee: Ident::new("_withSuperJSONProps".into(), DUMMY_SP).as_callee(),
                            span: DUMMY_SP,
                            type_args: None,
                        }));

                        new_items.push(item.take());
                    }
                    ModuleItem::Stmt(Stmt::Decl(decl)) => match decl {
                        // function gSSP ..
                        // =>
                        // const gSSP = wrap(.., excluded)
                        Decl::Fn(fn_decl) => {
                            *decl = Decl::Var(VarDecl {
                                declare: false,
                                decls: vec![VarDeclarator {
                                    definite: false,
                                    init: Some(Box::new(Expr::Call(CallExpr {
                                        span: DUMMY_SP,
                                        callee: Ident::new("_withSuperJSONProps".into(), DUMMY_SP)
                                            .as_callee(),
                                        args: vec![
                                            ExprOrSpread {
                                                spread: None,
                                                expr: Box::new(Expr::Fn(FnExpr {
                                                    function: fn_decl.function.take(),
                                                    ident: None,
                                                })),
                                            },
                                            self.excluded_expr(),
                                        ],
                                        type_args: Default::default(),
                                    }))),
                                    name: Pat::Ident(BindingIdent {
                                        id: fn_decl.ident.take(),
                                        type_ann: None,
                                    }),
                                    span: DUMMY_SP,
                                }],
                                kind: VarDeclKind::Const,
                                span: DUMMY_SP,
                            });

                            new_items.push(item.take());
                        }
                        // const gSSP = ..
                        // =>
                        // const gSSP = wrap(.., excluded)
                        Decl::Var(var_decl) => {
                            let v = var_decl
                                .decls
                                .index_mut(self.ssg_prop_ident_decl_pos.unwrap());

                            v.init = Some(Box::new(Expr::Call(CallExpr {
                                span: DUMMY_SP,
                                callee: Ident::new("_withSuperJSONProps".into(), DUMMY_SP)
                                    .as_callee(),
                                args: vec![v.init.take().unwrap().as_arg(), self.excluded_expr()],
                                type_args: None,
                            })));

                            new_items.push(item.take());
                        }
                        _ => {}
                    },
                    // export function not_gSSP() ..
                    // =>
                    // export const not_gSSP = wrap(function not_gSSP()..)
                    ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                        decl: export_decl,
                        ..
                    })) => match export_decl {
                        Decl::Fn(fn_decl) => {
                            *export_decl = Decl::Var(VarDecl {
                                declare: false,
                                decls: vec![VarDeclarator {
                                    definite: false,
                                    init: Some(Box::new(Expr::Call(CallExpr {
                                        span: DUMMY_SP,
                                        callee: Ident::new("_withSuperJSONProps".into(), DUMMY_SP)
                                            .as_callee(),
                                        args: vec![
                                            ExprOrSpread {
                                                spread: None,
                                                expr: Box::new(Expr::Fn(FnExpr {
                                                    function: fn_decl.function.take(),
                                                    ident: None,
                                                })),
                                            },
                                            self.excluded_expr(),
                                        ],
                                        type_args: Default::default(),
                                    }))),
                                    name: Pat::Ident(BindingIdent {
                                        id: fn_decl.ident.take(),
                                        type_ann: None,
                                    }),
                                    span: DUMMY_SP,
                                }],
                                kind: VarDeclKind::Const,
                                span: DUMMY_SP,
                            });

                            new_items.push(item.take());
                        }
                        // export const not_gSSP = ..
                        // =>
                        // export const not_gSSP = wrap(..)
                        Decl::Var(var_decl) => {
                            let v = var_decl
                                .decls
                                .index_mut(self.ssg_prop_ident_decl_pos.unwrap());

                            v.init = Some(Box::new(Expr::Call(CallExpr {
                                span: DUMMY_SP,
                                callee: Ident::new("_withSuperJSONProps".into(), DUMMY_SP)
                                    .as_callee(),
                                args: vec![v.init.take().unwrap().as_arg(), self.excluded_expr()],
                                type_args: None,
                            })));

                            new_items.push(item.take());
                        }
                        _ => {}
                    },
                    // import { not_gSSP as gSSP } from '..' <-
                    // export { gSSP }
                    // =>
                    // import { not_gSSP as _NEXT_SUPERJSON_IMPORTED_PROPS } from '..'
                    // const  _NEXT_SUPERJSON_SSG_PROPS = wrap(_NEXT_SUPERJSON_IMPORTED_PROPS)
                    // export { _NEXT_SUPERJSON_SSG_PROPS as gSSP }
                    //
                    // import { not_gSSP } from '..' <-
                    // export { not_gSSP as gSSP }
                    // =>
                    // import { not_gSSP as _NEXT_SUPERJSON_IMPORTED_PROPS } from '..'
                    // const _NEXT_SUPERJSON_SSG_PROPS = wrap(_NEXT_SUPERJSON_IMPORTED_PROPS)
                    // export { _NEXT_SUPERJSON_SSG_PROPS as gSSP }
                    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                        specifiers, ..
                    })) => {
                        let s = specifiers
                            .index_mut(self.ssg_prop_ident_spec_pos.unwrap())
                            .as_mut_named()
                            .unwrap();

                        // imported: None, local: not_gSSP
                        // =>
                        // imported: not_gSSP, local: _NEXT_SUPERJSON_IMPORTED_PROPS
                        if s.imported.is_none() {
                            s.imported = Some(ModuleExportName::Ident(s.local.take()));
                        }

                        s.local = Ident::new("_NEXT_SUPERJSON_IMPORTED_PROPS".into(), DUMMY_SP);

                        new_items.push(item.take());

                        new_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(VarDecl {
                            declare: false,
                            decls: vec![VarDeclarator {
                                definite: false,
                                init: Some(Box::new(Expr::Call(CallExpr {
                                    span: DUMMY_SP,
                                    callee: Ident::new("_withSuperJSONProps".into(), DUMMY_SP)
                                        .as_callee(),
                                    args: vec![
                                        ExprOrSpread {
                                            spread: None,
                                            expr: Box::new(Expr::Ident(Ident::new(
                                                "_NEXT_SUPERJSON_IMPORTED_PROPS".into(),
                                                DUMMY_SP,
                                            ))),
                                        },
                                        self.excluded_expr(),
                                    ],
                                    type_args: Default::default(),
                                }))),
                                name: Pat::Ident(BindingIdent {
                                    id: Ident::new("_NEXT_SUPERJSON_SSG_PROPS".into(), DUMMY_SP),
                                    type_ann: None,
                                }),
                                span: DUMMY_SP,
                            }],
                            kind: VarDeclKind::Const,
                            span: DUMMY_SP,
                        }))));
                    }
                    _ => {}
                }
            } else {
                if pos == self.ssg_prop_export_pos.unwrap() && !self.skip_ssg_prop {
                    match item {
                        ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                            decl: export_decl,
                            ..
                        })) => {
                            match export_decl {
                                // export function gSSP..
                                // =>
                                // export const gSSP = wrap(.., excluded)
                                Decl::Fn(fn_decl) => {
                                    *export_decl = Decl::Var(VarDecl {
                                        declare: false,
                                        decls: vec![VarDeclarator {
                                            definite: false,
                                            init: Some(Box::new(Expr::Call(CallExpr {
                                                span: DUMMY_SP,
                                                callee: Ident::new(
                                                    "_withSuperJSONProps".into(),
                                                    DUMMY_SP,
                                                )
                                                .as_callee(),
                                                args: vec![
                                                    ExprOrSpread {
                                                        spread: None,
                                                        expr: Box::new(Expr::Fn(FnExpr {
                                                            function: fn_decl.function.take(),
                                                            ident: None,
                                                        })),
                                                    },
                                                    self.excluded_expr(),
                                                ],
                                                type_args: Default::default(),
                                            }))),
                                            name: Pat::Ident(BindingIdent {
                                                id: fn_decl.ident.take(),
                                                type_ann: None,
                                            }),
                                            span: DUMMY_SP,
                                        }],
                                        kind: VarDeclKind::Const,
                                        span: DUMMY_SP,
                                    });

                                    //new_items.push(item.take());
                                }
                                // export const gSSP = ..
                                // =>
                                // export const gSSP = wrap(.., excluded)
                                Decl::Var(var_decl) => {
                                    let v = var_decl
                                        .decls
                                        .index_mut(self.ssg_prop_export_decl_pos.unwrap());

                                    v.init = Some(Box::new(Expr::Call(CallExpr {
                                        span: DUMMY_SP,
                                        callee: Ident::new("_withSuperJSONProps".into(), DUMMY_SP)
                                            .as_callee(),
                                        args: vec![
                                            v.init.take().unwrap().as_arg(),
                                            self.excluded_expr(),
                                        ],
                                        type_args: None,
                                    })));
                                }
                                _ => {}
                            }
                        }

                        ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                            specifiers,
                            src,
                            ..
                        })) => {
                            // export { not_gSSP as gSSP } from '..'
                            // =>
                            // import { not_gSSP as _NEXT_SUPERJSON_IMPORTED_PROPS } from '..'
                            // const _NEXT_SUPERJSON_SSG_PROPS = wrap(_NEXT_SUPERJSON_IMPORTED_PROPS, excluded)
                            // export { _NEXT_SUPERJSON_SSG_PROPS as gSSP }
                            if let Some(src) = src {
                                let s = specifiers
                                    .index_mut(self.ssg_prop_export_spec_pos.unwrap())
                                    .as_mut_named()
                                    .take()
                                    .unwrap();

                                new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(
                                    ImportDecl {
                                        asserts: None,
                                        span: DUMMY_SP,
                                        specifiers: vec![ImportSpecifier::Named(
                                            ImportNamedSpecifier {
                                                imported: Some(s.orig.clone()),
                                                is_type_only: false,
                                                local: Ident::new(
                                                    "_NEXT_SUPERJSON_IMPORTED_PROPS".into(),
                                                    DUMMY_SP,
                                                ),
                                                span: DUMMY_SP,
                                            },
                                        )],
                                        // should clone
                                        src: src.clone(),
                                        type_only: false,
                                    },
                                )));

                                new_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(VarDecl {
                                    declare: false,
                                    decls: vec![VarDeclarator {
                                        definite: false,
                                        init: Some(Box::new(Expr::Call(CallExpr {
                                            span: DUMMY_SP,
                                            callee: Ident::new(
                                                "_withSuperJSONProps".into(),
                                                DUMMY_SP,
                                            )
                                            .as_callee(),
                                            args: vec![
                                                ExprOrSpread {
                                                    spread: None,
                                                    expr: Box::new(Expr::Ident(Ident::new(
                                                        "_NEXT_SUPERJSON_IMPORTED_PROPS".into(),
                                                        DUMMY_SP,
                                                    ))),
                                                },
                                                self.excluded_expr(),
                                            ],
                                            type_args: Default::default(),
                                        }))),
                                        name: Pat::Ident(BindingIdent {
                                            id: Ident::new(
                                                "_NEXT_SUPERJSON_SSG_PROPS".into(),
                                                DUMMY_SP,
                                            ),
                                            type_ann: None,
                                        }),
                                        span: DUMMY_SP,
                                    }],
                                    kind: VarDeclKind::Const,
                                    span: DUMMY_SP,
                                }))));

                                new_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
                                    NamedExport {
                                        asserts: None,
                                        span: DUMMY_SP,
                                        specifiers: vec![ExportSpecifier::Named(
                                            ExportNamedSpecifier {
                                                exported: s.exported.take(),
                                                is_type_only: false,
                                                orig: ModuleExportName::Ident(Ident::new(
                                                    "_NEXT_SUPERJSON_SSG_PROPS".into(),
                                                    DUMMY_SP,
                                                )),
                                                span: DUMMY_SP,
                                            },
                                        )],
                                        src: None,
                                        type_only: false,
                                    },
                                )));

                                specifiers.remove(self.ssg_prop_export_spec_pos.unwrap());

                            // export { gSSP }
                            // export { not_gSSP as gSSP }
                            // =>
                            // export { _NEXT_SUPERJSON_SSG_PROPS as gSSP }
                            } else {
                                let s = specifiers
                                    .index_mut(self.ssg_prop_export_spec_pos.unwrap())
                                    .as_mut_named()
                                    .unwrap();

                                if s.exported.is_none() {
                                    s.exported = Some(s.orig.clone());
                                }

                                // case 1: imported
                                // import { not_gSSP as _NEXT_SUPERJSON_IMPORTED_PROPS }
                                // => _NEXT_SUPERJSON_SSG_PROPS
                                //
                                // case 2: local
                                // const gSSP = () => {}
                                // => gSSP
                                if self.ssg_prop_ident_spec_pos.is_some() {
                                    s.orig = ModuleExportName::Ident(Ident::new(
                                        "_NEXT_SUPERJSON_SSG_PROPS".into(),
                                        DUMMY_SP,
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if pos == self.page_pos.unwrap() && !self.skip_page {
                    match item {
                        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
                            ExportDefaultExpr { expr, .. },
                        )) => {
                            *expr = Box::new(Expr::Call(CallExpr {
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: expr.take(),
                                }],
                                callee: Expr::Ident(Ident::new(
                                    "_withSuperJSONPage".into(),
                                    DUMMY_SP,
                                ))
                                .as_callee(),
                                span: DUMMY_SP,
                                type_args: None,
                            }));
                        }
                        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(
                            ExportDefaultDecl { decl, .. },
                        )) => {
                            // TODO: remove duplicate code
                            match decl {
                                DefaultDecl::Class(class_expr) => {
                                    if class_expr.ident.is_some() {
                                        let id = class_expr.ident.as_ref().unwrap().clone();

                                        new_items.push(ModuleItem::Stmt(Stmt::Decl(
                                            class_expr.take().as_class_decl().unwrap().into(),
                                        )));

                                        *item = ModuleItem::ModuleDecl(
                                            ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                                                expr: Box::new(Expr::Call(CallExpr {
                                                    args: vec![ExprOrSpread {
                                                        spread: None,
                                                        expr: Box::new(Expr::Ident(id)),
                                                    }],
                                                    callee: Expr::Ident(Ident::new(
                                                        "_withSuperJSONPage".into(),
                                                        DUMMY_SP,
                                                    ))
                                                    .as_callee(),
                                                    span: DUMMY_SP,
                                                    type_args: None,
                                                })),
                                                span: DUMMY_SP,
                                            }),
                                        );
                                    } else {
                                        *item = ModuleItem::ModuleDecl(
                                            ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                                                expr: Box::new(Expr::Call(CallExpr {
                                                    args: vec![ExprOrSpread {
                                                        spread: None,
                                                        expr: Box::new(class_expr.take().into()),
                                                    }],
                                                    callee: Expr::Ident(Ident::new(
                                                        "_withSuperJSONPage".into(),
                                                        DUMMY_SP,
                                                    ))
                                                    .as_callee(),
                                                    span: DUMMY_SP,
                                                    type_args: None,
                                                })),
                                                span: DUMMY_SP,
                                            }),
                                        );
                                    }
                                }
                                DefaultDecl::Fn(fn_expr) => {
                                    if fn_expr.ident.is_some() {
                                        let id = fn_expr.ident.as_ref().unwrap().clone();

                                        new_items.push(ModuleItem::Stmt(Stmt::Decl(
                                            fn_expr.take().as_fn_decl().unwrap().into(),
                                        )));

                                        *item = ModuleItem::ModuleDecl(
                                            ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                                                expr: Box::new(Expr::Call(CallExpr {
                                                    args: vec![ExprOrSpread {
                                                        spread: None,
                                                        expr: Box::new(Expr::Ident(id)),
                                                    }],
                                                    callee: Expr::Ident(Ident::new(
                                                        "_withSuperJSONPage".into(),
                                                        DUMMY_SP,
                                                    ))
                                                    .as_callee(),
                                                    span: DUMMY_SP,
                                                    type_args: None,
                                                })),
                                                span: DUMMY_SP,
                                            }),
                                        );
                                    } else {
                                        *item = ModuleItem::ModuleDecl(
                                            ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                                                expr: Box::new(Expr::Call(CallExpr {
                                                    args: vec![ExprOrSpread {
                                                        spread: None,
                                                        expr: Box::new(fn_expr.take().into()),
                                                    }],
                                                    callee: Expr::Ident(Ident::new(
                                                        "_withSuperJSONPage".into(),
                                                        DUMMY_SP,
                                                    ))
                                                    .as_callee(),
                                                    span: DUMMY_SP,
                                                    type_args: None,
                                                })),
                                                span: DUMMY_SP,
                                            }),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                        ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                            specifiers,
                            src,
                            ..
                        })) => {
                            let s = specifiers
                                .index_mut(self.page_spec_pos.unwrap())
                                .as_mut_named()
                                .take()
                                .unwrap();

                            // export { unwrapped as default } from 'src'
                            // =>
                            // import { unwrapped as _NEXT_SUPERJSON_IMPORTED_PAGE } from 'src'
                            // export default wrap(_NEXT_SUPERJSON_IMPORTED_PAGE)
                            if let Some(src) = src {
                                new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(
                                    ImportDecl {
                                        asserts: None,
                                        span: DUMMY_SP,
                                        specifiers: vec![],
                                        src: src.take(),
                                        type_only: false,
                                    },
                                )));

                                new_items.push(ModuleItem::ModuleDecl(
                                    ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                                        expr: Box::new(Expr::Call(CallExpr {
                                            args: vec![ExprOrSpread {
                                                spread: None,
                                                expr: Box::new(Expr::Ident(Ident::new(
                                                    "_NEXT_SUPERJSON_IMPORTED_PAGE".into(),
                                                    DUMMY_SP,
                                                ))),
                                            }],
                                            callee: Expr::Ident(Ident::new(
                                                "_withSuperJSONPage".into(),
                                                DUMMY_SP,
                                            ))
                                            .as_callee(),
                                            span: DUMMY_SP,
                                            type_args: None,
                                        })),
                                        span: DUMMY_SP,
                                    }),
                                ));

                            // export { Page as default }
                            // =>
                            // export default wrap(Page, excluded)
                            } else {
                                if let ModuleExportName::Ident(id) = &s.orig {
                                    new_items.push(ModuleItem::ModuleDecl(
                                        ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                                            expr: Box::new(Expr::Call(CallExpr {
                                                args: vec![ExprOrSpread {
                                                    spread: None,
                                                    // TODO: how to take
                                                    expr: Box::new(Expr::Ident(id.clone())),
                                                }],
                                                callee: Expr::Ident(Ident::new(
                                                    "_withSuperJSONPage".into(),
                                                    DUMMY_SP,
                                                ))
                                                .as_callee(),
                                                span: DUMMY_SP,
                                                type_args: None,
                                            })),
                                            span: DUMMY_SP,
                                        }),
                                    ))
                                }
                            }

                            specifiers.remove(self.page_spec_pos.unwrap());
                        }
                        _ => {}
                    }
                }

                match item {
                    ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                        specifiers,
                        ..
                    })) => {
                        if !specifiers.is_empty() {
                            new_items.push(item.take());
                        }
                    }
                    _ => {
                        new_items.push(item.take());
                    }
                }
            }
        }

        // TODO: these two stmts can be combined
        if !self.skip_ssg_prop {
            prepend_stmt(&mut new_items, superjson_import_decl("withSuperJSONProps"));
        }
        if !self.skip_page {
            prepend_stmt(&mut new_items, superjson_import_decl("withSuperJSONPage"));
        }

        *items = new_items;
    }
}

impl NextSuperJsonTransformer {
    pub fn excluded_expr(&mut self) -> ExprOrSpread {
        ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Array(ArrayLit {
                span: DUMMY_SP,
                elems: self
                    .excluded
                    .iter()
                    .map(|e| {
                        Some(ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: e.clone().into(),
                                raw: None,
                            }))),
                        })
                    })
                    .collect(),
            })),
        }
    }

    pub fn find_ssg_prop(&mut self, items: &mut Vec<ModuleItem>) {
        let mut ssg_prop_ident = None;

        self.ssg_prop_export_pos = items.iter().position(|item| match item {
            // check has ssg props
            ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl { decl, .. })) => match decl {
                Decl::Fn(fn_decl) => SSG_EXPORTS.contains(&&*fn_decl.ident.sym),
                Decl::Var(var_decl) => {
                    self.ssg_prop_export_decl_pos = var_decl.decls.iter().position(|decl| {
                        SSG_EXPORTS.contains(&&*decl.name.as_ident().unwrap().sym)
                    });

                    self.ssg_prop_export_decl_pos.is_some()
                }
                _ => false,
            },
            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                specifiers,
                src,
                ..
            })) => {
                self.ssg_prop_export_spec_pos =
                    specifiers.iter().position(|specifier| match specifier {
                        ExportSpecifier::Named(ExportNamedSpecifier {
                            orig: ModuleExportName::Ident(orig_id),
                            exported,
                            ..
                        }) => {
                            let exported_as = match exported {
                                Some(ModuleExportName::Ident(exported_id)) => &exported_id.sym,
                                _ => &orig_id.sym,
                            };

                            if SSG_EXPORTS.contains(&&**exported_as) {
                                self.skip_ssg_prop = src.is_some()
                                    && (exported.is_none() || (&&**exported_as == &&*orig_id.sym));

                                if !self.skip_ssg_prop {
                                    ssg_prop_ident = Some((*orig_id.sym).to_string());
                                }
                                return true;
                            }
                            false
                        }
                        _ => false,
                    });

                self.ssg_prop_export_spec_pos.is_some()
            }
            _ => false,
        });

        if ssg_prop_ident.is_some() && !self.skip_ssg_prop {
            let mut n = items.len();

            while n > 0 {
                n -= 1;

                if self.ssg_prop_export_pos.unwrap() == n {
                    continue;
                }

                match &items[n] {
                    // gSSP = ..
                    ModuleItem::Stmt(Stmt::Expr(ExprStmt { expr, .. })) => {
                        if expr.is_assign() {
                            let assign = expr.as_assign().unwrap();

                            let left = assign.left.as_ident();

                            if left.is_some() {
                                if assign.op == op!("=")
                                    && &*left.unwrap().sym == ssg_prop_ident.as_ref().unwrap()
                                {
                                    self.ssg_prop_ident_pos = Some(n);
                                    break;
                                }
                            }
                        }
                    }
                    // function gSSP() ..
                    // const gSSP = ..
                    ModuleItem::Stmt(Stmt::Decl(decl)) => match decl {
                        Decl::Fn(fn_decl) => {
                            if &*fn_decl.ident.sym == ssg_prop_ident.as_ref().unwrap() {
                                self.ssg_prop_ident_pos = Some(n);
                                break;
                            }
                        }
                        Decl::Var(var_decl) => {
                            self.ssg_prop_ident_decl_pos = var_decl.decls.iter().position(|decl| {
                                let id = decl.name.as_ident();

                                if id.is_some()
                                    && &*id.unwrap().sym == ssg_prop_ident.as_ref().unwrap()
                                {
                                    self.ssg_prop_ident_pos = Some(n);
                                    return true;
                                }

                                false
                            });

                            if self.ssg_prop_ident_decl_pos.is_some() {
                                break;
                            }
                        }
                        _ => {}
                    },

                    // export function not_gSSP() ..
                    // export const not_gSSP = ..
                    ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                        decl: export_decl,
                        ..
                    })) => match export_decl {
                        Decl::Fn(fn_decl) => {
                            if &*fn_decl.ident.sym == ssg_prop_ident.as_ref().unwrap() {
                                self.ssg_prop_ident_pos = Some(n);
                                break;
                            }
                        }
                        Decl::Var(var_decl) => {
                            self.ssg_prop_ident_decl_pos = var_decl.decls.iter().position(|decl| {
                                let id = decl.name.as_ident();

                                if id.is_some()
                                    && &*id.unwrap().sym == ssg_prop_ident.as_ref().unwrap()
                                {
                                    self.ssg_prop_ident_pos = Some(n);
                                    return true;
                                }

                                false
                            });

                            if self.ssg_prop_ident_decl_pos.is_some() {
                                break;
                            }
                        }
                        _ => {}
                    },
                    // import { gSSP } from '..'
                    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                        specifiers, ..
                    })) => {
                        self.ssg_prop_ident_spec_pos = specifiers.iter().position(|specifier| {
                            if let ImportSpecifier::Named(ImportNamedSpecifier {
                                local,
                                imported,
                                ..
                            }) = specifier
                            {
                                if &*local.sym == ssg_prop_ident.as_ref().unwrap() {
                                    if imported.is_some() {
                                        if let ModuleExportName::Ident(ident) =
                                            imported.as_ref().unwrap()
                                        {
                                            self.skip_ssg_prop = SSG_EXPORTS.contains(&&*ident.sym);
                                        }
                                    }

                                    self.ssg_prop_ident_pos = Some(n);
                                    return true;
                                }
                            }
                            false
                        });

                        if self.ssg_prop_ident_pos.is_some() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn find_page(&mut self, items: &Vec<ModuleItem>) {
        self.page_pos = items.iter().position(|item| match item {
            // check has page
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(_)) => true,
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(_)) => true,
            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                specifiers,
                src,
                ..
            })) => {
                self.page_spec_pos = specifiers.iter().position(|spec| match spec {
                    ExportSpecifier::Named(ExportNamedSpecifier {
                        orig: ModuleExportName::Ident(Ident { sym, .. }),
                        exported,
                        ..
                    }) => match exported {
                        Some(ModuleExportName::Ident(Ident {
                            sym: exported_sym, ..
                        })) => {
                            self.skip_page =
                                exported_sym == "default" && sym == "default" && src.is_some();
                            exported_sym == "default"
                        }
                        _ => {
                            // export { default } from 'source' -> skip
                            self.skip_page = src.is_some() && sym == "default";
                            self.skip_page
                        }
                    },
                    _ => false,
                });

                self.page_spec_pos.is_some()
            }
            _ => false,
        })
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<Config>(&_metadata.get_transform_plugin_config().unwrap())
        .expect("Failed to parse plugin config");

    match _metadata.get_context(&TransformPluginMetadataContextKind::Filename) {
        Some(s) => {

            let mut path = Path::new(&s).components();

            // check file is under 'path' directory
            let is_page = path.any(|cmp| match cmp {
                Component::Normal(str) => str.to_str().unwrap_or_default() == "pages",
                _ => false,
            });

            if is_page {
                return program.fold_with(&mut plugin(config));
            }
            program
        }
        None => program,
    }
}
