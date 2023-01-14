use std::vec;

use swc_common::util::take::Take;
use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::{is_valid_ident, prepend_stmts},
        visit::*,
    },
};

use crate::Config;

static DIRECTIVE: &str = "data-superjson";
static SERIALIZER_FUNCTION: &str = "serialize";
static DESERIALIZER_COMPONENT: &str = "SuperJSONComponent";
static DESERIALIZER_PROPS_ATTR: &str = "props";
static DESERIALIZER_PROPS_COMPONENT: &str = "component";
static TOOLS_SRC: &str = "next-superjson-plugin/tools";
static CLIENT_SRC: &str = "next-superjson-plugin/client";

struct AppTransformer {
    transformed: bool,
}

pub fn transform_app(_: Config) -> impl VisitMut {
    AppTransformer { transformed: false }
}

trait JSXUtil {
    fn as_expr(&self) -> Expr;
}

impl JSXUtil for JSXMemberExpr {
    fn as_expr(&self) -> Expr {
        match &self.obj {
            JSXObject::Ident(id) => id.clone().into(),
            JSXObject::JSXMemberExpr(member) => MemberExpr {
                obj: Box::new(member.as_expr()),
                prop: MemberProp::Ident(member.prop.clone()),
                span: DUMMY_SP,
            }
            .into(),
        }
    }
}

impl JSXUtil for JSXElementName {
    fn as_expr(&self) -> Expr {
        match self {
            JSXElementName::Ident(id) => {
                if is_valid_ident(&id.sym) {
                    id.clone().into()
                } else {
                    Lit::Str(id.sym.clone().into()).into()
                }
            }
            JSXElementName::JSXMemberExpr(member) => MemberExpr {
                obj: Box::new(member.as_expr()),
                prop: member.prop.clone().into(),
                span: DUMMY_SP,
            }
            .into(),
            // namespace cannot be component
            _ => unreachable!(),
        }
    }
}

impl VisitMut for AppTransformer {
    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        items.visit_mut_children_with(self);

        if self.transformed {
            // add import decl

            prepend_stmts(
                items,
                vec![
                    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                        specifiers: vec![ImportNamedSpecifier {
                            local: Ident::new(SERIALIZER_FUNCTION.into(), DUMMY_SP),
                            span: DUMMY_SP,
                            imported: None,
                            is_type_only: false,
                        }
                        .into()],
                        src: Box::new(TOOLS_SRC.into()),
                        ..ImportDecl::dummy()
                    })),
                    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                        specifiers: vec![ImportDefaultSpecifier {
                            local: Ident::new(DESERIALIZER_COMPONENT.into(), DUMMY_SP),
                            span: DUMMY_SP,
                        }
                        .into()],
                        src: Box::new(CLIENT_SRC.into()),
                        ..ImportDecl::dummy()
                    })),
                ]
                .into_iter(),
            );
        }
    }

    fn visit_mut_jsx_element(&mut self, elem: &mut JSXElement) {
        elem.visit_mut_children_with(self);

        let mut found = false;

        // find and remove data-superjson directive
        elem.opening.attrs.retain(|attr_or_spread| {
            if let JSXAttrOrSpread::JSXAttr(JSXAttr {
                name: JSXAttrName::Ident(id),
                ..
            }) = attr_or_spread
            {
                if &*id.sym == DIRECTIVE {
                    found = true;
                    return false;
                }
            }
            true
        });

        if found {
            // attrs -> obj props
            let list: Vec<PropOrSpread> = elem
                .opening
                .attrs
                .take()
                .into_iter()
                .map(|attr_or_spread| match attr_or_spread {
                    JSXAttrOrSpread::JSXAttr(attr) => {
                        let key: PropName = match attr.name {
                            JSXAttrName::Ident(id) => id.into(),
                            JSXAttrName::JSXNamespacedName(ns_name) => PropName::Str(
                                format!("{}:{}", ns_name.ns.sym, ns_name.name.sym).into(),
                            ),
                        };

                        let value: Box<Expr> = match attr.value {
                            Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                expr: JSXExpr::Expr(expr),
                                ..
                            })) => expr,
                            Some(JSXAttrValue::JSXElement(element)) => {
                                Box::new(Expr::JSXElement(element))
                            }
                            Some(JSXAttrValue::JSXFragment(fragment)) => {
                                Box::new(Expr::JSXFragment(fragment))
                            }
                            Some(JSXAttrValue::Lit(lit)) => lit.into(),
                            None => Box::new(Expr::Lit(Lit::Bool(Bool {
                                value: true,
                                span: DUMMY_SP,
                            }))),
                            _ => unreachable!(),
                        };

                        Box::new(Prop::KeyValue(KeyValueProp { key, value })).into()
                    }
                    JSXAttrOrSpread::SpreadElement(spread) => SpreadElement {
                        expr: spread.expr,
                        dot3_token: DUMMY_SP,
                    }
                    .into(),
                })
                .collect();

            // replace attrs
            elem.opening.attrs = vec![
                JSXAttr {
                    name: Ident::new(DESERIALIZER_PROPS_ATTR.into(), DUMMY_SP).into(),
                    span: DUMMY_SP,
                    value: Some(
                        JSXExprContainer {
                            expr: Box::new(Expr::Call(CallExpr {
                                args: vec![Expr::Object(ObjectLit {
                                    span: DUMMY_SP,
                                    props: list,
                                })
                                .into()],
                                callee: Box::new(Expr::Ident(Ident::new(
                                    SERIALIZER_FUNCTION.into(),
                                    DUMMY_SP,
                                )))
                                .into(),
                                span: DUMMY_SP,
                                type_args: None,
                            }))
                            .into(),
                            span: DUMMY_SP,
                        }
                        .into(),
                    ),
                }
                .into(),
                JSXAttr {
                    name: Ident::new(DESERIALIZER_PROPS_COMPONENT.into(), DUMMY_SP).into(),
                    span: DUMMY_SP,
                    value: Some(
                        JSXExprContainer {
                            expr: Box::new(elem.opening.name.as_expr()).into(),
                            span: DUMMY_SP,
                        }
                        .into(),
                    ),
                }
                .into(),
            ];

            // change element name
            elem.opening.name = Ident::new(DESERIALIZER_COMPONENT.into(), DUMMY_SP).into();

            if let Some(closing) = &mut elem.closing {
                closing.name = Ident::new(DESERIALIZER_COMPONENT.into(), DUMMY_SP).into();
            }

            self.transformed = true;
        }
    }
}
