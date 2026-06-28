use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Field, GenericArgument, PathArguments, Result, Type, TypePath, Visibility};

enum DerivedFieldKind {
    Leaf {
        builder: Box<Expr>,
        value_type: Box<Type>,
        required: bool,
    },
    Group,
}

pub(crate) struct DerivedField {
    ident: Ident,
    visibility: Visibility,
    rust_type: Type,
    kind: DerivedFieldKind,
}

impl DerivedField {
    pub(crate) fn parse(field: Field) -> Result<Self> {
        let field_attr = parse_field_attr(&field)?;
        let ident = field.ident.ok_or_else(|| {
            syn::Error::new_spanned(&field.ty, "`FieldDefs` requires named struct fields")
        })?;
        let kind = match field_attr {
            FieldAttr::Leaf(builder) => {
                let (value_type, required) = declared_value_type(field.ty.clone())?;
                DerivedFieldKind::Leaf {
                    builder: Box::new(builder),
                    value_type: Box::new(value_type),
                    required,
                }
            }
            FieldAttr::Group => DerivedFieldKind::Group,
        };
        Ok(Self {
            ident,
            visibility: field.vis,
            rust_type: field.ty,
            kind,
        })
    }

    pub(crate) fn builder_field(&self) -> TokenStream2 {
        let ident = &self.ident;
        let visibility = &self.visibility;
        match &self.kind {
            DerivedFieldKind::Leaf { value_type, .. } => {
                let value_type = &**value_type;
                quote!(#visibility #ident: ::docnav_typed_fields::FieldDefBuilder<#value_type>)
            }
            DerivedFieldKind::Group => {
                let rust_type = &self.rust_type;
                quote!(#visibility #ident: <#rust_type as ::docnav_typed_fields::FieldDefs>::Builder)
            }
        }
    }

    pub(crate) fn builder_init(&self) -> TokenStream2 {
        let ident = &self.ident;
        match &self.kind {
            DerivedFieldKind::Leaf {
                builder,
                value_type,
                ..
            } => {
                let builder = &**builder;
                let value_type = &**value_type;
                quote! {
                    #ident: {
                        let field: ::docnav_typed_fields::FieldDefBuilder<#value_type> = #builder;
                        field
                    }
                }
            }
            DerivedFieldKind::Group => {
                let rust_type = &self.rust_type;
                quote!(#ident: <#rust_type as ::docnav_typed_fields::FieldDefs>::field_defs_builder())
            }
        }
    }

    pub(crate) fn register_step(&self) -> TokenStream2 {
        let ident = &self.ident;
        let path_segment = field_path_segment(ident);
        match &self.kind {
            DerivedFieldKind::Leaf { required, .. } => {
                quote! {
                    let mut field_path = declaration_path.clone();
                    field_path.push(::std::string::String::from(#path_segment));
                    let builder = builder.__field_with_declaration_path(
                        field_path,
                        self.#ident,
                        ::docnav_typed_fields::__private::ExpectedFieldShape {
                            required: #required,
                            nullable: !#required,
                        },
                    );
                }
            }
            DerivedFieldKind::Group => {
                let rust_type = &self.rust_type;
                quote! {
                    let mut field_path = declaration_path.clone();
                    field_path.push(::std::string::String::from(#path_segment));
                    let builder = <<#rust_type as ::docnav_typed_fields::FieldDefs>::Builder as ::docnav_typed_fields::FieldDefsBuilder>::__register_field_defs(
                        self.#ident,
                        builder,
                        field_path,
                    );
                }
            }
        }
    }

    pub(crate) fn values_init(&self) -> TokenStream2 {
        let ident = &self.ident;
        match &self.kind {
            DerivedFieldKind::Leaf {
                value_type,
                required,
                ..
            } => {
                let value_type = &**value_type;
                if *required {
                    quote! {
                        #ident: {
                            let value = values.__typed_required_slot::<#value_type>(next_slot);
                            next_slot += 1;
                            value
                        }
                    }
                } else {
                    quote! {
                        #ident: {
                            let value = values.__typed_optional_slot::<#value_type>(next_slot);
                            next_slot += 1;
                            value
                        }
                    }
                }
            }
            DerivedFieldKind::Group => {
                let rust_type = &self.rust_type;
                quote! {
                    #ident: {
                        let value = <#rust_type as ::docnav_typed_fields::FieldDefs>::__values_from_slots(
                            values,
                            next_slot,
                        );
                        next_slot += <#rust_type as ::docnav_typed_fields::FieldDefs>::__FIELD_COUNT;
                        value
                    }
                }
            }
        }
    }

    pub(crate) fn default_values_init(&self) -> TokenStream2 {
        let ident = &self.ident;
        match &self.kind {
            DerivedFieldKind::Leaf { value_type, .. } => {
                let value_type = &**value_type;
                quote! {
                    #ident: {
                        let value = values.__typed_optional_slot::<#value_type>(next_slot);
                        next_slot += 1;
                        value
                    }
                }
            }
            DerivedFieldKind::Group => {
                let rust_type = &self.rust_type;
                quote! {
                    #ident: {
                        let value = <#rust_type as ::docnav_typed_fields::FieldDefs>::__default_values_from_slots(
                            values,
                            next_slot,
                        );
                        next_slot += <#rust_type as ::docnav_typed_fields::FieldDefs>::__FIELD_COUNT;
                        value
                    }
                }
            }
        }
    }

    pub(crate) fn default_value_field(&self) -> TokenStream2 {
        let ident = &self.ident;
        let visibility = &self.visibility;
        match &self.kind {
            DerivedFieldKind::Leaf { value_type, .. } => {
                let value_type = &**value_type;
                quote!(#visibility #ident: ::std::option::Option<#value_type>)
            }
            DerivedFieldKind::Group => {
                let rust_type = &self.rust_type;
                quote!(#visibility #ident: <#rust_type as ::docnav_typed_fields::FieldDefs>::DefaultValues)
            }
        }
    }

    pub(crate) fn field_count_expr(&self) -> TokenStream2 {
        match &self.kind {
            DerivedFieldKind::Leaf { .. } => quote!(1),
            DerivedFieldKind::Group => {
                let rust_type = &self.rust_type;
                quote!(<#rust_type as ::docnav_typed_fields::FieldDefs>::__FIELD_COUNT)
            }
        }
    }
}

enum FieldAttr {
    Leaf(Expr),
    Group,
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(syn::Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            if ident == "group" && fork.is_empty() {
                input.parse::<Ident>()?;
                return Ok(Self::Group);
            }
        }
        Ok(Self::Leaf(input.parse()?))
    }
}

fn parse_field_attr(field: &Field) -> Result<FieldAttr> {
    let attrs = field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("field"))
        .collect::<Vec<_>>();
    match attrs.as_slice() {
        [attr] => attr.parse_args(),
        [] => Err(syn::Error::new_spanned(
            field,
            "`FieldDefs` fields must have a `#[field(...)]` attribute",
        )),
        _ => Err(syn::Error::new_spanned(
            field,
            "`FieldDefs` fields must not have multiple `#[field(...)]` attributes",
        )),
    }
}

