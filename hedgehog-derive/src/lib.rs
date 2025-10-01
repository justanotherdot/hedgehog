//! Derive macros for Hedgehog property-based testing.
//!
//! This crate provides procedural macros to automatically derive
//! generators and other utilities for custom types.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

/// Derive macro for automatically generating `Gen<T>` implementations.
///
/// This macro generates a generator for custom types by recursively
/// generating values for each field using their respective generators.
///
/// # Example
///
/// ```rust,ignore
/// use hedgehog::*;
/// use hedgehog_derive::Generate;
///
/// #[derive(Generate, Debug, Clone, PartialEq)]
/// struct User {
///     name: String,
///     age: u32,
///     email: String,
/// }
///
/// // Now you can use User::generate() automatically
/// let user_gen = User::generate();
/// ```
#[proc_macro_derive(Generate)]
pub fn derive_generate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Generate the implementation for the Generate trait.
fn generate_impl(input: &DeriveInput) -> Result<TokenStream2, syn::Error> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let generator_impl = match &input.data {
        Data::Struct(data) => generate_struct_impl(data)?,
        Data::Enum(data) => generate_enum_impl(data)?,
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Generate derive macro does not support unions",
            ));
        }
    };

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Generate a generator for this type.
            pub fn generate() -> hedgehog::Gen<Self> {
                #generator_impl
            }
        }
    };

    Ok(expanded)
}

/// Generate implementation for structs.
fn generate_struct_impl(data: &syn::DataStruct) -> Result<TokenStream2, syn::Error> {
    match &data.fields {
        Fields::Named(fields) => {
            let field_data: Vec<_> = fields
                .named
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_var =
                        syn::Ident::new(&format!("field_{i}"), proc_macro2::Span::call_site());
                    let generator = generate_field_generator(&field.ty)?;
                    Ok((field_name.clone(), field_var, generator))
                })
                .collect::<Result<Vec<_>, syn::Error>>()?;

            let field_bindings = field_data.iter().map(|(_, var, gen)| {
                quote! {
                    let (field_seed, next_seed) = current_seed.split();
                    current_seed = next_seed;
                    let #var = (#gen).generate(size, field_seed).outcome().clone();
                }
            });

            let field_assignments = field_data.iter().map(|(name, var, _)| {
                quote! {
                    #name: #var
                }
            });

            Ok(quote! {
                hedgehog::Gen::new(|size, seed| {
                    use hedgehog::{Tree, Seed};

                    let mut current_seed = seed;
                    #(#field_bindings)*

                    let value = Self {
                        #(#field_assignments),*
                    };

                    Tree::singleton(value)
                })
            })
        }
        Fields::Unnamed(fields) => {
            let field_data: Vec<_> = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let field_var =
                        syn::Ident::new(&format!("field_{i}"), proc_macro2::Span::call_site());
                    let generator = generate_field_generator(&field.ty)?;
                    Ok((field_var, generator))
                })
                .collect::<Result<Vec<_>, syn::Error>>()?;

            let field_bindings = field_data.iter().map(|(var, gen)| {
                quote! {
                    let (field_seed, next_seed) = current_seed.split();
                    current_seed = next_seed;
                    let #var = (#gen).generate(size, field_seed).outcome().clone();
                }
            });

            let field_vars = field_data.iter().map(|(var, _)| var);

            Ok(quote! {
                hedgehog::Gen::new(|size, seed| {
                    use hedgehog::{Tree, Seed};

                    let mut current_seed = seed;
                    #(#field_bindings)*

                    let value = Self(#(#field_vars),*);

                    Tree::singleton(value)
                })
            })
        }
        Fields::Unit => Ok(quote! {
            hedgehog::Gen::constant(Self)
        }),
    }
}

