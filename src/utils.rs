use swc_plugin::{
    ast::*,
    syntax_pos::DUMMY_SP,
    utils::{take::Take, ExprFactory},
};

use crate::{SUPERJSON_PROPS_LOCAL, SUPERJSON_PAGE_LOCAL};

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

pub trait Wrapper {
    fn wrap_props(self, excluded: ExprOrSpread) -> Box<Expr>;
    fn wrap_page(self) -> Box<Expr>;
}

impl Wrapper for Box<Expr> {
    fn wrap_props(self, excluded: ExprOrSpread) -> Box<Expr> {
        Box::new(Expr::Call(CallExpr {
            args: vec![self.as_arg(), excluded],
            callee: Ident::new(SUPERJSON_PROPS_LOCAL.into(), DUMMY_SP).as_callee(),
            span: DUMMY_SP,
            type_args: None,
        }))
    }
    fn wrap_page(self) -> Box<Expr> {
        Box::new(Expr::Call(CallExpr {
            args: vec![self.as_arg()],
            callee: Ident::new(SUPERJSON_PAGE_LOCAL.into(), DUMMY_SP).as_callee(),
            span: DUMMY_SP,
            type_args: None,
        }))
    }
}

pub trait DeclUtil {
    fn as_wrapped_var_decl(self, excluded: ExprOrSpread) -> Decl;
}

impl DeclUtil for FnDecl {
    fn as_wrapped_var_decl(mut self, excluded: ExprOrSpread) -> Decl {
        Decl::Var(VarDecl {
            declare: false,
            decls: vec![VarDeclarator {
                definite: false,
                init: Some(Box::new(Expr::Fn(FnExpr {
                    function: self.function.take(),
                    ident: None,
                })).wrap_props(excluded)),
                name: Pat::Ident(BindingIdent {
                    id: self.ident.take(),
                    type_ann: None,
                }),
                span: DUMMY_SP,
            }],
            kind: VarDeclKind::Const,
            span: DUMMY_SP,
        })
    }
}
