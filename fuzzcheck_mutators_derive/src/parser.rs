// #![allow(dead_code)]
// Copyright (c) 2019 makepad

// makepad/render/microserde/derive/src/macro_lib.rs
// commit 1c753ca

use proc_macro::token_stream::IntoIter;
use proc_macro::{Delimiter, Ident, Literal, Punct, Spacing, TokenStream, TokenTree};

use crate::token_builder::*;

pub struct TokenParser {
    backtracked: Option<Box<TokenParser>>,
    iter_stack: Vec<IntoIter>,
    current: Option<TokenTree>,
}

#[inline(never)]
fn token_tree_as_punct(tt: &TokenTree, what: char) -> Option<Punct> {
    if let TokenTree::Punct(p) = tt {
        if p.as_char() == what {
            Some(p.clone())
        } else {
            None
        }
    } else {
        None
    }
}

#[inline(never)]
fn token_tree_as_ident(tt: &TokenTree, what: &str) -> Option<Ident> {
    if let TokenTree::Ident(i) = tt {
        if i.to_string() == what {
            Some(i.clone())
        } else {
            None
        }
    } else {
        None
    }
}

#[derive(Clone)]
pub enum StructKind {
    Struct,
    Tuple,
}

#[derive(Clone)]
pub struct Struct {
    pub visibility: Option<TokenStream>,
    pub ident: Ident,
    pub generics: Generics,
    pub kind: StructKind,
    pub where_clause: Option<WhereClause>,
    pub struct_fields: Vec<StructField>,
}
#[derive(Clone)]
pub struct StructField {
    pub attributes: Vec<TokenStream>,
    pub visibility: Option<TokenStream>,
    pub identifier: Option<Ident>,
    pub ty: TokenStream,
}
#[derive(Clone)]
pub struct Enum {
    pub visibility: Option<TokenStream>,
    pub ident: Ident,
    pub generics: Generics,
    pub where_clause: Option<WhereClause>,
    pub items: Vec<EnumItem>,
}
#[derive(Clone)]
pub struct EnumItem {
    pub attributes: Vec<TokenStream>,
    pub ident: Ident,
    pub data: Option<EnumItemData>,
}
#[derive(Clone)]
pub enum EnumItemData {
    Discriminant(TokenTree),
    Struct(StructKind, Vec<StructField>),
}
#[derive(Clone)]
pub struct LifetimeParam {
    pub ident: TokenStream,
    pub bounds: Option<TokenStream>,
}
#[derive(Clone)]
pub struct TypeParam {
    pub attributes: Vec<TokenStream>,
    pub type_ident: TokenStream,
    pub bounds: Option<TokenStream>,
    pub equal_ty: Option<TokenStream>,
}
#[derive(Clone)]
pub struct Generics {
    pub lifetime_params: Vec<LifetimeParam>,
    pub type_params: Vec<TypeParam>,
}
#[derive(Clone)]
pub struct WhereClauseItem {
    pub for_lifetimes: Option<TokenStream>,
    pub lhs: TokenStream,
    pub rhs: TokenStream,
}
#[derive(Clone)]
pub struct WhereClause {
    pub items: Vec<WhereClauseItem>,
}
impl Enum {
    pub fn to_token_stream(self) -> TokenStream {
        let mut tb = TokenBuilder::new();
        tb.stream_opt(self.visibility);
        tb.add("enum");
        tb.extend(self.ident);
        tb.stream(self.generics.to_token_stream());
        
        let where_clause = self.where_clause.map(|wc| wc.to_token_stream());
        
        tb.stream_opt(where_clause.clone());
        
        tb.push_group(Delimiter::Brace);
        
        for item in self.items {
            for attribute in item.attributes.into_iter() {
                tb.stream(attribute);
            };

            tb.extend(item.ident);

            match item.data {
                Some(EnumItemData::Struct(struct_kind, fields)) => {
                    let delimiter = if matches!(struct_kind, StructKind::Struct) { Delimiter::Brace } else { Delimiter::Parenthesis };
                    tb.push_group(delimiter);
                    for field in fields {
                        tb.stream(field.to_token_stream());
                        tb.add(",");
                    }
                    tb.pop_group(delimiter);
                }
                Some(EnumItemData::Discriminant(discriminant)) => {
                    tb.add("=");
                    tb.extend(discriminant);
                }
                None => {

                }
            }

            tb.add(",");
        }
        tb.pop_group(Delimiter::Brace);

        tb.end()
    }
}

