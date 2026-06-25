use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Result};

mod model;

use model::{ensure_has_fields, DerivedField};

pub(crate) fn expand(input: DeriveInput) -> Result<TokenStream2> {
    let struct_name = input.ident;
    let visibility = input.vis;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            input.generics,
            "`FieldDefs` does not support generic parameter structs",
        ));
    }

    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            struct_name,
            "`FieldDefs` can only be derived for structs",
        ));
    };
    let Fields::Named(fields) = data.fields else {
        return Err(syn::Error::new_spanned(
            struct_name,
            "`FieldDefs` requires named struct fields",
        ));
    };

    let fields = fields
        .named
        .into_iter()
        .map(DerivedField::parse)
        .collect::<Result<Vec<_>>>()?;
    ensure_has_fields(&struct_name, &fields)?;

    let definition_set_name = format_ident!("{struct_name}FieldDefs");
    let builder_name = format_ident!("{struct_name}FieldDefsBuilder");
    let default_values_name = format_ident!("{struct_name}DefaultValues");

    let builder_fields = fields.iter().map(DerivedField::builder_field);
    let builder_inits = fields.iter().map(DerivedField::builder_init);
    let register_steps = fields.iter().map(DerivedField::register_step);
    let values_inits = fields.iter().map(DerivedField::values_init);
    let default_values_inits = fields.iter().map(DerivedField::default_values_init);
    let default_value_fields = fields.iter().map(DerivedField::default_value_field);
    let field_count_terms = fields.iter().map(DerivedField::field_count_expr);

    Ok(quote! {
        #[derive(Clone, Debug)]
        #visibility struct #builder_name {
            #(#builder_fields,)*
        }

        impl #builder_name {
            pub fn build(self) -> ::std::result::Result<#definition_set_name, ::docnav_typed_fields::FieldDefSetBuildError> {
                <Self as ::docnav_typed_fields::FieldDefsBuilder>::build(self)
            }
        }

        impl ::docnav_typed_fields::FieldDefsBuilder for #builder_name {
            type DefinitionSet = #definition_set_name;

            fn build(self) -> ::std::result::Result<Self::DefinitionSet, ::docnav_typed_fields::FieldDefSetBuildError> {
                let editable_builder = self.clone();
                let builder = ::docnav_typed_fields::FieldDefSet::builder();
                let builder = <Self as ::docnav_typed_fields::FieldDefsBuilder>::__register_field_defs(
                    self,
                    builder,
                    ::std::vec::Vec::new(),
                );
                let field_def_set = ::std::sync::Arc::new(builder.build()?);
                Ok(#definition_set_name {
                    __field_def_set: field_def_set,
                    __builder: editable_builder,
                })
            }

            #[doc(hidden)]
            fn __register_field_defs(
                self,
                builder: ::docnav_typed_fields::__private::FieldDefSetBuilder,
                declaration_path: ::std::vec::Vec<::std::string::String>,
            ) -> ::docnav_typed_fields::__private::FieldDefSetBuilder {
                #(#register_steps)*
                builder
            }
        }

        #[derive(Clone, Debug)]
        #visibility struct #definition_set_name {
            __field_def_set: ::std::sync::Arc<::docnav_typed_fields::FieldDefSet>,
            __builder: #builder_name,
        }

        impl #definition_set_name {
            pub fn to_builder(&self) -> #builder_name {
                self.__builder.clone()
            }

            pub fn extract_without_default(
                &self,
                root: &::docnav_typed_fields::__private::JsonValue,
            ) -> ::std::result::Result<#struct_name, ::docnav_typed_fields::FieldValidationErrors> {
                let values = self.__field_def_set.__extract_values_without_default(root)?;
                Ok(<#struct_name as ::docnav_typed_fields::FieldDefs>::__values_from_slots(&values, 0))
            }

            pub fn extract_with_static_defaults(
                &self,
                root: &::docnav_typed_fields::__private::JsonValue,
            ) -> ::std::result::Result<#struct_name, ::docnav_typed_fields::FieldValidationErrors> {
                let values = self.__field_def_set.__extract_values_with_static_defaults(root)?;
                Ok(<#struct_name as ::docnav_typed_fields::FieldDefs>::__values_from_slots(&values, 0))
            }

            pub fn default_values(&self) -> #default_values_name {
                let values = self.__field_def_set.__static_default_values();
                <#struct_name as ::docnav_typed_fields::FieldDefs>::__default_values_from_slots(&values, 0)
            }

            pub fn validate_without_default(
                &self,
                root: &::docnav_typed_fields::__private::JsonValue,
            ) -> ::std::result::Result<(), ::docnav_typed_fields::FieldValidationErrors> {
                self.__field_def_set.__validate_values_without_default(root)
            }

            pub fn validate_with_static_defaults(
                &self,
                root: &::docnav_typed_fields::__private::JsonValue,
            ) -> ::std::result::Result<(), ::docnav_typed_fields::FieldValidationErrors> {
                self.__field_def_set.__validate_values_with_static_defaults(root)
            }

            pub fn schema_metadata(&self) -> ::std::vec::Vec<::docnav_typed_fields::SchemaMetadataView> {
                self.__field_def_set.schema_metadata()
            }

            pub fn value_kinds(&self) -> ::std::collections::BTreeMap<::std::string::String, ::docnav_typed_fields::ValueKind> {
                self.__field_def_set.value_kinds()
            }
        }

        #[derive(Clone, Debug)]
        #visibility struct #default_values_name {
            #(#default_value_fields,)*
        }

        impl ::docnav_typed_fields::FieldDefs for #struct_name {
            type DefinitionSet = #definition_set_name;
            type Builder = #builder_name;
            type DefaultValues = #default_values_name;

            #[doc(hidden)]
            const __FIELD_COUNT: usize = 0 #(+ #field_count_terms)*;

            fn field_defs_builder() -> Self::Builder {
                #builder_name {
                    #(#builder_inits,)*
                }
            }

            #[doc(hidden)]
            fn __values_from_slots(
                values: &::docnav_typed_fields::__private::FieldValues,
                offset: usize,
            ) -> Self {
                let mut next_slot = offset;
                let value = Self {
                    #(#values_inits,)*
                };
                let _ = next_slot;
                value
            }

            #[doc(hidden)]
            fn __default_values_from_slots(
                values: &::docnav_typed_fields::__private::FieldValues,
                offset: usize,
            ) -> Self::DefaultValues {
                let mut next_slot = offset;
                let value = #default_values_name {
                    #(#default_values_inits,)*
                };
                let _ = next_slot;
                value
            }
        }
    })
}
