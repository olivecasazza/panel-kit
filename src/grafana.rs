//! Grafana embed components.
//!
//! Two components, one shared frame:
//!
//! - [`GrafanaDashboard`] — a whole dashboard through `/d/` with `&kiosk`
//!   (Grafana's own top/side chrome hidden). Use it when the dashboard's own
//!   layout is the thing you want, and give it a tall host panel: a 15-panel
//!   dashboard is ~1400px and will scroll inside a short frame.
//! - [`GrafanaPanel`] — a single panel through `/d-solo/`, which renders one
//!   visualisation and nothing else. Use it when you want Grafana's data in
//!   *your* layout: several `GrafanaPanel`s in a grid read far better than one
//!   squashed dashboard.
//!
//! Both wrap the iframe in panel-kit chrome: the host background shows through
//! until the frame loads (no white flash), a spinner covers the gap, and an
//! "open ↗" link to the real dashboard is always present — which doubles as the
//! error story, since a browser gives JS no way to detect an
//! `X-Frame-Options`-blocked frame.
//!
//! Template variables, theme, and the time range become query parameters
//! (`&var-<k>=<v>`, `&theme=…`, `&from=…&to=…`, defaulting to `now-6h`/`now`).
//!
//! # Styling caveat
//!
//! The iframe is cross-origin: its DOM is unreachable, so Grafana's own styles
//! cannot be transformed from here. The `theme` prop (`"light"` / `"dark"` /
//! `"system"`) is the whole styling surface Grafana exposes over the URL —
//! everything else has to match on panel-kit's side of the border. In
//! particular Grafana does not honour a `transparent` URL parameter; a panel's
//! transparent background is a per-panel setting in the dashboard JSON, and
//! even then it does not carry into an iframe.
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
//! use panel_kit::{GrafanaDashboard, GrafanaPanel};
//!
//! fn whole_dashboard() -> Element {
//!     rsx! {
//!         GrafanaDashboard {
//!             base_url: "https://grafana.example.com",
//!             dashboard_uid: "abc123",
//!             slug: "cluster-overview",
//!             vars: vec![("namespace".to_string(), "prod".to_string())],
//!             theme: "dark",
//!         }
//!     }
//! }
//!
//! fn one_panel() -> Element {
//!     rsx! {
//!         GrafanaPanel {
//!             base_url: "https://grafana.example.com",
//!             dashboard_uid: "abc123",
//!             panel_id: 8,
//!             theme: "dark",
//!             title: "Training loss",
//!         }
//!     }
//! }
//! ```

use crate::Spinner;
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

/// `{base}/{endpoint}/{uid}[/{slug}]`, with a trailing `/` on `base_url`
/// stripped and the slug segment omitted when absent (Grafana resolves the
/// dashboard from the UID alone).
fn build_path(base_url: &str, dashboard_uid: &str, slug: Option<&str>, endpoint: &str) -> String {
    let base = base_url.trim_end_matches('/');
    match slug {
        Some(slug) if !slug.is_empty() => format!("{base}/{endpoint}/{dashboard_uid}/{slug}"),
        _ => format!("{base}/{endpoint}/{dashboard_uid}"),
    }
}