pub(crate) fn ensure_has_fields(struct_name: &Ident, fields: &[DerivedField]) -> Result<()> {
    if fields.is_empty() {
        return Err(syn::Error::new_spanned(
            struct_name,
            "`FieldDefs` requires at least one field",
        ));
    }
    Ok(())
}

fn declared_value_type(declared_type: Type) -> Result<(Type, bool)> {
    if let Some(inner) = option_inner_type(&declared_type)? {
        return Ok((inner, false));
    }
    Ok((declared_type, true))
}

fn option_inner_type(declared_type: &Type) -> Result<Option<Type>> {
    let Type::Path(TypePath { qself: None, path }) = declared_type else {
        return Ok(None);
    };
    let Some(segment) = path.segments.last() else {
        return Ok(None);
    };
    if segment.ident != "Option" {
        return Ok(None);
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            segment,
            "`Option` field declarations must include one value type",
        ));
    };
    if arguments.args.len() != 1 {
        return Err(syn::Error::new_spanned(
            arguments,
            "`Option` field declarations must include exactly one value type",
        ));
    }
    let Some(GenericArgument::Type(inner)) = arguments.args.first() else {
        return Err(syn::Error::new_spanned(
            arguments,
            "`Option` field declarations must use a type argument",
        ));
    };
    Ok(Some(inner.clone()))
}

fn field_path_segment(ident: &Ident) -> String {
    let value = ident.to_string();
    value.strip_prefix("r#").unwrap_or(&value).to_string()
}
