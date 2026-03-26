use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error};

/// Implementation of `#[derive(Widget)]`.
///
/// Generates an inherent `impl #name` block with private `__` helper methods:
/// - `fn __widget_type_name() -> &'static str` — returns the struct name as a string literal
/// - `fn __can_focus() -> bool` — returns `true` if `#[focusable]` is present on the struct
/// - `fn __on_mount(&self, id: ::textual_rs::widget::WidgetId)` — sets `self.own_id`
/// - `fn __on_unmount(&self, _id: ::textual_rs::widget::WidgetId)` — clears `self.own_id`
///
/// IMPORTANT: Does NOT generate `impl Widget for #name`. The user's `#[widget_impl]`-annotated
/// `impl Widget for MyStruct { ... }` block is the SOLE Widget trait impl.
pub fn derive_widget_impl(input: DeriveInput) -> TokenStream {
    let name = &input.ident;

    // Only support named structs — error on enum or union
    match &input.data {
        Data::Struct(_) => {}
        _ => {
            return Error::new(name.span(), "Widget derive only supports named structs")
                .to_compile_error();
        }
    }

    let name_str = name.to_string();

    // Check for #[focusable] attribute on the struct
    let can_focus_val = input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("focusable"));

    let can_focus_val = if can_focus_val {
        quote! { true }
    } else {
        quote! { false }
    };

    // Generate the inherent impl block with __ helper methods
    quote! {
        impl #name {
            fn __widget_type_name() -> &'static str {
                #name_str
            }

            fn __can_focus() -> bool {
                #can_focus_val
            }

            fn __on_mount(&self, id: ::textual_rs::widget::WidgetId) {
                self.own_id.set(Some(id));
            }

            fn __on_unmount(&self, _id: ::textual_rs::widget::WidgetId) {
                self.own_id.set(None);
            }
        }
    }
}
