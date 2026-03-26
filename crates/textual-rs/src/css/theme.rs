use std::collections::HashMap;

use super::types::TcssColor;

/// Convert RGB (0-255) to HSL (H: 0-360, S: 0.0-1.0, L: 0.0-1.0).
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    let l = (max + min) / 2.0;

    if delta < 1e-10 {
        return (0.0, 0.0, l);
    }

    let s = if l <= 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let h = if (max - rf).abs() < 1e-10 {
        let mut h = (gf - bf) / delta;
        if h < 0.0 {
            h += 6.0;
        }
        h * 60.0
    } else if (max - gf).abs() < 1e-10 {
        ((bf - rf) / delta + 2.0) * 60.0
    } else {
        ((rf - gf) / delta + 4.0) * 60.0
    };

    (h, s, l)
}

/// Convert HSL (H: 0-360, S: 0.0-1.0, L: 0.0-1.0) back to RGB (0-255).
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s < 1e-10 {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h_norm = h / 360.0;

    let hue_to_rgb = |t: f64| -> f64 {
        let mut t = t;
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };

    let r = (hue_to_rgb(h_norm + 1.0 / 3.0) * 255.0).round() as u8;
    let g = (hue_to_rgb(h_norm) * 255.0).round() as u8;
    let b = (hue_to_rgb(h_norm - 1.0 / 3.0) * 255.0).round() as u8;

    (r, g, b)
}

/// Adjust the luminosity of a color by `delta` (positive = lighten, negative = darken).
/// Only operates on `TcssColor::Rgb`; other variants are returned unchanged.
pub fn lighten_color(color: TcssColor, delta: f64) -> TcssColor {
    match color {
        TcssColor::Rgb(r, g, b) => {
            let (h, s, l) = rgb_to_hsl(r, g, b);
            let new_l = (l + delta).clamp(0.0, 1.0);
            let (nr, ng, nb) = hsl_to_rgb(h, s, new_l);
            TcssColor::Rgb(nr, ng, nb)
        }
        other => other,
    }
}

/// A semantic theme with named color slots and shade generation.
///
/// Colors are stored as `(u8, u8, u8)` RGB tuples. The `resolve` method
/// maps variable names like `"primary"`, `"primary-lighten-2"`, or
/// `"accent-darken-1"` to concrete `TcssColor::Rgb` values.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub primary: (u8, u8, u8),
    pub secondary: (u8, u8, u8),
    pub accent: (u8, u8, u8),
    pub surface: (u8, u8, u8),
    pub panel: (u8, u8, u8),
    pub background: (u8, u8, u8),
    pub foreground: (u8, u8, u8),
    pub success: (u8, u8, u8),
    pub warning: (u8, u8, u8),
    pub error: (u8, u8, u8),
    pub dark: bool,
    pub luminosity_spread: f64,
    /// User-defined variable overrides. Checked before computed shades.
    pub variables: HashMap<String, TcssColor>,
}

impl Theme {
    /// Resolve a theme variable name to a concrete color.
    ///
    /// Supports base names (`"primary"`) and shade variants
    /// (`"primary-lighten-2"`, `"accent-darken-1"`).
    /// Checks `variables` HashMap first for user overrides.
    pub fn resolve(&self, name: &str) -> Option<TcssColor> {
        // Check user overrides first
        if let Some(color) = self.variables.get(name) {
            return Some(*color);
        }

        // Try to parse shade suffix: "base-lighten-N" or "base-darken-N"
        let (base_name, shade_delta) = if let Some(rest) = name.strip_suffix("-lighten-1") {
            (rest, Some(1))
        } else if let Some(rest) = name.strip_suffix("-lighten-2") {
            (rest, Some(2))
        } else if let Some(rest) = name.strip_suffix("-lighten-3") {
            (rest, Some(3))
        } else if let Some(rest) = name.strip_suffix("-darken-1") {
            (rest, Some(-1))
        } else if let Some(rest) = name.strip_suffix("-darken-2") {
            (rest, Some(-2))
        } else if let Some(rest) = name.strip_suffix("-darken-3") {
            (rest, Some(-3))
        } else {
            (name, None)
        };

        // Look up the base color from struct fields
        let base_rgb = match base_name {
            "primary" => Some(self.primary),
            "secondary" => Some(self.secondary),
            "accent" => Some(self.accent),
            "surface" => Some(self.surface),
            "panel" => Some(self.panel),
            "background" => Some(self.background),
            "foreground" => Some(self.foreground),
            "success" => Some(self.success),
            "warning" => Some(self.warning),
            "error" => Some(self.error),
            _ => None,
        }?;

        let base_color = TcssColor::Rgb(base_rgb.0, base_rgb.1, base_rgb.2);

        match shade_delta {
            None => Some(base_color),
            Some(n) => {
                let step = self.luminosity_spread / 2.0;
                let delta = n as f64 * step;
                Some(lighten_color(base_color, delta))
            }
        }
    }
}