impl Struct {
    pub fn to_token_stream(self) -> TokenStream {
        let mut tb = TokenBuilder::new();
        tb.stream_opt(self.visibility);
        tb.add("struct");
        tb.extend(self.ident);
        tb.stream(self.generics.to_token_stream());
        let delimiter = match self.kind {
            StructKind::Struct => Delimiter::Brace,
            StructKind::Tuple => Delimiter::Parenthesis,
        };
        let where_clause = self.where_clause.map(|wc| wc.to_token_stream());
        if matches!(self.kind, StructKind::Struct) {
            tb.stream_opt(where_clause.clone());
        }
        tb.push_group(delimiter);
        for field in self.struct_fields {
            for attribute in field.attributes.into_iter() {
                tb.stream(attribute);
            }
            tb.stream_opt(field.visibility);

            if let Some(ident) = field.identifier {
                tb.extend(ident);
                tb.add(":");
            }
            tb.stream(field.ty);
            tb.add(",");
        }
        tb.pop_group(delimiter);
        if matches!(self.kind, StructKind::Tuple) {
            tb.stream_opt(where_clause);
        }
        tb.end()
    }
}
impl StructField {
    pub fn to_token_stream(self) -> TokenStream {
        let mut tb = TokenBuilder::new();
        for attribute in self.attributes.into_iter() {
            tb.stream(attribute);
        }
        tb.stream_opt(self.visibility);
        if let Some(ident) = self.identifier {
            tb.extend(ident);
            tb.add(":");
        }
        tb.stream(self.ty);
        tb.end()
    }
}
impl LifetimeParam {
    pub fn to_token_stream(self) -> TokenStream {
        let mut tb = TokenBuilder::new();
        tb.stream(self.ident);
        if let Some(bounds) = self.bounds {
            tb.add(":");
            tb.stream(bounds);
        }
        tb.end()
    }
}
impl TypeParam {
    pub fn to_token_stream(self) -> TokenStream {
        let mut tb = TokenBuilder::new();
        for attribute in self.attributes.into_iter() {
            tb.stream(attribute);
        }
        tb.stream(self.type_ident);
        if let Some(bounds) = self.bounds {
            tb.add(":");
            tb.stream(bounds);
        }
        if let Some(eqty) = self.equal_ty {
            tb.add("=");
            tb.stream(eqty);
        }
        tb.end()
    }
}
impl Generics {
    pub fn to_token_stream(self) -> TokenStream {
        if self.lifetime_params.is_empty() && self.type_params.is_empty() {
            return TokenStream::new();
        }
        let mut tb = TokenBuilder::new();
        tb.add("<");
        for item in self.lifetime_params.into_iter() {
            tb.stream(item.to_token_stream());
            tb.add(",");
        }
        for item in self.type_params.into_iter() {
            tb.stream(item.to_token_stream());
            tb.add(",");
        }
        tb.add(">");
        tb.end()
    }
    pub fn removing_bounds_and_eq_type(&self) -> (Self, Vec<Option<TokenStream>>) {
        let mut bounds = Vec::new();
        let mut new = self.clone();
        for lifetime_param in new.lifetime_params.iter_mut() {
            bounds.push(lifetime_param.bounds.clone());
            lifetime_param.bounds = None;
        }
        for type_param in new.type_params.iter_mut() {
            bounds.push(type_param.bounds.clone());
            type_param.bounds = None;
            type_param.equal_ty = None;
        }
        (new, bounds)
    }
}
impl WhereClauseItem {
    pub fn to_token_stream(self) -> TokenStream {
        let mut tb = TokenBuilder::new();
        tb.stream_opt(self.for_lifetimes);
        tb.stream(self.lhs);
        tb.add(":");
        tb.stream(self.rhs);
        tb.end()
    }
}
impl WhereClause {
    pub fn to_token_stream(self) -> TokenStream {
        let mut tb = TokenBuilder::new();
        tb.add("where");
        for item in self.items.into_iter() {
            tb.stream(item.to_token_stream());
            tb.add(",");
        }
        tb.end()
    }
}

impl TokenParser {
    #[inline(never)]
    pub fn new(start: TokenStream) -> Self {
        let mut ret = Self {
            backtracked: None,
            iter_stack: vec![start.into_iter()],
            current: None,
        };
        ret.advance();
        ret
    }

    #[inline(never)]
    pub fn backtrack(&mut self, ts: TokenStream) {
        if !ts.is_empty() {
            if let Some(backtracked) = &mut self.backtracked {
                backtracked.backtrack(ts)
            } else {
                self.backtracked = Some(Box::new(TokenParser::new(ts)));
            }
        }
    }

    #[inline(never)]
    pub fn peek(&mut self) -> Option<&TokenTree> {
        if let Some(backtracked) = &mut self.backtracked {
            backtracked.peek()
        } else {
            self.current.as_ref()
        }
    }

    #[inline(never)]
    pub fn advance(&mut self) {
        if let Some(backtracked) = &mut self.backtracked {
            backtracked.advance();
            if backtracked.peek().is_none() {
                self.backtracked = None;
            }
            return;
        }
        let last = self.iter_stack.last_mut().unwrap();
        let value = last.next();

        if let Some(tok) = value {
            self.current = Some(tok);
        } else {
            self.current = None;
        }
    }

