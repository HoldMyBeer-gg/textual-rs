use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse2, punctuated::Punctuated, Ident, ImplItem, ImplItemFn, ItemImpl, LitStr, Token,
};

/// Parse a key combo string like "ctrl+s", "enter", "shift+tab", "a", "f1" into
/// (KeyCode tokens, KeyModifiers tokens).
fn parse_key_combo(combo: &str, span: Span) -> Result<(TokenStream, TokenStream), syn::Error> {
    let combo_lower = combo.to_ascii_lowercase();
    let parts: Vec<&str> = combo_lower.split('+').collect();

    let key_part;
    let modifiers: Vec<&str>;

    if parts.len() > 1 {
        // Last part is the key, rest are modifiers
        key_part = *parts.last().unwrap();
        modifiers = parts[..parts.len() - 1].to_vec();
    } else {
        key_part = parts[0];
        modifiers = vec![];
    }

    let key_code = match key_part {
        "enter" => quote! { ::crossterm::event::KeyCode::Enter },
        "tab" => quote! { ::crossterm::event::KeyCode::Tab },
        "up" => quote! { ::crossterm::event::KeyCode::Up },
        "down" => quote! { ::crossterm::event::KeyCode::Down },
        "left" => quote! { ::crossterm::event::KeyCode::Left },
        "right" => quote! { ::crossterm::event::KeyCode::Right },
        "esc" | "escape" => quote! { ::crossterm::event::KeyCode::Esc },
        "backspace" => quote! { ::crossterm::event::KeyCode::Backspace },
        "delete" | "del" => quote! { ::crossterm::event::KeyCode::Delete },
        "home" => quote! { ::crossterm::event::KeyCode::Home },
        "end" => quote! { ::crossterm::event::KeyCode::End },
        "pageup" => quote! { ::crossterm::event::KeyCode::PageUp },
        "pagedown" => quote! { ::crossterm::event::KeyCode::PageDown },
        "insert" | "ins" => quote! { ::crossterm::event::KeyCode::Insert },
        "space" => quote! { ::crossterm::event::KeyCode::Char(' ') },
        "f1" => quote! { ::crossterm::event::KeyCode::F(1) },
        "f2" => quote! { ::crossterm::event::KeyCode::F(2) },
        "f3" => quote! { ::crossterm::event::KeyCode::F(3) },
        "f4" => quote! { ::crossterm::event::KeyCode::F(4) },
        "f5" => quote! { ::crossterm::event::KeyCode::F(5) },
        "f6" => quote! { ::crossterm::event::KeyCode::F(6) },
        "f7" => quote! { ::crossterm::event::KeyCode::F(7) },
        "f8" => quote! { ::crossterm::event::KeyCode::F(8) },
        "f9" => quote! { ::crossterm::event::KeyCode::F(9) },
        "f10" => quote! { ::crossterm::event::KeyCode::F(10) },
        "f11" => quote! { ::crossterm::event::KeyCode::F(11) },
        "f12" => quote! { ::crossterm::event::KeyCode::F(12) },
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap();
            quote! { ::crossterm::event::KeyCode::Char(#ch) }
        }
        _ => {
            return Err(syn::Error::new(span, format!("Unknown key: {:?}", key_part)));
        }
    };

    // Build modifier flags
    let mut mod_tokens: Vec<TokenStream> = vec![];
    for m in &modifiers {
        match *m {
            "ctrl" | "control" => {
                mod_tokens.push(quote! { ::crossterm::event::KeyModifiers::CONTROL });
            }
            "shift" => {
                mod_tokens.push(quote! { ::crossterm::event::KeyModifiers::SHIFT });
            }
            "alt" => {
                mod_tokens.push(quote! { ::crossterm::event::KeyModifiers::ALT });
            }
            other => {
                return Err(syn::Error::new(span, format!("Unknown modifier: {:?}", other)));
            }
        }
    }

    let modifiers_ts = if mod_tokens.is_empty() {
        quote! { ::crossterm::event::KeyModifiers::NONE }
    } else {
        let mut combined = mod_tokens.remove(0);
        for m in mod_tokens {
            combined = quote! { #combined | #m };
        }
        combined
    };

    Ok((key_code, modifiers_ts))
}

/// Collected info for a single #[on(Type)] annotation.
struct OnAnnotation {
    type_path: syn::Path,
    method_name: Ident,
}

/// Collected info for a single #[keybinding("key", "action")] annotation.
struct KeybindingAnnotation {
    key_code: TokenStream,
    modifiers: TokenStream,
    action: String,
    method_name: Ident,
}

/// Check if a method with the given name exists in the impl items.
fn method_exists(items: &[ImplItem], name: &str) -> bool {
    items.iter().any(|item| {
        if let ImplItem::Fn(f) = item {
            f.sig.ident == name
        } else {
            false
        }
    })
}

/// Attempt to extract #[on(TypePath)] — returns the type path if this is an `on` attr.
/// Uses syn 2.x API.
fn try_parse_on_attr(attr: &syn::Attribute) -> Option<syn::Path> {
    if !attr.path().is_ident("on") {
        return None;
    }
    // Parse the tokens inside the parens as a syn::Path
    attr.parse_args::<syn::Path>().ok()
}

/// Attempt to extract #[keybinding("key_combo", "action")] — returns (combo, action, span).
/// Uses syn 2.x API.
fn try_parse_keybinding_attr(attr: &syn::Attribute) -> Option<(String, String, Span)> {
    if !attr.path().is_ident("keybinding") {
        return None;
    }
    // Parse as two comma-separated string literals
    let result: Result<Punctuated<LitStr, Token![,]>, _> =
        attr.parse_args_with(Punctuated::parse_terminated);
    if let Ok(args) = result {
        let args: Vec<LitStr> = args.into_iter().collect();
        if args.len() == 2 {
            let span = attr.pound_token.span;
            return Some((args[0].value(), args[1].value(), span));
        }
    }
    None
}

/// Main transform function for the `#[widget_impl]` attribute macro.
///
/// Processes the `impl Widget for Struct` block:
/// 1. Scans methods for `#[on(T)]` and `#[keybinding(...)]` annotations, strips them.
/// 2. Injects delegation methods that weren't manually provided.
/// 3. Generates `on_event`, `key_bindings`, `on_action` if annotations found.
pub fn widget_impl_transform(mut input: ItemImpl) -> TokenStream {
    let mut on_annotations: Vec<OnAnnotation> = vec![];
    let mut keybinding_annotations: Vec<KeybindingAnnotation> = vec![];
    let mut errors: Vec<syn::Error> = vec![];

    // Collect and strip annotations from methods
    for item in &mut input.items {
        if let ImplItem::Fn(func) = item {
            let mut retained_attrs = vec![];
            for attr in func.attrs.drain(..) {
                if let Some(type_path) = try_parse_on_attr(&attr) {
                    on_annotations.push(OnAnnotation {
                        type_path,
                        method_name: func.sig.ident.clone(),
                    });
                    // Strip the attr
                    continue;
                } else if let Some((combo, action, span)) = try_parse_keybinding_attr(&attr) {
                    match parse_key_combo(&combo, span) {
                        Ok((key_code, modifiers)) => {
                            keybinding_annotations.push(KeybindingAnnotation {
                                key_code,
                                modifiers,
                                action,
                                method_name: func.sig.ident.clone(),
                            });
                        }
                        Err(e) => errors.push(e),
                    }
                    // Strip the attr
                    continue;
                }
                retained_attrs.push(attr);
            }
            func.attrs = retained_attrs;
        }
    }

    // If there were parse errors, emit them
    if !errors.is_empty() {
        let err_tokens: TokenStream = errors.into_iter().map(|e| e.to_compile_error()).collect();
        return err_tokens;
    }

    // Determine what methods are already manually provided
    let has_widget_type_name = method_exists(&input.items, "widget_type_name");
    let has_can_focus = method_exists(&input.items, "can_focus");
    let has_on_mount = method_exists(&input.items, "on_mount");
    let has_on_unmount = method_exists(&input.items, "on_unmount");
    let has_on_event = method_exists(&input.items, "on_event");
    let has_key_bindings = method_exists(&input.items, "key_bindings");
    let has_on_action = method_exists(&input.items, "on_action");

    // Build list of injected method token streams
    let mut injected: Vec<TokenStream> = vec![];

    if !has_widget_type_name {
        injected.push(quote! {
            fn widget_type_name(&self) -> &'static str {
                Self::__widget_type_name()
            }
        });
    }

    if !has_can_focus {
        injected.push(quote! {
            fn can_focus(&self) -> bool {
                Self::__can_focus()
            }
        });
    }

    if !has_on_mount {
        injected.push(quote! {
            fn on_mount(&self, id: ::textual_rs::widget::WidgetId) {
                self.__on_mount(id);
            }
        });
    }

    if !has_on_unmount {
        injected.push(quote! {
            fn on_unmount(&self, id: ::textual_rs::widget::WidgetId) {
                self.__on_unmount(id);
            }
        });
    }

    // Generate on_event dispatch from #[on(T)] annotations
    if !has_on_event && !on_annotations.is_empty() {
        let dispatch_arms: Vec<TokenStream> = on_annotations
            .iter()
            .map(|ann| {
                let type_path = &ann.type_path;
                let method = &ann.method_name;
                quote! {
                    if let Some(msg) = event.downcast_ref::<#type_path>() {
                        self.#method(msg, ctx);
                        return ::textual_rs::widget::EventPropagation::Stop;
                    }
                }
            })
            .collect();

        injected.push(quote! {
            fn on_event(
                &self,
                event: &dyn ::std::any::Any,
                ctx: &::textual_rs::widget::context::AppContext,
            ) -> ::textual_rs::widget::EventPropagation {
                #(#dispatch_arms)*
                ::textual_rs::widget::EventPropagation::Continue
            }
        });
    }

    // Generate key_bindings from #[keybinding] annotations
    if !has_key_bindings && !keybinding_annotations.is_empty() {
        let binding_entries: Vec<TokenStream> = keybinding_annotations
            .iter()
            .map(|ann| {
                let key_code = &ann.key_code;
                let modifiers = &ann.modifiers;
                let action = &ann.action;
                quote! {
                    ::textual_rs::event::KeyBinding {
                        key: #key_code,
                        modifiers: #modifiers,
                        action: #action,
                        description: #action,
                        show: true,
                    }
                }
            })
            .collect();

        injected.push(quote! {
            fn key_bindings(&self) -> &[::textual_rs::event::KeyBinding] {
                static BINDINGS: ::std::sync::OnceLock<
                    ::std::vec::Vec<::textual_rs::event::KeyBinding>
                > = ::std::sync::OnceLock::new();
                BINDINGS.get_or_init(|| vec![#(#binding_entries),*])
            }
        });
    }

    // Generate on_action dispatch from #[keybinding] annotations
    if !has_on_action && !keybinding_annotations.is_empty() {
        let action_arms: Vec<TokenStream> = keybinding_annotations
            .iter()
            .map(|ann| {
                let action = &ann.action;
                let method = &ann.method_name;
                quote! {
                    #action => self.#method(ctx),
                }
            })
            .collect();

        injected.push(quote! {
            fn on_action(&self, action: &str, ctx: &::textual_rs::widget::context::AppContext) {
                match action {
                    #(#action_arms)*
                    _ => {}
                }
            }
        });
    }

    // Parse and inject all generated methods into the impl block's items
    for method_ts in injected {
        let parsed_fn: Result<ImplItemFn, _> = parse2(method_ts.clone());
        match parsed_fn {
            Ok(f) => input.items.push(ImplItem::Fn(f)),
            Err(e) => {
                return e.to_compile_error();
            }
        }
    }

    quote! { #input }
}
