use quote::{format_ident, ToTokens};
use syn::{parse::Parse, punctuated::Punctuated, Token};

#[derive(Default)]
struct Method {
    vis: Option<syn::Visibility>,
    name: Option<syn::Ident>,
    metas: Vec<syn::Meta>,
}

impl Parse for Method {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut r = Method::default();
        let metas = Punctuated::<syn::Meta, Token![,]>::parse_terminated(input)?;
        for meta in metas {
            match meta {
                syn::Meta::NameValue(pair) => {
                    let key = path_to_ident(&pair.path);
                    match key.as_str() {
                        "vis" => {
                            let value_tokens = trim_string_lit(&pair.value)?;
                            let vis = syn::parse2::<syn::Visibility>(value_tokens)?;
                            assert!(r.vis.is_none(), "duplicate key: {}", key);
                            r.vis = Some(vis);
                        }
                        "method" => {
                            let value_tokens = trim_string_lit(&pair.value)?;
                            let name = syn::parse2::<syn::Ident>(value_tokens)?;
                            assert!(r.name.is_none(), "duplicate key: {}", key);
                            r.name = Some(name);
                        }
                        _ => {
                            r.metas.push(syn::Meta::from(pair));
                        }
                    }
                }
                meta => {
                    r.metas.push(meta);
                }
            }
        }
        Ok(r)
    }
}

#[derive(Default)]
struct Rw {
    read: Method,
    write: Method,
}

impl Parse for Rw {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut rw = Rw::default();
        let metas = Punctuated::<syn::Meta, Token![,]>::parse_terminated(input)?;
        for meta in metas {
            match meta {
                syn::Meta::NameValue(pair) => {
                    let key = path_to_ident(&pair.path);
                    match key.as_str() {
                        "write_vis" => {
                            let value_tokens = trim_string_lit(&pair.value)?;
                            let vis = syn::parse2::<syn::Visibility>(value_tokens)?;
                            assert!(rw.write.vis.is_none(), "duplicate key: {}", key);
                            rw.write.vis = Some(vis);
                        }
                        "write" => {
                            let value_tokens = trim_string_lit(&pair.value)?;
                            let name = syn::parse2::<syn::Ident>(value_tokens)?;
                            assert!(rw.write.name.is_none(), "duplicate key: {}", key);
                            rw.write.name = Some(name);
                        }
                        "read_vis" => {
                            let value_tokens = trim_string_lit(&pair.value)?;
                            let vis = syn::parse2::<syn::Visibility>(value_tokens)?;
                            assert!(rw.read.vis.is_none(), "duplicate key: {}", key);
                            rw.read.vis = Some(vis);
                        }
                        "read" => {
                            let value_tokens = trim_string_lit(&pair.value)?;
                            let name = syn::parse2::<syn::Ident>(value_tokens)?;
                            assert!(rw.read.name.is_none(), "duplicate key: {}", key);
                            rw.read.name = Some(name);
                        }
                        _ => panic!("unexpected key: {}", key),
                    }
                }
                syn::Meta::List(list) => {
                    let list_name = path_to_ident(&list.path);
                    match list_name.as_str() {
                        "read" => {
                            rw.read = syn::parse2::<Method>(list.tokens)?;
                        }
                        "write" => {
                            rw.write = syn::parse2::<Method>(list.tokens)?;
                        }
                        _ => panic!("unexpected key: {}", list_name),
                    }
                }
                _ => panic!("unexpected value"),
            }
        }
        Ok(rw)
    }
}

type Mutex = Method;

#[derive(Default)]
enum Lock {
    Mutex(Mutex),
    Rw(Rw),
    #[default]
    Arc,
}

#[derive(Default)]
struct Config {
    struct_vis: Option<syn::Visibility>,
    lock: Option<Lock>,
    name: Option<syn::Ident>,
    metas: Vec<syn::Meta>,
}

fn path_to_ident(path: &syn::Path) -> String {
    assert!(path.leading_colon.is_none(), "expect ident");
    assert_eq!(path.segments.len(), 1, "expect ident");
    path.segments.first().unwrap().ident.to_string()
}

impl Parse for Config {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut config = Config::default();

        let metas = Punctuated::<syn::Meta, Token![,]>::parse_terminated(input)?;
        for meta in metas {
            match meta {
                syn::Meta::Path(path) => {
                    let ident = path_to_ident(&path);
                    println!("ident: {}", ident);
                    match ident.as_str() {
                        "mutex" => {
                            assert!(config.lock.is_none(), "duplicate key: {}", ident);
                            config.lock = Some(Lock::Mutex(Mutex::default()));
                        }
                        "rwlock" => {
                            assert!(config.lock.is_none(), "duplicate key: {}", ident);
                            config.lock = Some(Lock::Rw(Rw::default()));
                        }
                        _ => {
                            config.metas.push(syn::Meta::from(path));
                        }
                    }
                }
                syn::Meta::NameValue(pair) => {
                    let key = path_to_ident(&pair.path);
                    let value_tokens = trim_string_lit(&pair.value)?;

                    match key.as_str() {
                        "vis" => {
                            let vis = syn::parse2::<syn::Visibility>(value_tokens)?;
                            assert!(config.struct_vis.is_none(), "duplicate key: {}", key);
                            config.struct_vis = Some(vis);
                        }
                        "lock" => {
                            let lock = match value_tokens.to_string().as_str() {
                                "mutex" => Lock::Mutex(Mutex::default()),
                                "rwlock" => Lock::Rw(Rw::default()),
                                "none" => Lock::Arc,
                                _ => panic!(
                                    "unexpected value: {}; expect one of [mutex, rwloock, none]",
                                    value_tokens
                                ),
                            };
                            assert!(config.lock.is_none(), "duplicate key: {}", key);
                            config.lock = Some(lock);
                        }
                        "rename" => {
                            let name = syn::parse2::<syn::Ident>(value_tokens)?;
                            assert!(config.name.is_none(), "duplicate key: {}", key);
                            config.name = Some(name);
                        }
                        _ => {
                            config.metas.push(syn::Meta::from(pair));
                        }
                    }
                }
                syn::Meta::List(list) => {
                    let list_name = path_to_ident(&list.path);
                    match list_name.as_str() {
                        "mutex" => {
                            assert!(config.lock.is_none(), "duplicate key: {}", list_name);
                            config.lock = Some(Lock::Mutex(syn::parse2::<Mutex>(list.tokens)?));
                        }
                        "rwlock" => {
                            assert!(config.lock.is_none(), "duplicate key: {}", list_name);
                            config.lock = Some(Lock::Rw(syn::parse2::<Rw>(list.tokens)?));
                        }
                        _ => {
                            config.metas.push(syn::Meta::from(list));
                        }
                    }
                }
            }
        }
        Ok(config)
    }
}

