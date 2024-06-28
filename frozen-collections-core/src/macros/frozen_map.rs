use std::cmp::PartialEq;
use std::fmt::Display;
use std::hash::RandomState;
use std::str::FromStr;

use bitvec::macros::internal::funty::Fundamental;
use num_traits::PrimInt;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Expr, LitInt, LitStr, Token, Type};

use crate::analyzers::int_key_analyzer::{analyze_int_keys, IntKeyAnalysisResult};
use crate::analyzers::slice_key_analyzer::{analyze_slice_keys, SliceKeyAnalysisResult};

struct Entry(Expr, Expr);

struct Map {
    ty: Type,
    entries: Vec<Entry>,
}

impl ToTokens for Entry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let key = self.0.clone();
        let value = self.1.clone();

        tokens.extend(quote!(#key, #value));
    }
}

impl Parse for Map {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut entries = Vec::<Entry>::new();

        let ty = input.parse::<Type>()?;
        input.parse::<Token![,]>()?;

        while !input.is_empty() {
            let key = input.parse::<Expr>()?;
            input.parse::<Token![:]>()?;
            let value = input.parse::<Expr>()?;

            entries.push(Entry(key, value));

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { ty, entries })
    }
}

#[derive(PartialEq)]
enum KeyVariety {
    Common,
    Integer,
    String,
}

#[doc(hidden)]
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn frozen_map_macro(args: TokenStream) -> TokenStream {
    // proc_marco2 version of "parse_macro_input!(input as ParsedMap)"
    let input = match parse2::<Map>(args) {
        Ok(syntax_tree) => syntax_tree,
        Err(error) => return error.to_compile_error(),
    };

    let mut kv_pairs = input.entries;

    if kv_pairs.len() < 3 {
        return quote!({
            let m = ::frozen_collections::specialized_maps::ScanningMap::from_vec(vec![
            #(
                (#kv_pairs),
            )*
            ]);
            m
        });
    }

    let mut ty = input.ty;
    let type_name = format!("{}", ty.to_token_stream());

    let mut variety = KeyVariety::Integer;
    let mut int_analysis = IntKeyAnalysisResult::Normal;
    let mut slice_analysis = SliceKeyAnalysisResult::Normal;

    // TODO: fix the unwrap usage
    match type_name.as_str() {
        "u8" => int_analysis = process_int_keys::<u8>(&kv_pairs).unwrap(),
        "i8" => int_analysis = process_int_keys::<i8>(&kv_pairs).unwrap(),
        "u16" => int_analysis = process_int_keys::<u16>(&kv_pairs).unwrap(),
        "i16" => int_analysis = process_int_keys::<i16>(&kv_pairs).unwrap(),
        "u32" => int_analysis = process_int_keys::<u32>(&kv_pairs).unwrap(),
        "i32" => int_analysis = process_int_keys::<i32>(&kv_pairs).unwrap(),
        "u64" => int_analysis = process_int_keys::<u64>(&kv_pairs).unwrap(),
        "i64" => int_analysis = process_int_keys::<i64>(&kv_pairs).unwrap(),
        "u128" => int_analysis = process_int_keys::<u128>(&kv_pairs).unwrap(),
        "i128" => int_analysis = process_int_keys::<i128>(&kv_pairs).unwrap(),

        "& str" => {
            variety = KeyVariety::String;
            slice_analysis =
                process_string_keys(kv_pairs.iter().map(|x| x.0.to_token_stream())).unwrap();

            let mut copy = Vec::with_capacity(kv_pairs.len());
            for kv in kv_pairs {
                let original = kv.0.to_token_stream();
                let modified = quote!(String::from(#original));
                copy.push(Entry(parse2::<Expr>(modified).unwrap(), kv.1));
            }

            kv_pairs = copy;
            ty = parse2::<Type>(quote!(String)).unwrap();
        }

        _ => variety = KeyVariety::Common,
    }

    let map_type = match variety {
        KeyVariety::Integer => {
            if int_analysis == IntKeyAnalysisResult::Range {
                format_ident!("{}", "IntegerRangeMap")
            } else {
                format_ident!("{}", "IntegerMap")
            }
        }

        KeyVariety::String => match slice_analysis {
            SliceKeyAnalysisResult::Normal => format_ident!("{}", "CommonMap"),
            SliceKeyAnalysisResult::Length => format_ident!("{}", "LengthMap"),

            SliceKeyAnalysisResult::LeftHandSubslice {
                subslice_index: _,
                subslice_len: _,
            } => format_ident!("{}", "LeftSliceMap"),

            SliceKeyAnalysisResult::RightHandSubslice {
                subslice_index: _,
                subslice_len: _,
            } => format_ident!("{}", "RightSliceMap"),
        },

        KeyVariety::Common => format_ident!("{}", "CommonMap"),
    };

    let payload_size = format_ident!(
        "{}",
        if kv_pairs.len() <= u8::MAX.as_usize() {
            "u8"
        } else if kv_pairs.len() <= u16::MAX.as_usize() {
            "u16"
        } else {
            "usize"
        }
    );

    match slice_analysis {
        SliceKeyAnalysisResult::LeftHandSubslice {
            subslice_index,
            subslice_len,
        } => quote!(
        {
            let m: ::frozen_collections::specialized_maps::#map_type<#ty, _, #payload_size, ::std::hash::RandomState> = ::frozen_collections::specialized_maps::#map_type::from_vec(vec![
            #(
                (#kv_pairs),
            )*
            ], #subslice_index..#subslice_index + #subslice_len);
            m
        }),

        SliceKeyAnalysisResult::RightHandSubslice {
            subslice_index,
            subslice_len,
        } => quote!(
        {
            let m: ::frozen_collections::specialized_maps::#map_type<#ty, _, #payload_size, ::std::hash::RandomState> = ::frozen_collections::specialized_maps::#map_type::from_vec(vec![
            #(
                (#kv_pairs),
            )*
            ], #subslice_index..#subslice_index + #subslice_len);
            m
        }),

        _ => quote!(
        {
            let m: ::frozen_collections::specialized_maps::#map_type<#ty, _, #payload_size> = ::frozen_collections::specialized_maps::#map_type::from_vec(vec![
            #(
                (#kv_pairs),
            )*
            ]);
            m
        }),
    }
}

