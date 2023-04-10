use heck::ToShoutySnakeCase;
use syn::{self, ext::IdentExt, Data, DataStruct, Fields, Ident};
use synstructure::quote;

synstructure::decl_derive!([EnvArgs] => env_args_macro);

fn env_args_macro(
    s: synstructure::Structure,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let ast = s.ast();

    match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => {
            let fields = fields
                .named
                .iter()
                .map(|field| {
                    let ident = field
                        .ident
                        .clone()
                        .expect("names field should have identifier");
                    let env = ident.unraw().to_string();
                    let env = env.to_shouty_snake_case();

                    (env, ident)
                })
                .collect::<Vec<(String, Ident)>>();

            let mut body = quote!();

            for field in fields {
                let env = field.0;
                let name = field.1;
                body.extend(quote!(
                    map.insert(#env.to_owned(), self.#name.clone());
                ));
            }

            Ok(s.gen_impl(quote!(
                use std::collections::BTreeMap;
                gen impl EnvArgs for @Self {
                    fn get_env_pairs(&self) -> BTreeMap<String, String> {
                        let mut map = BTreeMap::new();

                        #body

                        map
                    }
                }
            )))
        }
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use synstructure::test_derive;

    use crate::env_args_macro;
    use env_arg::EnvArgs;

    #[test]
    fn derives_correctly() {
        test_derive! {
            env_args_macro {
                struct Options {
                    pub discord_token: String,
                }
            }
            expands to {
                #[allow(non_upper_case_globals)]
                const _DERIVE_EnvArgs_FOR_Options: () = {
                    use std::collections::BTreeMap;
                    impl EnvArgs for Options {
                        fn get_env_pairs(&self) -> BTreeMap<String, String> {
                            let mut map  = BTreeMap::new();

                            map.insert("DISCORD_TOKEN".to_owned(), self.discord_token.clone());

                            map
                        }
                    }
                };
            }
        }
    }
}
