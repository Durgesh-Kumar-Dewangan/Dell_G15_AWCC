//! "Frost Glass" visual theme.
//!
//! GTK4 CSS has no `backdrop-filter`/blur primitive, and live blur-behind-window
//! is a compositor feature that isn't reliably available across Wayland/X11
//! desktop environments — so this does NOT claim to blur whatever's behind the
//! window (unlike macOS "Liquid Glass"/vibrancy). What it *does* do, entirely
//! within what GTK4 CSS actually supports, is a real glassmorphism treatment:
//! translucent layered "frost" panels over a deep gradient background, soft
//! inner sheen highlights that read as light catching a glass edge, and a cool
//! cyan accent — chosen deliberately because it echoes the app's own subject
//! (thermal/cooling), not as a generic decoration.
//!
//! Call [`install`] once, right after `adw::Application` is constructed and
//! before the first window is presented.

const GLASS_CSS: &str = r#"
/* ---- Palette --------------------------------------------------------- */
@define-color glass_bg_deep    #0a1420;
@define-color glass_bg_mid     #0e2233;
@define-color glass_panel      rgba(255, 255, 255, 0.055);
@define-color glass_panel_hi   rgba(255, 255, 255, 0.10);
@define-color glass_border     rgba(255, 255, 255, 0.14);
@define-color glass_sheen      rgba(255, 255, 255, 0.16);
@define-color frost_accent     #5eead4;   /* icy cyan-teal: the "frost" signature */
@define-color frost_accent_dim #2dd4bf;
@define-color frost_text       #eaf6f6;
@define-color frost_text_dim   #9fb8c2;

/* ---- Window shell ------------------------------------------------------ */
window, .background {
    background-image:
        radial-gradient(circle at 15% -10%, alpha(@frost_accent, 0.10) 0%, transparent 45%),
        linear-gradient(180deg, @glass_bg_mid 0%, @glass_bg_deep 60%);
    background-color: @glass_bg_deep;
    color: @frost_text;
}

headerbar {
    background-color: rgba(10, 20, 32, 0.55);
    background-image: linear-gradient(180deg, rgba(255,255,255,0.05), transparent);
    border-bottom: 1px solid @glass_border;
    box-shadow: none;
    color: @frost_text;
}

headerbar windowtitle,
headerbar .title {
    color: @frost_text;
}

/* ---- Glass cards (list rows, preference groups) ----------------------- */
list.boxed-list,
list.boxed-list > row,
.card {
    background-color: @glass_panel;
    background-image: linear-gradient(180deg, @glass_sheen 0%, transparent 40%);
    border: 1px solid @glass_border;
    border-radius: 14px;
    box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.08),
        0 6px 18px rgba(0, 0, 0, 0.35);
}

list.boxed-list > row {
    margin: 3px 0;
    padding: 4px 2px;
}

list.boxed-list > row:hover {
    background-color: @glass_panel_hi;
}

/* AdwPreferencesGroup renders its own frame; keep it consistent with .card */
preferencesgroup > box > list {
    background-color: @glass_panel;
    border: 1px solid @glass_border;
    border-radius: 14px;
}

/* ---- Frame (used for the temperature history graphs) ------------------- */
frame {
    background-color: @glass_panel;
    border: 1px solid @glass_border;
    border-radius: 12px;
}
frame > border {
    border: none;
}

/* ---- Buttons / toggles -------------------------------------------------- */
button {
    border-radius: 10px;
    transition: background-color 150ms ease, box-shadow 150ms ease;
}

togglebutton:checked,
button.suggested-action {
    background-color: alpha(@frost_accent, 0.22);
    background-image: linear-gradient(180deg, alpha(@frost_accent, 0.18), transparent);
    border: 1px solid alpha(@frost_accent, 0.55);
    color: @frost_text;
    box-shadow: 0 0 12px alpha(@frost_accent, 0.25);
}

/* ---- Sliders / level bars: frosted track, glowing fill ------------------ */
scale trough {
    background-color: rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    min-height: 6px;
}
scale trough highlight {
    background-color: @frost_accent_dim;
    background-image: linear-gradient(90deg, @frost_accent_dim, @frost_accent);
    border-radius: 999px;
}
scale slider {
    background-color: @frost_text;
    box-shadow: 0 0 6px alpha(@frost_accent, 0.7);
    min-width: 14px;
    min-height: 14px;
    border-radius: 999px;
}

levelbar block.filled {
    background-color: @frost_accent_dim;
    background-image: linear-gradient(90deg, @frost_accent_dim, @frost_accent);
    border-radius: 999px;
}
levelbar block.empty {
    background-color: rgba(255, 255, 255, 0.08);
    border-radius: 999px;
}
levelbar trough {
    border-radius: 999px;
    padding: 2px;
}

/* ---- Banner (daemon-unreachable notice) --------------------------------- */
banner {
    background-color: rgba(94, 234, 212, 0.12);
    border-bottom: 1px solid alpha(@frost_accent, 0.4);
    color: @frost_text;
}

/* ---- Dim labels: keep readable against the dark glass background ------- */
label.dim-label {
    color: @frost_text_dim;
}
"#;

/// Load and apply the frost-glass theme for the whole application. Uses
/// `STYLE_PROVIDER_PRIORITY_APPLICATION` so the user's own GTK theme (light/dark
/// switch, high-contrast accessibility themes, etc.) can still override it via
/// their own higher-priority user stylesheet if they want to opt out.
pub fn install() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(GLASS_CSS);

    let Some(display) = gtk::gdk::Display::default() else {
        tracing::warn!("theme: no default GDK display available, skipping glass theme");
        return;
    };

    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
