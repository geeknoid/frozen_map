use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;

use frozen_collections_core::macros::frozen_map_macro;

#[proc_macro]
#[proc_macro_error]
pub fn frozen_map(item: TokenStream) -> TokenStream {
    frozen_map_macro(item.into()).into()
}
