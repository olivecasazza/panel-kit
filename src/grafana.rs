//! Grafana embed panel component.
//!
//! Provides [`GrafanaPanel`], a Dioxus component that embeds a Grafana
//! dashboard — or a single dashboard panel — through an `<iframe>`, sized to
//! fill its host panel. It is the embed counterpart of the `bevy` canvas
//! component: instead of mounting a wasm module it points an iframe at a Grafana
//! URL, building the correct shape for the two cases:
//!
//! - **Single panel** ([`panel_id`](GrafanaPanelProps::panel_id) is `Some`):
//!   the `/d-solo/` endpoint, which renders one panel with no dashboard chrome.
//! - **Full dashboard** ([`panel_id`](GrafanaPanelProps::panel_id) is `None`):
//!   the `/d/` endpoint with `&kiosk`, which hides Grafana's top/side chrome.
//!
//! Template variables, theme, and the time range are appended as query
//! parameters ([`vars`](GrafanaPanelProps::vars) → `&var-<k>=<v>`,
//! [`theme`](GrafanaPanelProps::theme) → `&theme=…`,
//! [`from`](GrafanaPanelProps::from)/[`to`](GrafanaPanelProps::to) →
//! `&from=…&to=…`, defaulting to `now-6h`/`now`).
//!
//! # Embedding caveat
//!
//! For the iframe to render at all, the Grafana server must allow framing:
//! set `[security] allow_embedding = true` (otherwise Grafana sends
//! `X-Frame-Options: deny` and the browser blocks the frame). For the panel
//! to load without an interactive login, either enable anonymous viewing
//! (`[auth.anonymous] enabled = true`) or rely on the viewer's existing
//! Grafana session cookie — which, cross-origin, may additionally require
//! `[security] cookie_samesite = none` (and therefore HTTPS).
//!
//! # Example
//!
//! ```no_run
//! use dioxus::prelude::*;
//! use panel_kit::GrafanaPanel;
//!
//! fn metrics_panel() -> Element {
//!     rsx! {
//!         GrafanaPanel {
//!             base_url: "https://grafana.example.com",
//!             dashboard_uid: "abc123",
//!             slug: "cluster-overview",
//!             panel_id: 8,
//!             vars: vec![("namespace".to_string(), "prod".to_string())],
//!             theme: "dark",
//!         }
//!     }
//! }
//! ```

use dioxus::prelude::*;

/// Percent-encode a Grafana template-variable value for use in a query
/// string. Dependency-light: encodes the characters that would otherwise
/// break the `&var-k=v` parameter (`&`, `=`, `#`, `%`, `+`, space, quotes,
/// `<`/`>`), leaving everything else untouched. Most values (namespaces,
/// pod names, instances) pass through verbatim.
fn encode_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' | '*' | ':' | '/' => {
                out.push(ch)
            }
            ' ' => out.push_str("%20"),
            other => {
                let mut buf = [0u8; 4];
                for b in other.encode_utf8(&mut buf).bytes() {
                    out.push_str(&format!("%{b:02X}"));
                }
            }
        }
    }
    out
}

