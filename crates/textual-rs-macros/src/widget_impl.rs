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

/// Collected info for a single #[on(Type)] annotation — method extracted from impl Widget block.
struct OnAnnotation {
    type_path: syn::Path,
    method_name: Ident,
    /// The method itself (moved to inherent impl)
    method: ImplItemFn,
}

/// Collected info for a single #[keybinding("key", "action")] annotation — method extracted.
struct KeybindingAnnotation {
    key_code: TokenStream,
    modifiers: TokenStream,
    action: String,
    method_name: Ident,
    /// The method itself (moved to inherent impl)
    method: ImplItemFn,
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
fn try_parse_on_attr(attr: &syn::Attribute) -> Option<syn::Path> {
    if !attr.path().is_ident("on") {
        return None;
    }
    attr.parse_args::<syn::Path>().ok()
}

/// Attempt to extract #[keybinding("key_combo", "action")] — returns (combo, action, span).
fn try_parse_keybinding_attr(attr: &syn::Attribute) -> Option<(String, String, Span)> {
    if !attr.path().is_ident("keybinding") {
        return None;
    }
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
/// 1. Scans methods for `#[on(T)]` and `#[keybinding(...)]` annotations.
/// 2. Removes annotated handler methods from the `impl Widget` block.
/// 3. Moves those handler methods into a separate inherent `impl` block.
/// 4. Injects delegation methods that weren't manually provided into the Widget impl.
/// 5. Generates `on_event`, `key_bindings`, `on_action` methods in the Widget impl.
pub fn widget_impl_transform(mut input: ItemImpl) -> TokenStream {
    let mut on_annotations: Vec<OnAnnotation> = vec![];
    let mut keybinding_annotations: Vec<KeybindingAnnotation> = vec![];
    let mut errors: Vec<syn::Error> = vec![];

    // Scan methods for annotations and extract them
    let mut remaining_items: Vec<ImplItem> = vec![];

    for item in input.items.drain(..) {
        if let ImplItem::Fn(mut func) = item {
            // Check if this function has #[on(T)] or #[keybinding(...)] attributes
            let mut on_type: Option<syn::Path> = None;
            let mut keybinding_info: Option<(String, String, Span)> = None;
            let mut other_attrs: Vec<syn::Attribute> = vec![];

            for attr in func.attrs.drain(..) {
                if let Some(tp) = try_parse_on_attr(&attr) {
                    on_type = Some(tp);
                    // Strip this attr (don't push to other_attrs)
                } else if let Some(kb) = try_parse_keybinding_attr(&attr) {
                    keybinding_info = Some(kb);
                    // Strip this attr
                } else {
                    other_attrs.push(attr);
                }
            }
            func.attrs = other_attrs;

            if let Some(type_path) = on_type {
                // This method is an #[on(T)] handler — extract it to inherent impl
                on_annotations.push(OnAnnotation {
                    type_path,
                    method_name: func.sig.ident.clone(),
                    method: func,
                });
                // Don't add to remaining_items — it goes to the inherent impl
            } else if let Some((combo, action, span)) = keybinding_info {
                match parse_key_combo(&combo, span) {
                    Ok((key_code, modifiers)) => {
                        keybinding_annotations.push(KeybindingAnnotation {
                            key_code,
                            modifiers,
                            action,
                            method_name: func.sig.ident.clone(),
                            method: func,
                        });
                    }
                    Err(e) => {
                        errors.push(e);
                        remaining_items.push(ImplItem::Fn(func));
                    }
                }
                // Don't add to remaining_items — it goes to the inherent impl
            } else {
                // Normal Widget trait method — keep in the impl Widget block
                remaining_items.push(ImplItem::Fn(func));
            }
        } else {
            remaining_items.push(item);
        }
    }

    input.items = remaining_items;

    if !errors.is_empty() {
        let err_tokens: TokenStream = errors.into_iter().map(|e| e.to_compile_error()).collect();
        return err_tokens;
    }

    // Determine what methods are already manually provided in the Widget trait impl
    let has_widget_type_name = method_exists(&input.items, "widget_type_name");
    let has_can_focus = method_exists(&input.items, "can_focus");
    let has_on_mount = method_exists(&input.items, "on_mount");
    let has_on_unmount = method_exists(&input.items, "on_unmount");
    let has_on_event = method_exists(&input.items, "on_event");
    let has_key_bindings = method_exists(&input.items, "key_bindings");
    let has_on_action = method_exists(&input.items, "on_action");

    // Build injection list for the Widget trait impl
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

    // Generate on_event from #[on(T)] annotations
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

    // Generate on_action from #[keybinding] annotations
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

    // Inject generated methods into the Widget trait impl block
    for method_ts in injected {
        let parsed_fn: Result<ImplItemFn, _> = parse2(method_ts.clone());
        match parsed_fn {
            Ok(f) => input.items.push(ImplItem::Fn(f)),
            Err(e) => {
                return e.to_compile_error();
            }
        }
    }

    // Build the inherent impl block for handler methods (from #[on] and #[keybinding])
    let self_ty = &input.self_ty;

    let on_handler_methods: Vec<&ImplItemFn> =
        on_annotations.iter().map(|a| &a.method).collect();
    let kb_handler_methods: Vec<&ImplItemFn> =
        keybinding_annotations.iter().map(|a| &a.method).collect();

    let inherent_impl = if !on_handler_methods.is_empty() || !kb_handler_methods.is_empty() {
        quote! {
            impl #self_ty {
                #(#on_handler_methods)*
                #(#kb_handler_methods)*
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #input
        #inherent_impl
    }
}