/// Blend two RGB colors: result = a * (1 - factor) + b * factor
fn blend_rgb(a: (u8, u8, u8), b: (u8, u8, u8), factor: f64) -> (u8, u8, u8) {
    let r = (a.0 as f64 * (1.0 - factor) + b.0 as f64 * factor).round() as u8;
    let g = (a.1 as f64 * (1.0 - factor) + b.1 as f64 * factor).round() as u8;
    let b_val = (a.2 as f64 * (1.0 - factor) + b.2 as f64 * factor).round() as u8;
    (r, g, b_val)
}

/// Returns the default dark theme matching Python Textual's `textual-dark` palette.
pub fn default_dark_theme() -> Theme {
    let primary = (1, 120, 212);
    let surface = (30, 30, 30);
    let panel = blend_rgb(surface, primary, 0.1);

    Theme {
        name: "textual-dark".to_string(),
        primary,
        secondary: (0, 69, 120),
        accent: (255, 166, 43),
        surface,
        panel,
        background: (18, 18, 18),
        foreground: (224, 224, 224),
        success: (78, 191, 113),
        warning: (255, 166, 43),
        error: (186, 60, 91),
        dark: true,
        luminosity_spread: 0.15,
        variables: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- HSL round-trip tests ---

    #[test]
    fn hsl_round_trip_pure_red() {
        let (h, s, l) = rgb_to_hsl(255, 0, 0);
        assert!((h - 0.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
        let (r, g, b) = hsl_to_rgb(h, s, l);
        assert_eq!((r, g, b), (255, 0, 0));
    }

    #[test]
    fn hsl_round_trip_white() {
        let (h, _s, l) = rgb_to_hsl(255, 255, 255);
        assert!((l - 1.0).abs() < 0.01);
        let (r, g, b) = hsl_to_rgb(h, 0.0, l);
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn hsl_round_trip_black() {
        let (_h, _s, l) = rgb_to_hsl(0, 0, 0);
        assert!((l - 0.0).abs() < 0.01);
        let (r, g, b) = hsl_to_rgb(0.0, 0.0, l);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn hsl_round_trip_primary_blue() {
        // #0178D4 = (1, 120, 212)
        let (h, s, l) = rgb_to_hsl(1, 120, 212);
        let (r, g, b) = hsl_to_rgb(h, s, l);
        assert!((r as i16 - 1).abs() <= 1);
        assert!((g as i16 - 120).abs() <= 1);
        assert!((b as i16 - 212).abs() <= 1);
    }

    // --- Default dark theme tests ---

    #[test]
    fn default_dark_theme_primary() {
        let theme = default_dark_theme();
        assert_eq!(theme.primary, (1, 120, 212));
    }

    #[test]
    fn default_dark_theme_all_colors() {
        let theme = default_dark_theme();
        assert_eq!(theme.name, "textual-dark");
        assert_eq!(theme.primary, (1, 120, 212));
        assert_eq!(theme.secondary, (0, 69, 120));
        assert_eq!(theme.accent, (255, 166, 43));
        assert_eq!(theme.warning, (255, 166, 43));
        assert_eq!(theme.error, (186, 60, 91));
        assert_eq!(theme.success, (78, 191, 113));
        assert_eq!(theme.foreground, (224, 224, 224));
        assert_eq!(theme.background, (18, 18, 18));
        assert_eq!(theme.surface, (30, 30, 30));
        assert!(theme.dark);
        assert!((theme.luminosity_spread - 0.15).abs() < 0.001);
    }

    #[test]
    fn default_dark_theme_panel_blend() {
        let theme = default_dark_theme();
        // panel = surface * 0.9 + primary * 0.1
        // r = 30*0.9 + 1*0.1 = 27.1 -> 27
        // g = 30*0.9 + 120*0.1 = 39.0 -> 39
        // b = 30*0.9 + 212*0.1 = 48.2 -> 48
        assert_eq!(theme.panel, (27, 39, 48));
    }

    // --- Resolve base names ---

    #[test]
    fn resolve_primary_returns_rgb() {
        let theme = default_dark_theme();
        assert_eq!(theme.resolve("primary"), Some(TcssColor::Rgb(1, 120, 212)));
    }

    #[test]
    fn resolve_all_base_names() {
        let theme = default_dark_theme();
        assert_eq!(theme.resolve("secondary"), Some(TcssColor::Rgb(0, 69, 120)));
        assert_eq!(theme.resolve("accent"), Some(TcssColor::Rgb(255, 166, 43)));
        assert_eq!(theme.resolve("surface"), Some(TcssColor::Rgb(30, 30, 30)));
        assert_eq!(theme.resolve("panel"), Some(TcssColor::Rgb(27, 39, 48)));
        assert_eq!(theme.resolve("background"), Some(TcssColor::Rgb(18, 18, 18)));
        assert_eq!(theme.resolve("foreground"), Some(TcssColor::Rgb(224, 224, 224)));
        assert_eq!(theme.resolve("success"), Some(TcssColor::Rgb(78, 191, 113)));
        assert_eq!(theme.resolve("warning"), Some(TcssColor::Rgb(255, 166, 43)));
        assert_eq!(theme.resolve("error"), Some(TcssColor::Rgb(186, 60, 91)));
    }

    #[test]
    fn resolve_unknown_returns_none() {
        let theme = default_dark_theme();
        assert_eq!(theme.resolve("nonexistent"), None);
        assert_eq!(theme.resolve(""), None);
        assert_eq!(theme.resolve("primary-lighten-99"), None);
    }

    // --- Shade generation tests ---

    #[test]
    fn resolve_primary_lighten_1_is_lighter() {
        let theme = default_dark_theme();
        let base = theme.resolve("primary").unwrap();
        let lighter = theme.resolve("primary-lighten-1").unwrap();
        // Lighter means higher luminosity
        let base_l = match base {
            TcssColor::Rgb(r, g, b) => rgb_to_hsl(r, g, b).2,
            _ => panic!("expected Rgb"),
        };
        let lighter_l = match lighter {
            TcssColor::Rgb(r, g, b) => rgb_to_hsl(r, g, b).2,
            _ => panic!("expected Rgb"),
        };
        assert!(lighter_l > base_l, "lighten-1 should have higher L than base");
    }

    #[test]
    fn resolve_primary_darken_1_is_darker() {
        let theme = default_dark_theme();
        let base = theme.resolve("primary").unwrap();
        let darker = theme.resolve("primary-darken-1").unwrap();
        let base_l = match base {
            TcssColor::Rgb(r, g, b) => rgb_to_hsl(r, g, b).2,
            _ => panic!("expected Rgb"),
        };
        let darker_l = match darker {
            TcssColor::Rgb(r, g, b) => rgb_to_hsl(r, g, b).2,
            _ => panic!("expected Rgb"),
        };
        assert!(darker_l < base_l, "darken-1 should have lower L than base");
    }

    #[test]
    fn shades_are_monotonically_ordered() {
        let theme = default_dark_theme();
        let names = [
            "primary-darken-3",
            "primary-darken-2",
            "primary-darken-1",
            "primary",
            "primary-lighten-1",
            "primary-lighten-2",
            "primary-lighten-3",
        ];
        let luminosities: Vec<f64> = names
            .iter()
            .map(|n| {
                let color = theme.resolve(n).unwrap_or_else(|| panic!("failed to resolve {}", n));
                match color {
                    TcssColor::Rgb(r, g, b) => rgb_to_hsl(r, g, b).2,
                    _ => panic!("expected Rgb"),
                }
            })
            .collect();

        for i in 1..luminosities.len() {
            assert!(
                luminosities[i] > luminosities[i - 1],
                "L[{}] ({}) should be > L[{}] ({}), names: {} > {}",
                i, luminosities[i], i - 1, luminosities[i - 1], names[i], names[i - 1]
            );
        }
    }

    #[test]
    fn accent_lighten_2_works() {
        let theme = default_dark_theme();
        let result = theme.resolve("accent-lighten-2");
        assert!(result.is_some(), "accent-lighten-2 should resolve");
        let base_l = match theme.resolve("accent").unwrap() {
            TcssColor::Rgb(r, g, b) => rgb_to_hsl(r, g, b).2,
            _ => panic!("expected Rgb"),
        };
        let shade_l = match result.unwrap() {
            TcssColor::Rgb(r, g, b) => rgb_to_hsl(r, g, b).2,
            _ => panic!("expected Rgb"),
        };
        assert!(shade_l > base_l);
    }

    // --- Variables override ---

    #[test]
    fn variables_override_computed_shades() {
        let mut theme = default_dark_theme();
        let override_color = TcssColor::Rgb(99, 99, 99);
        theme.variables.insert("primary".to_string(), override_color);
        assert_eq!(theme.resolve("primary"), Some(TcssColor::Rgb(99, 99, 99)));
    }

    #[test]
    fn variables_override_shade_variant() {
        let mut theme = default_dark_theme();
        let override_color = TcssColor::Rgb(42, 42, 42);
        theme.variables.insert("primary-lighten-1".to_string(), override_color);
        assert_eq!(
            theme.resolve("primary-lighten-1"),
            Some(TcssColor::Rgb(42, 42, 42))
        );
    }

    // --- lighten_color direct tests ---

    #[test]
    fn lighten_color_positive_delta() {
        let base = TcssColor::Rgb(100, 100, 100);
        let lighter = lighten_color(base, 0.1);
        let base_l = rgb_to_hsl(100, 100, 100).2;
        match lighter {
            TcssColor::Rgb(r, g, b) => {
                let new_l = rgb_to_hsl(r, g, b).2;
                assert!(new_l > base_l);
            }
            _ => panic!("expected Rgb"),
        }
    }

    #[test]
    fn lighten_color_negative_delta_darkens() {
        let base = TcssColor::Rgb(100, 100, 100);
        let darker = lighten_color(base, -0.1);
        let base_l = rgb_to_hsl(100, 100, 100).2;
        match darker {
            TcssColor::Rgb(r, g, b) => {
                let new_l = rgb_to_hsl(r, g, b).2;
                assert!(new_l < base_l);
            }
            _ => panic!("expected Rgb"),
        }
    }

    #[test]
    fn lighten_color_clamps_to_max() {
        let base = TcssColor::Rgb(250, 250, 250);
        let result = lighten_color(base, 1.0);
        match result {
            TcssColor::Rgb(r, g, b) => {
                let l = rgb_to_hsl(r, g, b).2;
                assert!(l <= 1.0);
            }
            _ => panic!("expected Rgb"),
        }
    }

    #[test]
    fn lighten_color_non_rgb_unchanged() {
        let reset = TcssColor::Reset;
        assert_eq!(lighten_color(reset, 0.5), TcssColor::Reset);

        let named = TcssColor::Named("red");
        assert_eq!(lighten_color(named, 0.5), TcssColor::Named("red"));
    }
}
