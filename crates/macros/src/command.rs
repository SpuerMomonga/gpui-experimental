use quote::quote;
use syn::{Data, DeriveInput, Expr, Fields, Meta, Type, spanned::Spanned};

pub(crate) fn expand_args(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            Fields::Unnamed(fields) => {
                return Err(syn::Error::new(
                    fields.span(),
                    "Args can only be derived for structs with named fields",
                ));
            }
            Fields::Unit => {
                return Ok(quote! {
                    impl ::command::Args for #name {
                        fn schema() -> ::command::Schema {
                            ::command::Schema::default()
                        }

                        fn decode(input: ::command::Input) -> ::command::anyhow::Result<Self> {
                            match input {
                                ::command::Input::External(value) => {
                                    ::command::serde_json::from_value(value)
                                        .map_err(::command::anyhow::Error::from)
                                }
                                ::command::Input::Internal(_) => {
                                    ::command::anyhow::bail!(
                                        "argumented command requires external input"
                                    )
                                }
                            }
                        }
                    }
                });
            }
        },
        _other => {
            return Err(syn::Error::new(
                name.span(),
                "Args can only be derived for structs",
            ));
        }
    };

    let mut schema_fields = Vec::new();
    let mut default_inserts = Vec::new();
    for field in &fields {
        let ident = field.ident.as_ref().expect("named field");
        let field_name = ident.to_string();
        let mut schema_name = field_name.clone();
        let mut description = doc_comment(&field.attrs);
        let mut default = None;
        let mut kind = None;

        for attr in &field.attrs {
            if !attr.path().is_ident("arg") {
                continue;
            }
            let metas = attr.parse_args_with(
                syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
            )?;
            for meta in metas {
                let Meta::NameValue(name_value) = meta else {
                    return Err(syn::Error::new(
                        meta.span(),
                        "expected `key = value` in #[arg]",
                    ));
                };
                let Some(key) = name_value.path.get_ident() else {
                    return Err(syn::Error::new(
                        name_value.path.span(),
                        "invalid #[arg] key",
                    ));
                };
                match key.to_string().as_str() {
                    "name" => schema_name = string_literal(&name_value.value, "name")?,
                    "description" => {
                        description = Some(string_literal(&name_value.value, "description")?)
                    }
                    "default" => default = Some(name_value.value),
                    "kind" => kind = Some(string_literal(&name_value.value, "kind")?),
                    unknown => {
                        return Err(syn::Error::new(
                            key.span(),
                            format!("unknown #[arg] option `{unknown}`"),
                        ));
                    }
                }
            }
        }

        let ty = &field.ty;
        let required = default.is_none() && !is_option(ty);
        let kind_expr = match kind.as_deref() {
            Some("string") => quote!(::command::Kind::String),
            Some("integer") => quote!(::command::Kind::Integer),
            Some("number") => quote!(::command::Kind::Number),
            Some("boolean") => quote!(::command::Kind::Boolean),
            Some("json") => quote!(::command::Kind::Json),
            Some(other) => {
                return Err(syn::Error::new(
                    field.span(),
                    format!("unknown argument kind `{other}`"),
                ));
            }
            None => quote!(<#ty as ::command::FieldType>::kind()),
        };
        let description_expr =
            description.map_or_else(|| quote!(None), |value| quote!(Some(#value.to_owned())));
        let default_expr = default.as_ref().map_or_else(
            || quote!(None),
            |value| quote!(::command::serde_json::to_value(#value).ok()),
        );
        if let Some(value) = default.clone() {
            default_inserts.push(quote! {
                if !map.contains_key(#field_name) {
                    map.insert(
                        #field_name.to_owned(),
                        ::command::serde_json::to_value(#value)
                            .map_err(::command::anyhow::Error::from)?,
                    );
                }
            });
        }
        schema_fields.push(quote! {
            ::command::Field {
                name: #schema_name.to_owned(),
                description: #description_expr,
                kind: #kind_expr,
                required: #required,
                default: #default_expr,
            }
        });
    }

    Ok(quote! {
        impl ::command::Args for #name {
            fn schema() -> ::command::Schema {
                ::command::Schema {
                    fields: vec![#(#schema_fields),*],
                }
            }

            fn decode(input: ::command::Input) -> ::command::anyhow::Result<Self> {
                match input {
                    ::command::Input::External(value) => {
                        let mut value = value;
                        if let ::command::serde_json::Value::Object(map) = &mut value {
                            #(#default_inserts)*
                        }
                        ::command::serde_json::from_value(value)
                            .map_err(::command::anyhow::Error::from)
                    }
                    ::command::Input::Internal(_) => {
                        ::command::anyhow::bail!("argumented command requires external input")
                    }
                }
            }
        }
    })
}

fn string_literal(expr: &Expr, option: &str) -> syn::Result<String> {
    match expr {
        Expr::Lit(expr) => match &expr.lit {
            syn::Lit::Str(value) => Ok(value.value()),
            _ => Err(syn::Error::new(
                expr.span(),
                format!("#[arg({option})] must be a string"),
            )),
        },
        _ => Err(syn::Error::new(
            expr.span(),
            format!("#[arg({option})] must be a string"),
        )),
    }
}

fn doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let Meta::NameValue(meta) = &attr.meta else {
            continue;
        };
        if let Expr::Lit(expr) = &meta.value {
            if let syn::Lit::Str(value) = &expr.lit {
                lines.push(value.value().trim().to_owned());
            }
        }
    }
    while lines.first().is_some_and(String::is_empty) {
        lines.remove(0);
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

fn is_option(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Option")
}
