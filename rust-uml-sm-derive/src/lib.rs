extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use quote::format_ident;
use syn;

/// Usage:
/// [derive(StateMachine)]
/// struct Foo{...};
/// This will add an implementation of the trait StateMachine to our
/// struct, enabling us to call Foo::new_statemachine()

#[proc_macro_derive(StateMachine)]
pub fn statemachine_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_statemachine_macro(&ast)
}

fn impl_statemachine_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let sm = format_ident!("{}StateMachine", name);
    let sm_trait = format_ident!("{}StateMachineTrait", name);
    //let sm_new = format_ident!("{}::new", sm);

    let gen = quote! {
        impl #name {
            fn new_statemachine() -> #sm {
                println!("Constructing, StateMachine {}!", stringify!(#sm));
                let state = #name::new();
                let state_def = StateMachineDef::new(stringify!(#sm));
                #sm {
                    state: state,
                    define: state_def,
                }
            }
        }
        struct #sm {
            state: #name,
            define: StateMachineDef,
        }
        /*
        impl #sm {
            fn new() {
                println!("Creating {}!", stringify!(#sm));
            }
        }
        */
    };
    gen.into()
}