    #[inline(never)]
    pub fn advance_if<T>(&mut self, cond: impl FnOnce(&TokenTree) -> Option<T>) -> Option<T> {
        let value = self.peek();
        if let Some(tok) = value {
            if let Some(value) = cond(tok) {
                self.advance();
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn open_delim(&mut self, delim: Delimiter) -> bool {
        let iter = if let Some(TokenTree::Group(group)) = self.peek() {
            if group.delimiter() == delim {
                Some(group.stream().into_iter())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(iter) = iter {
            self.iter_stack.push(iter);
            self.advance();
            true
        } else {
            false
        }
    }

    #[inline(never)]
    pub fn open_brace(&mut self) -> bool {
        self.open_delim(Delimiter::Brace)
    }

    #[inline(never)]
    pub fn open_paren(&mut self) -> bool {
        self.open_delim(Delimiter::Parenthesis)
    }

    #[inline(never)]
    pub fn open_bracket(&mut self) -> bool {
        self.open_delim(Delimiter::Bracket)
    }

    #[inline(never)]
    pub fn is_eot(&mut self) -> bool {
        if self.current.is_none() && self.iter_stack.len() != 0 {
            return true;
        } else {
            return false;
        }
    }

    #[inline(never)]
    pub fn eat_eot(&mut self) -> bool {
        // current is None
        if self.is_eot() {
            self.iter_stack.pop();
            if self.iter_stack.len() != 0 {
                self.advance()
            }
            return true;
        }
        return false;
    }

    #[inline(never)]
    pub fn eat_ident(&mut self, what: &str) -> Option<Ident> {
        self.advance_if(|tt| token_tree_as_ident(tt, what))
    }

    // #[inline(never)]
    // pub fn is_punct(&mut self, what: char) -> bool {
    //     // check if our punct is multichar.
    //     if let Some(tt) = self.peek() {
    //         token_tree_as_punct(tt, what).is_some()
    //     } else {
    //         return false;
    //     }
    // }

    #[inline(never)]
    pub fn eat_punct_with_spacing(&mut self, what: char, spacing: Spacing) -> Option<Punct> {
        self.advance_if(|tt| {
            if let TokenTree::Punct(p) = tt {
                if p.as_char() == what && p.spacing() == spacing {
                    Some(p.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    #[inline(never)]
    pub fn eat_punct(&mut self, what: char) -> Option<Punct> {
        self.advance_if(|tt| token_tree_as_punct(tt, what))
    }

    #[inline(never)]
    pub fn eat_any_ident(&mut self) -> Option<Ident> {
        self.advance_if(|tt| {
            if let TokenTree::Ident(id) = tt {
                Some(id.clone())
            } else {
                None
            }
        })
    }

    #[inline(never)]
    pub fn eat_literal(&mut self) -> Option<Literal> {
        self.advance_if(|tt| {
            if let TokenTree::Literal(l) = tt {
                Some(l.clone())
            } else {
                None
            }
        })
    }

    #[inline(never)]
    pub fn eat_type_bound_where_clause_item(&mut self) -> Option<WhereClauseItem> {
        let for_lifetimes = self.eat_for_lifetimes();
        if let Some(ty) = self.eat_type() {
            let lhs = ty;
            if let Some(_) = self.eat_punct(':') {
                let mut rhs = TokenBuilder::new();
                if let Some(typbs) = self.eat_type_param_bounds() {
                    rhs.stream(typbs);
                }
                let rhs = rhs.end();
                return Some(WhereClauseItem {
                    for_lifetimes,
                    lhs,
                    rhs,
                });
            }
        }
        None
    }

    #[inline(never)]
    pub fn eat_lifetime_bounds(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        while let Some(lt) = self.eat_lifetime() {
            tb.stream(lt);
            if let Some(plus) = self.eat_punct('+') {
                tb.extend(plus);
                continue;
            } else {
                break;
            }
        }
        Some(tb.end())
    }

    #[inline(never)]
    pub fn eat_lifetime_where_clause_item(&mut self) -> Option<WhereClauseItem> {
        if let Some(lt) = self.eat_lifetime() {
            let lhs = lt;
            if let Some(_) = self.eat_punct(':') {
                if let Some(lt_bounds) = self.eat_lifetime_bounds() {
                    let rhs = lt_bounds;

                    return Some(WhereClauseItem {
                        for_lifetimes: None,
                        lhs,
                        rhs,
                    });
                }
            }
        }
        None
    }

    #[inline(never)]
    pub fn eat_where_clause_item(&mut self) -> Option<WhereClauseItem> {
        self.eat_lifetime_where_clause_item()
            .or_else(|| self.eat_type_bound_where_clause_item())
    }

    #[inline(never)]
    pub fn eat_where_clause(&mut self) -> Option<WhereClause> {
        if let Some(_) = self.eat_ident("where") {
            let mut items = Vec::new();
            while let Some(clause_item) = self.eat_where_clause_item() {
                items.push(clause_item);
                if let Some(_) = self.eat_punct(',') {
                    continue;
                } else {
                    break;
                }
            }
            Some(WhereClause { items })
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_struct(&mut self) -> Option<Struct> {
        let visibility = self.eat_visibility();
        if let Some(_) = self.eat_ident("struct") {
            if let Some(ident) = self.eat_any_ident() {
                let generics = self.eat_generics();
                if self.open_paren() {
                    let struct_fields = self.eat_tuple_fields();

                    if self.eat_eot() {
                        let where_clause = self.eat_where_clause();
                        if let Some(_) = self.eat_punct(';') {
                            return Some(Struct {
                                visibility,
                                ident,
                                generics,
                                kind: StructKind::Tuple,
                                where_clause,
                                struct_fields,
                            });
                        }
                    }
                } else {
                    // struct struct
                    let where_clause = self.eat_where_clause();
                    if self.open_brace() {
                        let struct_fields = self.eat_struct_fields();

                        if self.eat_eot() {
                            return Some(Struct {
                                visibility,
                                ident,
                                generics,
                                kind: StructKind::Struct,
                                where_clause,
                                struct_fields,
                            });
                        }
                    }
                }
            }
        }
        None
    }

    #[inline(never)]
    pub fn eat_enum_item(&mut self) -> Option<EnumItem> {
        let mut attributes = Vec::new();
        while let Some(attr) = self.eat_outer_attribute() {
            attributes.push(attr);
        }
        let _ = self.eat_visibility();

        if let Some(ident) = self.eat_any_ident() {
            if self.open_paren() {
                let struct_fields = self.eat_tuple_fields();
                self.eat_eot();
                Some(EnumItem {
                    attributes,
                    ident,
                    data: Some(EnumItemData::Struct(StructKind::Tuple, struct_fields)),
                })
            } else if self.open_brace() {
                let struct_fields = self.eat_struct_fields();
                self.eat_eot();
                Some(EnumItem {
                    attributes,
                    ident,
                    data: Some(EnumItemData::Struct(StructKind::Struct, struct_fields)),
                })
            } else if let Some(_) = self.eat_punct('=') {
                let expr: Option<TokenTree> = self
                    .eat_literal()
                    .map(TokenTree::Literal)
                    .or_else(|| self.eat_any_ident().map(TokenTree::Ident))
                    .or_else(|| self.eat_group(Delimiter::Brace))
                    .into();
                if let Some(expr) = expr {
                    Some(EnumItem {
                        attributes,
                        ident,
                        data: Some(EnumItemData::Discriminant(expr)),
                    })
                } else {
                    // self.backtrack(tb.end());
                    None
                }
            } else {
                Some(EnumItem {
                    attributes,
                    ident,
                    data: None,
                })
            }
        } else {
            // self.backtrack(tb.end());
            None
        }
    }

    #[inline(never)]
    pub fn eat_enumeration(&mut self) -> Option<Enum> {
        let visibility = self.eat_visibility();

        if let Some(_) = self.eat_ident("enum") {
            if let Some(ident) = self.eat_any_ident() {
                let generics = self.eat_generics();
                let where_clause = self.eat_where_clause();
                if self.open_brace() {
                    let mut items = Vec::new();
                    while let Some(item) = self.eat_enum_item() {
                        items.push(item);
                        if let Some(_) = self.eat_punct(',') {
                            
                        } else {
                            break;
                        }
                    }
                    if self.eat_eot() {
                        return Some(Enum {
                            visibility,
                            ident,
                            generics,
                            where_clause,
                            items,
                        });
                    }
                }
            }
        }
        // self.backtrack(tb.end());
        return None;
    }

    #[inline(never)]
    pub fn eat_lifetime_param(&mut self) -> Option<LifetimeParam> {
        let mut attributes = Vec::new();
        while let Some(outer_attribute) = self.eat_outer_attribute() {
            attributes.push(outer_attribute);
        }
        if let Some(ident) = self.eat_lifetime_or_label() {
            if let Some(_) = self.eat_punct(':') {
                if let Some(bounds) = self.eat_lifetime_bounds() {
                    Some(LifetimeParam {
                        ident,
                        bounds: Some(bounds),
                    })
                } else {
                    // self.backtrack(tb.end());
                    None
                }
            } else {
                Some(LifetimeParam { ident, bounds: None })
            }
        } else {
            // self.backtrack(tb.end());
            return None;
        }
    }

    #[inline(never)]
    pub fn eat_type_param(&mut self) -> Option<TypeParam> {
        let mut attributes = Vec::new();
        while let Some(attr) = self.eat_outer_attribute() {
            attributes.push(attr);
        }
        if let Some(ident) = self.eat_any_ident() {
            let bounds = if let Some(_) = self.eat_punct(':') {
                if let Some(bounds) = self.eat_type_param_bounds() {
                    Some(bounds)
                } else {
                    // self.backtrack(tb.end());
                    return None;
                }
            } else {
                None
            };
            let equal_ty = if let Some(_) = self.eat_punct('=') {
                if let Some(ty) = self.eat_type() {
                    Some(ty)
                } else {
                    // self.backtrack(tb.end());
                    return None;
                }
            } else {
                None
            };
            Some(TypeParam {
                attributes,
                type_ident: TokenTree::Ident(ident).into(),
                bounds,
                equal_ty,
            })
        } else {
            // self.backtrack(tb.end());
            None
        }
    }

    #[inline(never)]
    pub fn eat_generics(&mut self) -> Generics {
        if let Some(_) = self.eat_punct('<') {
            let mut lifetime_params = Vec::new();
            while let Some(lt_param) = self.eat_lifetime_param() {
                lifetime_params.push(lt_param);
                if let Some(_) = self.eat_punct(',') {}
            }
            let mut type_params = Vec::new();
            while let Some(ty_param) = self.eat_type_param() {
                type_params.push(ty_param);
                if let Some(_) = self.eat_punct(',') {
                } else {
                    break;
                }
            }

            if let Some(_) = self.eat_punct('>') {
                return Generics {
                    lifetime_params,
                    type_params,
                };
            }
        }
        Generics {
            lifetime_params: Vec::new(),
            type_params: Vec::new(),
        }
    }

    #[inline(never)]
    pub fn eat_tuple_fields(&mut self) -> Vec<StructField> {
        if let Some(field) = self.eat_tuple_field() {
            let mut fields = Vec::new();
            fields.push(field);
            while let Some(_) = self.eat_punct(',') {
                if let Some(field) = self.eat_tuple_field() {
                    fields.push(field);
                } else {
                    break;
                }
            }
            fields
        } else {
            Vec::new()
        }
    }

    #[inline(never)]
    pub fn eat_tuple_field(&mut self) -> Option<StructField> {
        let mut attributes = Vec::new();
        while let Some(attribute) = self.eat_outer_attribute() {
            attributes.push(attribute);
        }
        let visibility = self.eat_visibility();
        if let Some(ty) = self.eat_type() {
            Some(StructField {
                attributes,
                visibility,
                identifier: None,
                ty,
            })
        } else {
            // self.backtrack(tb.end());
            None
        }
    }

    #[inline(never)]
    pub fn eat_struct_fields(&mut self) -> Vec<StructField> {
        let mut fields = Vec::new();
        while let Some(field) = self.eat_struct_field() {
            fields.push(field);
            if let Some(_) = self.eat_punct(',') {
            } else {
                break;
            }
        }
        fields
    }

    #[inline(never)]
    pub fn eat_struct_field(&mut self) -> Option<StructField> {
        let mut attributes = vec![];
        while let Some(outer_attribute) = self.eat_outer_attribute() {
            attributes.push(outer_attribute.clone());
        }
        let visibility = self.eat_visibility();
        if let Some(identifier) = self.eat_any_ident() {
            let _ = self.eat_punct(':');
            if let Some(ty) = self.eat_type() {
                Some(StructField {
                    attributes,
                    visibility,
                    identifier: Some(identifier),
                    ty,
                })
            } else {
                // self.backtrack(tb.end());
                None
            }
        } else {
            // self.backtrack(tb.end());
            None
        }
    }

    #[inline(never)]
    pub fn eat_group_angle_bracket(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        // if we have a <, keep running and keep a < stack

        if let Some(ob) = self.eat_punct('<') {
            tb.extend(ob);
            let mut stack = 1;
            // keep eating things till we are at stack 0 for a ">"
            while stack > 0 {
                if let Some(start_arrow) = self.eat_punct_with_spacing('-', Spacing::Joint) {
                    tb.extend(start_arrow);
                    if let Some(end_arrow) = self.eat_punct('>') {
                        tb.extend(end_arrow);
                    }
                }
                if let Some(ob) = self.eat_punct('<') {
                    tb.extend(ob);
                    stack += 1;
                }
                if let Some(cb) = self.eat_punct('>') {
                    tb.extend(cb);
                    stack -= 1;
                } else if self.eat_eot() {
                    // shits broken
                    return None;
                } else {
                    // store info here in generics struct
                    if let Some(current) = self.peek() {
                        tb.extend(current.clone());
                    }
                    self.advance();
                }
            }
            return Some(tb.end());
        } else {
            return None;
        }
    }

    #[inline(never)] // same as lifetime_token because I don't distinguish between ident and keywords
    pub fn eat_lifetime_or_label(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        if let Some(ap) = self.eat_punct('\'') {
            tb.extend(ap);
            if let Some(lifetime) = self.eat_any_ident() {
                tb.extend(lifetime);
            } else {
                // self.backtrack(tb.end());
                return None;
            }
            Some(tb.end())
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_lifetime(&mut self) -> Option<TokenStream> {
        self.eat_lifetime_or_label().or_else(|| {
            let mut tb = TokenBuilder::new();
            if let Some(ap) = self.eat_punct_with_spacing('\'', Spacing::Joint) {
                tb.extend(ap);
                if let Some(anon) = self.eat_punct_with_spacing('_', Spacing::Alone) {
                    tb.extend(anon);
                } else {
                    // self.backtrack(tb.end());
                    return None;
                }
                Some(tb.end())
            } else {
                None
            }
        })
    }

    #[inline(never)]
    pub fn eat_double_colon(&mut self) -> Option<TokenStream> {
        if let Some(c1) = self.eat_punct_with_spacing(':', Spacing::Joint) {
            let mut tb = TokenBuilder::new();
            tb.extend(c1);
            if let Some(c2) = self.eat_punct_with_spacing(':', Spacing::Alone) {
                tb.extend(c2);
                Some(tb.end())
            } else {
                self.backtrack(tb.end());
                None
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_fn_arrow(&mut self) -> Option<TokenStream> {
        if let Some(c1) = self.eat_punct_with_spacing('-', Spacing::Joint) {
            let mut tb = TokenBuilder::new();
            tb.extend(c1);
            if let Some(c2) = self.eat_punct_with_spacing('>', Spacing::Alone) {
                tb.extend(c2);
                Some(tb.end())
            } else {
                // self.backtrack(tb.end());
                None
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_type_path_segment(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        if let Some(ident) = self.eat_any_ident() {
            tb.extend(ident);
            let mut colons_tb = TokenBuilder::new();
            if let Some(colons) = self.eat_double_colon() {
                colons_tb.stream(colons);
            }
            if let Some(generic) = self.eat_group_angle_bracket() {
                tb.stream(colons_tb.end());
                tb.stream(generic);
            } else if let Some(fn_args) = self.eat_group(Delimiter::Parenthesis) {
                tb.stream(colons_tb.end());
                tb.extend(fn_args);
                if let Some(arrow) = self.eat_fn_arrow() {
                    tb.stream(arrow);
                    if let Some(ty) = self.eat_type() {
                        tb.stream(ty);
                    } else {
                        // self.backtrack(tb.end());
                        return None;
                    }
                }
            } else {
                self.backtrack(colons_tb.end());
            }
            Some(tb.end())
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_type_path(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        if let Some(colons) = self.eat_double_colon() {
            tb.stream(colons);
        }
        if let Some(segment) = self.eat_type_path_segment() {
            tb.stream(segment);
        } else {
            //self.backtrack(tb.end());
            return None;
        }
        while let Some(colons) = self.eat_double_colon() {
            tb.stream(colons);
            if let Some(segment) = self.eat_type_path_segment() {
                tb.stream(segment);
            } else {
                //self.backtrack(tb.end());
                return None;
            }
        }
        Some(tb.end())
    }

    #[inline(never)]
    pub fn eat_raw_pointer_type(&mut self) -> Option<TokenStream> {
        if let Some(star) = self.eat_punct('*') {
            let mut tb = TokenBuilder::new();
            tb.extend(star);
            if let Some(ident) = self.eat_ident("cont").or_else(|| self.eat_ident("mut")) {
                tb.extend(ident);
                if let Some(ty) = self.eat_type_no_bounds() {
                    tb.stream(ty);
                    Some(tb.end())
                } else {
                    // self.backtrack(tb.end());
                    return None;
                }
            } else {
                // self.backtrack(tb.end());
                return None;
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_qualified_path_in_type(&mut self) -> Option<TokenStream> {
        // qualified path type
        let mut tb = TokenBuilder::new();
        if let Some(qtp) = self.eat_group_angle_bracket() {
            tb.stream(qtp);

            if let Some(colons) = self.eat_double_colon() {
                tb.stream(colons);
                if let Some(tps) = self.eat_type_path_segment() {
                    tb.stream(tps);

                    while let Some(colons) = self.eat_double_colon() {
                        tb.stream(colons);
                        if let Some(tps) = self.eat_type_path_segment() {
                            tb.stream(tps);
                        } else {
                            // self.backtrack(tb.end());
                            return None;
                        }
                    }
                    Some(tb.end())
                } else {
                    // self.backtrack(tb.end());
                    None
                }
            } else {
                // self.backtrack(tb.end());
                None
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_for_lifetimes(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();

        if let Some(for_ident) = self.eat_ident("for") {
            tb.extend(for_ident);
            if let Some(lifetime_params) = self.eat_group_angle_bracket() {
                tb.stream(lifetime_params);
                Some(tb.end())
            } else {
                // self.backtrack(tb.end());
                None
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_outer_attribute(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        let nbr_sign = self.eat_punct('#')?;
        if self.open_bracket() {
            tb.extend(nbr_sign);
            if let Some(content) = self.eat_any_group() {
                tb.extend(content);
                Some(tb.end())
            } else {
                // self.backtrack(tb.end());
                None
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_simple_path(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        if let Some(db) = self.eat_double_colon() {
            tb.stream(db);
        }
        if let Some(sps) = self.eat_any_ident() {
            // simple path segment, except $crate
            tb.extend(sps);
            while let Some(db) = self.eat_double_colon() {
                if let Some(sps) = self.eat_any_ident() {
                    // simple path segment, except $crate
                    tb.stream(db);
                    tb.extend(sps);
                } else {
                    // self.backtrack(tb.end());
                    return None;
                }
            }
            Some(tb.end())
        } else {
            // self.backtrack(tb.end());
            None
        }
    }

    #[inline(never)]
    pub fn eat_macro_invocation(&mut self) -> Option<TokenStream> {
        if let Some(sp) = self.eat_simple_path() {
            let mut tb = TokenBuilder::new();
            tb.stream(sp);
            if let Some(tree) = self.eat_any_group() {
                tb.extend(tree);
                Some(tb.end())
            } else {
                // self.backtrack(tb.end());
                None
            }
        } else {
            None
        }
    }

    #[inline(never)] // TODO: check for backtracking correctness
    pub fn eat_trait_bound(&mut self) -> Option<TokenStream> {
        if let Some(g) = self.eat_group(Delimiter::Parenthesis) {
            let mut tb = TokenBuilder::new();
            tb.extend(g);
            Some(tb.end())
        } else {
            let q = self.eat_punct('?');
            let for_lt = self.eat_for_lifetimes();

            let mut tb = TokenBuilder::new();

            if let Some(tp) = self.eat_type_path() {
                if let Some(q) = q {
                    tb.extend(q);
                }
                if let Some(for_lt) = for_lt {
                    tb.stream(for_lt);
                }
                tb.stream(tp);

                Some(tb.end())
            } else {
                // self.backtrack(tb.end());
                return None;
            }
        }
    }

    #[inline(never)]
    pub fn eat_trait_object_type_one_bound(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        if let Some(dyn_ident) = self.eat_ident("dyn") {
            tb.extend(dyn_ident);
        }
        if let Some(trait_bound) = self.eat_trait_bound() {
            tb.stream(trait_bound);
            Some(tb.end())
        } else {
            // self.backtrack(tb.end());
            None
        }
    }

    #[inline(never)]
    pub fn eat_impl_trait_type_one_bound(&mut self) -> Option<TokenStream> {
        if let Some(impl_ident) = self.eat_ident("impl") {
            let mut tb = TokenBuilder::new();
            if let Some(trait_bound) = self.eat_trait_bound() {
                tb.extend(impl_ident);
                tb.stream(trait_bound);
                Some(tb.end())
            } else {
                // self.backtrack(tb.end());
                None
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_type_no_bounds(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();

        if let Some(tys) = self.eat_group(Delimiter::Parenthesis) {
            // parenthesized_type
            tb.extend(tys);
        } else if let Some(ittob) = self.eat_impl_trait_type_one_bound() {
            // impl trait one bound
            tb.stream(ittob);
        } else if let Some(itotob) = self.eat_trait_object_type_one_bound() {
            tb.stream(itotob);
        } else if let Some(typath) = self.eat_type_path() {
            // type path
            tb.stream(typath);
        } else if let Some(tuple) = self.eat_group(Delimiter::Parenthesis) {
            // tuple type
            tb.extend(tuple);
        } else if let Some(never) = self.eat_punct('!') {
            // never type
            tb.extend(never);
        } else if let Some(raw_ptr) = self.eat_raw_pointer_type() {
            // raw pointer type
            tb.stream(raw_ptr);
        } else if let Some(amp) = self.eat_punct('&') {
            // reference type
            tb.extend(amp);
            if let Some(lt) = self.eat_lifetime() {
                tb.stream(lt);
            }
            if let Some(mut_ident) = self.eat_ident("mut") {
                tb.extend(mut_ident);
            }
            let ty = self.eat_type_no_bounds()?;
            tb.stream(ty);
        } else if let Some(arr_or_slice) = self.eat_group(Delimiter::Bracket) {
            // array type + slice type
            tb.extend(arr_or_slice);
        } else if let Some(punct) = self.eat_punct('_') {
            // inferred type
            tb.extend(punct);
        } else if let Some(qpit) = self.eat_qualified_path_in_type() {
            // qualified path in type
            tb.stream(qpit);
        } else if let Some(m) = self.eat_macro_invocation() {
            tb.stream(m);
        } else {
            return None;
        }
        return Some(tb.end());
    }

    #[inline(never)]
    pub fn eat_type_param_bounds(&mut self) -> Option<TokenStream> {
        if let Some(tpb) = self.eat_type_param_bound() {
            let mut tb = TokenBuilder::new();
            tb.stream(tpb);
            while let Some(plus) = self.eat_punct('+') {
                tb.extend(plus);
                if let Some(bound) = self.eat_type_param_bound() {
                    tb.stream(bound);
                }
            }
            Some(tb.end())
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_type_param_bound(&mut self) -> Option<TokenStream> {
        self.eat_lifetime().or_else(|| self.eat_trait_bound())
    }

    #[inline(never)]
    pub fn eat_impl_trait_type(&mut self) -> Option<TokenStream> {
        if let Some(impl_ident) = self.eat_ident("impl") {
            let mut tb = TokenBuilder::new();
            tb.extend(impl_ident);
            if let Some(tpbs) = self.eat_type_param_bounds() {
                tb.stream(tpbs);
                Some(tb.end())
            } else {
                // self.backtrack(tb.end());
                None
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_trait_object_type(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        if let Some(dyn_ident) = self.eat_ident("dyn") {
            tb.extend(dyn_ident);
        }
        if let Some(tpbs) = self.eat_type_param_bounds() {
            tb.stream(tpbs);
            Some(tb.end())
        } else {
            // self.backtrack(tb.end());
            None
        }
    }

    #[inline(never)]
    pub fn eat_type(&mut self) -> Option<TokenStream> {
        self.eat_type_no_bounds()
            .or_else(|| self.eat_impl_trait_type())
            .or_else(|| self.eat_trait_object_type())
    }

    #[inline(never)]
    pub fn eat_group(&mut self, delim: Delimiter) -> Option<TokenTree> {
        if let Some(TokenTree::Group(group)) = self.peek() {
            if group.delimiter() == delim {
                let ret = Some(TokenTree::Group(group.clone()));
                self.advance();
                ret
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_any_group(&mut self) -> Option<TokenTree> {
        if let Some(TokenTree::Group(group)) = self.peek() {
            let ret = Some(TokenTree::Group(group.clone()));
            self.advance();
            return ret;
        }
        return None;
    }

    #[inline(never)]
    pub fn eat_visibility(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        if let Some(pub_ident) = self.eat_ident("pub") {
            tb.extend(pub_ident);
            if let Some(tt) = self.eat_group(Delimiter::Bracket) {
                tb.extend(tt);
            }
            Some(tb.end())
        } else {
            None
        }
    }
}

/*
    pub fn eat_triple_dot(&mut self) -> Option<TokenStream> {
        if let Some(c1) = self.eat_punct('.') {
            let mut tb = TokenBuilder::new();
            tb.extend(c1);
            if let Some(c2) = self.eat_punct('.') {
                tb.extend(c2);
                if let Some(c3) = self.eat_punct('.') {
                    tb.extend(c3);
                    Some(tb.end())
                } else {
                    self.backtrack(tb.end());
                    None
                }
            } else {
                self.backtrack(tb.end());
                None
            }
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn eat_function_qualifiers(&mut self) -> TokenStream {
        let mut tb = TokenBuilder::new();

        if let Some(async_const) = self.eat_ident("const").or_else(|| self.eat_ident("async")) {
            tb.extend(async_const);
        }

        if let Some(unsafe_ident) = self.eat_ident("unsafe") {
            tb.extend(unsafe_ident);
        }

        if let Some(extern_ident) = self.eat_ident("extern") {
            tb.extend(extern_ident);
            if let Some(abi) = self.eat_literal() {
                tb.extend_literal(abi);
            }
        }

        tb.end()
    }

    #[inline(never)]
    pub fn eat_bare_function_type(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();

        if let Some(for_lt) = self.eat_for_lifetimes() {
            tb.stream(for_lt);
        }
        let fq = self.eat_function_qualifiers();
        tb.stream(fq);

        if let Some(fn_ident) = self.eat_ident("fn") {
            tb.extend(fn_ident);

            if let Some(arrow) = self.eat_fn_arrow() {
                tb.stream(arrow);
                if let Some(type_no_bounds) = self.eat_type_no_bounds() {
                    tb.stream(type_no_bounds);
                } else {
                    self.backtrack(tb.end());
                    return None
                }
            }

            Some(tb.end())
        } else {
            self.backtrack(tb.end());
            return None
        }
    }


    #[inline(never)]
    pub fn eat_maybe_named_function_parameters_variadic(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        while let Some(mnp) = self.eat_maybe_named_param() {
            let comma = self.eat_punct(',')?;
            tb.stream(mnp);
            tb.extend(comma);
        }
        let mnp = self.eat_maybe_named_param()?;
        let comma = self.eat_punct(',')?;

        tb.stream(mnp);
        tb.extend(comma);

        while let Some(attr) = self.eat_outer_attribute() {
            tb.stream(attr);
        }

        let triple_dots = self.eat_triple_dot()?;
        tb.stream(triple_dots);

        Some(tb.end())
    }

    #[inline(never)]
    pub fn eat_maybe_named_param(&mut self) -> Option<TokenStream> {
        let mut tb = TokenBuilder::new();
        while let Some(attr) = self.eat_outer_attribute() {
            tb.stream(attr);
        }
        if let Some(ident_or_anon) = self.eat_any_ident().or_else(|| self.eat_punct('_')) {
            let colon = self.eat_punct(':')?;
            tb.extend(ident_or_anon);
            tb.extend(colon);
        }
        let ty = self.eat_type()?;
        tb.stream(ty);

        Some(tb.end())
    }

    #[inline(never)]
    pub fn eat_maybe_named_function_parameters(&mut self) -> Option<TokenStream> {
        let mnp1 = self.eat_maybe_named_param()?;
        let mut tb = TokenBuilder::new();
        tb.stream(mnp1);
        while let Some(comma) = self.eat_punct(',') {
            let mnp_i = self.eat_maybe_named_param()?;
            tb.extend(comma);
            tb.stream(mnp_i);
        }
        if let Some(comma) = self.eat_punct(',') {
            tb.extend(comma);
        }

        Some(tb.end())
    }

    #[inline(never)]
    pub fn eat_function_parameters_maybe_named_variadic(&mut self) -> Option<TokenStream> {
        self.eat_maybe_named_function_parameters().or_else(|| self.eat_maybe_named_function_parameters_variadic())
    }
*/