fn process_int_keys<K>(kv_pairs: &[Entry]) -> syn::Result<IntKeyAnalysisResult>
where
    K: PrimInt + FromStr,
    K::Err: Display,
{
    let keys = kv_pairs.iter().map(|x| x.0.to_token_stream());
    let mut parsed = Vec::new();
    for key in keys {
        let li = parse2::<LitInt>(key)?;
        let v = li.base10_parse::<K>()?;
        parsed.push(v);
    }

    Ok(analyze_int_keys(parsed.into_iter()))
}

fn process_string_keys<I>(keys: I) -> syn::Result<SliceKeyAnalysisResult>
where
    I: Iterator<Item = TokenStream>,
{
    let mut parsed = Vec::new();
    for key in keys {
        let ls = parse2::<LitStr>(key)?;
        parsed.push(ls.value());
    }

    let bh = RandomState::new();
    Ok(analyze_slice_keys(parsed.iter().map(String::as_bytes), &bh))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use proc_macro2::TokenStream;

    use crate::macros::frozen_map::frozen_map_macro;

    #[test]
    fn basic() {
        let ts = TokenStream::from_str(
            "
            &str,
            \"first_key\": (1, \"first_value\"),
            \"second_key\": (2, \"second_value\"),
            \"third_key\": (3, \"third_value\"),
            \"fourth_key\": (4, \"fourth_value\"),
            \"fifth_key\": (5, \"fifth_value\"),
        ",
        )
        .unwrap();

        let ts2 = frozen_map_macro(ts);

        println!("{ts2}");
    }
}