fn trim_string_lit(expr: &syn::Expr) -> Result<proc_macro2::TokenStream, proc_macro2::LexError> {
    expr.into_token_stream()
        .to_string()
        .trim_start_matches("\"")
        .trim_end_matches("\"")
        .parse()
}

#[proc_macro_attribute]
pub fn arc_wrapper(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let derive_input = item.clone();
    let raw = proc_macro2::TokenStream::from(item);
    let derive_input = syn::parse_macro_input!(derive_input as syn::DeriveInput);
    let Config {
        struct_vis: new_struct_vis,
        lock: lock_kind,
        name: new_struct_name,
        metas: new_struct_metas,
    } = syn::parse_macro_input!(attr as Config);
    let lock_kind = lock_kind.unwrap_or_default();

    let raw_struct_name = derive_input.ident.clone();
    let new_struct_vis = new_struct_vis.unwrap_or(derive_input.vis);
    let new_struct_name = new_struct_name.unwrap_or(format_ident!("Arc{}", derive_input.ident));

    let new_struct_generic = derive_input.generics.clone();
    let raw_struct_generic_without_where = {
        let mut raw_struct_generic = derive_input.generics.clone();
        raw_struct_generic.where_clause = None;
        raw_struct_generic
    };
    let raw_struct_type = quote::quote! {
        #raw_struct_name #raw_struct_generic_without_where
    };

    let new_struct = {
        let inner = match &lock_kind {
            Lock::Mutex(_) => quote::quote! {
                ::std::sync::Arc<::std::sync::Mutex<#raw_struct_type>>
            },
            Lock::Rw(_) => quote::quote! {
                ::std::sync::Arc<::std::sync::RwLock<#raw_struct_type>>
            },
            Lock::Arc => quote::quote! {
                ::std::sync::Arc<#raw_struct_type>
            },
        };

        quote::quote! {
            #(#[#new_struct_metas])*
            #new_struct_vis struct #new_struct_name #new_struct_generic {
                inner: #inner
            }
        }
    };

    let from_impl = quote::quote! {
        impl #raw_struct_generic_without_where From< #raw_struct_type > for #new_struct_name #new_struct_generic {
            fn from(inner: #raw_struct_type ) -> Self {
                Self {
                    inner: ::std::sync::Arc::new(inner.into())
                }
            }
        }
    };

    let new_struct_methods = match lock_kind {
        Lock::Mutex(mutex) => {
            let method = mutex.name.unwrap_or_else(|| format_ident!("lock_guard"));
            let vis = mutex.vis.unwrap_or(new_struct_vis);
            let metas = &mutex.metas;
            quote::quote! {
                #(#[#metas])*
                #vis fn #method(&self) -> ::std::sync::MutexGuard<'_, #raw_struct_type > {
                    ::std::result::Result::unwrap(::std::sync::Mutex::lock(&self.inner))
                }
            }
        }
        Lock::Rw(rw_lock) => {
            let read_method = rw_lock
                .read
                .name
                .unwrap_or_else(|| format_ident!("read_guard"));
            let read_vis = rw_lock.read.vis.unwrap_or(new_struct_vis.clone());
            let read_metas = &rw_lock.read.metas;
            let write_method = rw_lock
                .write
                .name
                .unwrap_or_else(|| format_ident!("write_guard"));
            let write_vis = rw_lock.write.vis.unwrap_or(new_struct_vis);
            let write_metas = &rw_lock.write.metas;
            quote::quote! {
                #(#[#read_metas])*
                #read_vis fn #read_method(&self) -> ::std::sync::RwLockReadGuard<'_, #raw_struct_type > {
                    ::std::result::Result::unwrap(::std::sync::RwLock::read(&self.inner))
                }

                #(#[#write_metas])*
                #write_vis fn #write_method(&self) -> ::std::sync::RwLockWriteGuard<'_, #raw_struct_type > {
                    ::std::result::Result::unwrap(::std::sync::RwLock::write(&self.inner))
                }
            }
        }
        Lock::Arc => quote::quote! {},
    };

    let new_struct_impl = quote::quote! {
        impl #raw_struct_generic_without_where #new_struct_name #new_struct_generic {
            #new_struct_methods
        }
    };

    quote::quote! {
        #raw

        #new_struct

        #from_impl

        #new_struct_impl

    }
    .into()
}
