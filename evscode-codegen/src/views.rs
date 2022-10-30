use crate::util::{
	invoke_list::InvocationList, option_literal, params::{self, ParamMap}
};
use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
	parse::{Parse, ParseStream}, parse_macro_input, ItemFn, LitStr
};

pub static VIEW_INVOKELIST: InvocationList = InvocationList::new("View");

pub fn generate(params: TokenStream, item: TokenStream) -> TokenStream {
	let params: Params = parse_macro_input!(params);
	let item: ItemFn = parse_macro_input!(item);
	let local_name = &item.sig.ident;
	let name = LitStr::new(&params.name, Span::call_site().into());
    let addto = LitStr::new(&params.addto, Span::call_site().into());
	let machinery = VIEW_INVOKELIST.invoke(quote! {
		evscode::meta::View {
			id: evscode::meta::Identifier {
				module_path: module_path!(),
				local_name: stringify!(#local_name),
			},
			name: #name,
			addto: #addto,
		}
	});
	TokenStream::from(quote! {
		#item
		#machinery
	})
}

#[derive(Debug)]
pub struct Params {
	pub name: String,
    pub addto: String,
}
impl Parse for Params {
	fn parse(input: ParseStream) -> params::R<Params> {
		let mut params: ParamMap = input.parse()?;
		let r = Params { name: params.get("name")? , addto: params.get("addto")?};
		params.finish()?;
		Ok(r)
	}
}