/// Generate implementation for enums.
fn generate_enum_impl(data: &syn::DataEnum) -> Result<TokenStream2, syn::Error> {
    let variants: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;

            match &variant.fields {
                Fields::Named(fields) => {
                    let field_data: Vec<_> = fields
                        .named
                        .iter()
                        .enumerate()
                        .map(|(i, field)| {
                            let field_name = field.ident.as_ref().unwrap();
                            let field_var = syn::Ident::new(
                                &format!("field_{i}"),
                                proc_macro2::Span::call_site(),
                            );
                            let generator = generate_field_generator(&field.ty)?;
                            Ok((field_name.clone(), field_var, generator))
                        })
                        .collect::<Result<Vec<_>, syn::Error>>()?;

                    let field_bindings = field_data.iter().map(|(_, var, gen)| {
                        quote! {
                            let (field_seed, next_seed) = current_seed.split();
                            current_seed = next_seed;
                            let #var = (#gen).generate(size, field_seed).outcome().clone();
                        }
                    });

                    let field_assignments = field_data.iter().map(|(name, var, _)| {
                        quote! {
                            #name: #var
                        }
                    });

                    Ok(quote! {
                        hedgehog::Gen::new(|size, seed| {
                            use hedgehog::{Tree, Seed};

                            let mut current_seed = seed;
                            #(#field_bindings)*

                            let value = Self::#variant_name {
                                #(#field_assignments),*
                            };

                            Tree::singleton(value)
                        })
                    })
                }
                Fields::Unnamed(fields) => {
                    let field_data: Vec<_> = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, field)| {
                            let field_var = syn::Ident::new(
                                &format!("field_{i}"),
                                proc_macro2::Span::call_site(),
                            );
                            let generator = generate_field_generator(&field.ty)?;
                            Ok((field_var, generator))
                        })
                        .collect::<Result<Vec<_>, syn::Error>>()?;

                    let field_bindings = field_data.iter().map(|(var, gen)| {
                        quote! {
                            let (field_seed, next_seed) = current_seed.split();
                            current_seed = next_seed;
                            let #var = (#gen).generate(size, field_seed).outcome().clone();
                        }
                    });

                    let field_vars = field_data.iter().map(|(var, _)| var);

                    Ok(quote! {
                        hedgehog::Gen::new(|size, seed| {
                            use hedgehog::{Tree, Seed};

                            let mut current_seed = seed;
                            #(#field_bindings)*

                            let value = Self::#variant_name(#(#field_vars),*);

                            Tree::singleton(value)
                        })
                    })
                }
                Fields::Unit => Ok(quote! {
                    hedgehog::Gen::constant(Self::#variant_name)
                }),
            }
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    Ok(quote! {
        hedgehog::Gen::one_of(vec![
            #(#variants),*
        ])
    })
}

/// Generate a field generator based on the type.
fn generate_field_generator(field_type: &Type) -> Result<TokenStream2, syn::Error> {
    match field_type {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "String" => Ok(quote! { hedgehog::Gen::<String>::ascii_alpha() }),
                    "i32" => Ok(
                        quote! { hedgehog::Gen::<i32>::from_range(hedgehog::Range::new(0, 100)) },
                    ),
                    "u32" => Ok(
                        quote! { hedgehog::Gen::<u32>::from_range(hedgehog::Range::new(0, 100)) },
                    ),
                    "i64" => Ok(
                        quote! { hedgehog::Gen::<i64>::from_range(hedgehog::Range::new(0, 100)) },
                    ),
                    "f64" => Ok(
                        quote! { hedgehog::Gen::<f64>::from_range(hedgehog::Range::new(0.0, 100.0)) },
                    ),
                    "bool" => Ok(quote! { hedgehog::Gen::bool() }),
                    "char" => Ok(quote! { hedgehog::Gen::<char>::ascii_alpha() }),
                    "u8" => Ok(
                        quote! { hedgehog::Gen::<u32>::from_range(hedgehog::Range::new(0, 255)).map(|x| x as u8) },
                    ),
                    "u16" => Ok(
                        quote! { hedgehog::Gen::<u32>::from_range(hedgehog::Range::new(0, 65535)).map(|x| x as u16) },
                    ),
                    "u64" => Ok(
                        quote! { hedgehog::Gen::<u32>::from_range(hedgehog::Range::new(0, u32::MAX)).map(|x| x as u64) },
                    ),
                    "i8" => Ok(
                        quote! { hedgehog::Gen::<i32>::from_range(hedgehog::Range::new(-128, 127)).map(|x| x as i8) },
                    ),
                    "i16" => Ok(
                        quote! { hedgehog::Gen::<i32>::from_range(hedgehog::Range::new(-32768, 32767)).map(|x| x as i16) },
                    ),
                    "f32" => Ok(
                        quote! { hedgehog::Gen::<f64>::from_range(hedgehog::Range::new(0.0, 100.0)).map(|x| x as f32) },
                    ),
                    _ => {
                        // For custom types, assume they have a generate() method
                        Ok(quote! { #field_type::generate() })
                    }
                }
            } else {
                Err(syn::Error::new_spanned(
                    field_type,
                    "Unable to generate generator for this type",
                ))
            }
        }
        _ => {
            // For other types, try to call generate() on them
            Ok(quote! { #field_type::generate() })
        }
    }
}