/// Query string (leading `?`) shared by the embed and "open in Grafana" URLs:
/// time range first (defaulting to `now-6h`/`now`), then `extra` — the bit
/// that differs per endpoint (`kiosk`, `panelId=…`, `viewPanel=…`) — then
/// theme and template variables.
fn build_query(
    extra: &[String],
    vars: &[(String, String)],
    theme: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> String {
    let mut q = format!(
        "?from={}&to={}",
        encode_value(from.unwrap_or("now-6h")),
        encode_value(to.unwrap_or("now"))
    );
    for frag in extra {
        q.push('&');
        q.push_str(frag);
    }
    if let Some(theme) = theme {
        if !theme.is_empty() {
            q.push_str(&format!("&theme={}", encode_value(theme)));
        }
    }
    for (k, v) in vars {
        q.push_str(&format!("&var-{}={}", encode_value(k), encode_value(v)));
    }
    q
}

/// The chrome both components share: the iframe, a spinner covering the load,
/// and an always-present escape hatch to the real dashboard.
///
/// `src` is the embed URL, `open_url` the human-facing one (same dashboard, no
/// kiosk) opened in a new tab. The spinner is dismissed on the iframe's `load`
/// event; a frame blocked by `X-Frame-Options` may also fire `load`, which is
/// exactly why `open_url` is not conditional on failure.
#[component]
fn GrafanaFrame(src: String, open_url: String, title: String) -> Element {
    let mut loaded = use_signal(|| false);
    // Re-mounting on a prop change would be cheaper than a manual reset, but
    // Dioxus reuses the node; reset explicitly so a re-scoped embed re-spins.
    use_effect(use_reactive(&src, move |_| loaded.set(false)));

    rsx! {
        div { class: "grafana-panel",
            iframe {
                class: "grafana-frame",
                src: "{src}",
                title: "{title}",
                // Grafana panels are interactive (zoom, tooltips); allow the
                // frame to drive its own scroll/fullscreen.
                allow: "fullscreen",
                // Deliberately NOT loading="lazy": a panel that is minimized,
                // docked, or scrolled out of the workspace never fetches, so
                // re-scoping it leaves the spinner up with no load event ever
                // coming. Eager costs one request the user asked for anyway.
                onload: move |_| loaded.set(true),
            }
            if !loaded() {
                div { class: "grafana-loading", Spinner { label: "Grafana" } }
            }
            a {
                class: "grafana-open",
                href: "{open_url}",
                target: "_blank",
                rel: "noopener noreferrer",
                title: "Open in Grafana",
                "open ↗"
            }
        }
    }
}

/// Embed a whole Grafana dashboard via `/d/` with `&kiosk` (Grafana's chrome
/// hidden), filling its host panel.
///
/// Dashboards are tall — give this a tall panel, or reach for [`GrafanaPanel`]
/// and lay the individual panels out yourself. See the
/// [module docs](crate::grafana) for the styling and `allow_embedding`
/// caveats; styling comes from the `.grafana-panel*` rules in [`crate::CSS`].
#[component]
pub fn GrafanaDashboard(
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
    /// Dashboard template variables, rendered as `&var-<k>=<v>` query
    /// parameters (values percent-encoded).
    #[props(default)]
    vars: Vec<(String, String)>,
    /// Grafana theme override — `"light"`, `"dark"`, or `"system"`; appended
    /// as `&theme=…`. This is the only styling lever the URL exposes (the
    /// iframe is cross-origin). Omitted when `None`.
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
    /// `"Grafana dashboard"`.
    #[props(default = "Grafana dashboard".to_string())]
    title: String,
) -> Element {
    let path = build_path(&base_url, &dashboard_uid, slug.as_deref(), "d");
    let query = |extra: &[String]| {
        build_query(
            extra,
            &vars,
            theme.as_deref(),
            from.as_deref(),
            to.as_deref(),
        )
    };
    rsx! {
        GrafanaFrame {
            src: format!("{path}{}", query(&["kiosk".to_string()])),
            open_url: format!("{path}{}", query(&[])),
            title,
        }
    }
}

/// Embed a single Grafana panel via `/d-solo/`, filling its host panel.
///
/// The counterpart to [`GrafanaDashboard`]: one visualisation, no dashboard
/// layout, so several of these compose into a panel-kit layout of your own.
///
/// `panel_id` is the numeric panel id from the dashboard JSON. Beware that
/// dashboards provisioned without explicit `id` fields get ids auto-assigned
/// on import, which are not stable across re-provisioning — pin them in the
/// dashboard source before deep-linking. See the
/// [module docs](crate::grafana) for the styling and `allow_embedding`
/// caveats; styling comes from the `.grafana-panel*` rules in [`crate::CSS`].
#[component]
pub fn GrafanaPanel(
    /// Grafana base URL, e.g. `"https://grafana.example.com"`. A trailing
    /// slash is stripped.
    base_url: String,
    /// Dashboard UID — the stable identifier in the dashboard's URL
    /// (`/d/<uid>/…`).
    dashboard_uid: String,
    /// Numeric id of the panel within that dashboard.
    panel_id: u32,
    /// Optional URL slug (human-readable dashboard name). Omitted from the
    /// URL entirely when `None`; Grafana resolves the dashboard from the UID
    /// alone.
    #[props(default)]
    slug: Option<String>,
    /// Dashboard template variables, rendered as `&var-<k>=<v>` query
    /// parameters (values percent-encoded).
    #[props(default)]
    vars: Vec<(String, String)>,
    /// Grafana theme override — `"light"`, `"dark"`, or `"system"`; appended
    /// as `&theme=…`. This is the only styling lever the URL exposes (the
    /// iframe is cross-origin). Omitted when `None`.
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
    /// `"Grafana panel"`.
    #[props(default = "Grafana panel".to_string())]
    title: String,
) -> Element {
    let query = |extra: &[String]| {
        build_query(
            extra,
            &vars,
            theme.as_deref(),
            from.as_deref(),
            to.as_deref(),
        )
    };
    let solo = build_path(&base_url, &dashboard_uid, slug.as_deref(), "d-solo");
    // The escape hatch opens the panel in its dashboard, not solo again.
    let dash = build_path(&base_url, &dashboard_uid, slug.as_deref(), "d");
    rsx! {
        GrafanaFrame {
            src: format!("{solo}{}", query(&[format!("panelId={panel_id}")])),
            open_url: format!("{dash}{}", query(&[format!("viewPanel={panel_id}")])),
            title,
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

    /// Mirrors what `GrafanaPanel` builds, so the tests cover the real shape.
    // Mirrors the component's props one-for-one; grouping them here would stop
    // it mirroring.
    #[allow(clippy::too_many_arguments)]
    fn solo_url(
        base: &str,
        uid: &str,
        slug: Option<&str>,
        panel_id: u32,
        vars: &[(String, String)],
        theme: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> String {
        format!(
            "{}{}",
            build_path(base, uid, slug, "d-solo"),
            build_query(&[format!("panelId={panel_id}")], vars, theme, from, to)
        )
    }

    /// Mirrors what `GrafanaDashboard` builds.
    fn dash_url(
        base: &str,
        uid: &str,
        slug: Option<&str>,
        vars: &[(String, String)],
        theme: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> String {
        format!(
            "{}{}",
            build_path(base, uid, slug, "d"),
            build_query(&["kiosk".to_string()], vars, theme, from, to)
        )
    }

    #[test]
    fn single_panel_uses_d_solo_and_panel_id() {
        assert_eq!(
            solo_url(
                "https://g.example.com",
                "uid1",
                Some("overview"),
                8,
                &[],
                None,
                None,
                None
            ),
            "https://g.example.com/d-solo/uid1/overview?from=now-6h&to=now&panelId=8"
        );
    }

    #[test]
    fn full_dashboard_uses_d_and_kiosk() {
        // Trailing slash on base is stripped; full dashboard gets &kiosk.
        assert_eq!(
            dash_url(
                "https://g.example.com/",
                "uid1",
                Some("overview"),
                &[],
                None,
                None,
                None
            ),
            "https://g.example.com/d/uid1/overview?from=now-6h&to=now&kiosk"
        );
    }

    #[test]
    fn slug_omitted_when_none() {
        assert_eq!(
            solo_url(
                "https://g.example.com",
                "uid1",
                None,
                2,
                &[],
                None,
                None,
                None
            ),
            "https://g.example.com/d-solo/uid1?from=now-6h&to=now&panelId=2"
        );
    }

    #[test]
    fn theme_vars_and_time_range() {
        assert_eq!(
            solo_url(
                "https://g.example.com",
                "uid1",
                Some("ov"),
                3,
                &vars(&[("namespace", "prod"), ("pod", "a b")]),
                Some("dark"),
                Some("now-24h"),
                Some("now-1h"),
            ),
            "https://g.example.com/d-solo/uid1/ov?from=now-24h&to=now-1h&panelId=3&theme=dark&var-namespace=prod&var-pod=a%20b"
        );
    }

    /// The "open in Grafana" link drops kiosk and points at the dashboard —
    /// for a solo embed, focused on that panel.
    #[test]
    fn open_links_drop_kiosk() {
        let q = |extra: &[String]| build_query(extra, &[], None, None, None);
        let dash = build_path("https://g.example.com", "uid1", None, "d");
        assert_eq!(
            format!("{dash}{}", q(&[])),
            "https://g.example.com/d/uid1?from=now-6h&to=now"
        );
        assert_eq!(
            format!("{dash}{}", q(&["viewPanel=8".to_string()])),
            "https://g.example.com/d/uid1?from=now-6h&to=now&viewPanel=8"
        );
    }

    #[test]
    fn encode_value_escapes_specials() {
        assert_eq!(encode_value("a b&c=d"), "a%20b%26c%3Dd");
        assert_eq!(encode_value("plain-Value_1.0"), "plain-Value_1.0");
    }
}