/// Build the Grafana embed URL from the component props.
///
/// `/d-solo/` (single panel) when `panel_id` is `Some`, otherwise `/d/` with
/// `&kiosk` (full dashboard, chrome hidden). A trailing `/` on `base_url` is
/// stripped, the `slug` segment is omitted entirely when `None`, and the time
/// range defaults to `from=now-6h&to=now`.
#[allow(clippy::too_many_arguments)]
fn build_url(
    base_url: &str,
    dashboard_uid: &str,
    slug: Option<&str>,
    panel_id: Option<u32>,
    vars: &[(String, String)],
    theme: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> String {
    let base = base_url.trim_end_matches('/');
    let endpoint = if panel_id.is_some() { "d-solo" } else { "d" };
    let mut url = match slug {
        Some(slug) if !slug.is_empty() => format!("{base}/{endpoint}/{dashboard_uid}/{slug}"),
        _ => format!("{base}/{endpoint}/{dashboard_uid}"),
    };

    // First query parameter uses `?`, the rest `&`.
    let mut sep = '?';
    let mut push = |url: &mut String, frag: String| {
        url.push(sep);
        url.push_str(&frag);
        sep = '&';
    };

    let from = from.unwrap_or("now-6h");
    let to = to.unwrap_or("now");
    push(&mut url, format!("from={}", encode_value(from)));
    push(&mut url, format!("to={}", encode_value(to)));

    if let Some(id) = panel_id {
        push(&mut url, format!("panelId={id}"));
    } else {
        // `kiosk` hides Grafana's chrome on the full-dashboard endpoint; it is
        // a no-op on `/d-solo/`, so only emit it for `/d/`.
        push(&mut url, "kiosk".to_string());
    }

    if let Some(theme) = theme {
        if !theme.is_empty() {
            push(&mut url, format!("theme={}", encode_value(theme)));
        }
    }

    for (k, v) in vars {
        push(
            &mut url,
            format!("var-{}={}", encode_value(k), encode_value(v)),
        );
    }

    url
}

/// A Dioxus component that embeds a Grafana dashboard or single panel in a
/// responsive `<iframe>` filling its host panel.
///
/// See the [module docs](crate::grafana) for the URL shapes it builds and the
/// Grafana-side `allow_embedding` / anonymous-access requirements. Styling
/// comes from the `.grafana-panel*` rules in [`crate::CSS`].
#[component]
pub fn GrafanaPanel(
    /// Grafana base URL, e.g. `"https://grafana.example.com"`. A trailing
    /// slash is stripped.
    base_url: String,
    /// Dashboard UID — the stable identifier in the dashboard's URL
    /// (`/d/<uid>/…`).
    dashboard_uid: String,
    /// Optional URL slug (human-readable dashboard name). Omitted from the
    /// URL entirely when `None`; Grafana resolves the dashboard from the UID
    /// alone.
    #[props(default)]
    slug: Option<String>,
    /// When `Some`, embed just that panel via the `/d-solo/` endpoint; when
    /// `None`, embed the full dashboard via `/d/` with `&kiosk` (chrome
    /// hidden).
    #[props(default)]
    panel_id: Option<u32>,
    /// Dashboard template variables, rendered as `&var-<k>=<v>` query
    /// parameters (values percent-encoded).
    #[props(default)]
    vars: Vec<(String, String)>,
    /// Grafana theme override, typically `"light"` or `"dark"`; appended as
    /// `&theme=…`. Omitted when `None` (Grafana uses its own default).
    #[props(default)]
    theme: Option<String>,
    /// Time-range start (Grafana time syntax, e.g. `"now-24h"` or an epoch
    /// millis string). Defaults to `"now-6h"`.
    #[props(default)]
    from: Option<String>,
    /// Time-range end (Grafana time syntax). Defaults to `"now"`.
    #[props(default)]
    to: Option<String>,
    /// Accessible title for the iframe; also its tooltip. Defaults to
    /// `"Grafana"`.
    #[props(default = "Grafana".to_string())]
    title: String,
) -> Element {
    let url = build_url(
        &base_url,
        &dashboard_uid,
        slug.as_deref(),
        panel_id,
        &vars,
        theme.as_deref(),
        from.as_deref(),
        to.as_deref(),
    );

    rsx! {
        div { class: "grafana-panel",
            iframe {
                class: "grafana-frame",
                src: "{url}",
                title: "{title}",
                // Grafana panels are interactive (zoom, tooltips); allow the
                // frame to drive its own scroll/fullscreen.
                allow: "fullscreen",
                "loading": "lazy",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn single_panel_uses_d_solo_and_panel_id() {
        let url = build_url(
            "https://g.example.com",
            "uid1",
            Some("overview"),
            Some(8),
            &[],
            None,
            None,
            None,
        );
        assert_eq!(
            url,
            "https://g.example.com/d-solo/uid1/overview?from=now-6h&to=now&panelId=8"
        );
    }

    #[test]
    fn full_dashboard_uses_d_and_kiosk() {
        let url = build_url(
            "https://g.example.com/",
            "uid1",
            Some("overview"),
            None,
            &[],
            None,
            None,
            None,
        );
        // Trailing slash on base is stripped; full dashboard gets &kiosk.
        assert_eq!(
            url,
            "https://g.example.com/d/uid1/overview?from=now-6h&to=now&kiosk"
        );
    }

    #[test]
    fn slug_omitted_when_none() {
        let url = build_url(
            "https://g.example.com",
            "uid1",
            None,
            Some(2),
            &[],
            None,
            None,
            None,
        );
        assert_eq!(
            url,
            "https://g.example.com/d-solo/uid1?from=now-6h&to=now&panelId=2"
        );
    }

    #[test]
    fn theme_vars_and_time_range() {
        let url = build_url(
            "https://g.example.com",
            "uid1",
            Some("ov"),
            Some(3),
            &vars(&[("namespace", "prod"), ("pod", "a b")]),
            Some("dark"),
            Some("now-24h"),
            Some("now-1h"),
        );
        assert_eq!(
            url,
            "https://g.example.com/d-solo/uid1/ov?from=now-24h&to=now-1h&panelId=3&theme=dark&var-namespace=prod&var-pod=a%20b"
        );
    }

    #[test]
    fn encode_value_escapes_specials() {
        assert_eq!(encode_value("a b&c=d"), "a%20b%26c%3Dd");
        assert_eq!(encode_value("plain-Value_1.0"), "plain-Value_1.0");
    }
}
