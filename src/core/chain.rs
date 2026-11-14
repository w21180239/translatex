//! Round-trip planning: turn a request's chain, preset, depth or pivot into
//! hops. Pure, so precedence and validation are testable without providers.

use crate::core::config::Settings;
use crate::core::error::{Error, Result};
use crate::core::lang::{alternate_pivot, default_pivot, is_chinese, is_known, normalize};
use crate::core::types::RoundtripRequest;

pub const PRESETS: &[&str] = &["light", "standard", "strong"];

/// Hop target: a language, or back to the (possibly still undetected) source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Lang(String),
    Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HopSpec {
    /// `None` races the fallback chain.
    pub engine: Option<String>,
    pub to: Target,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub hops: Vec<HopSpec>,
    /// Built from a guess about an `auto` source; rebuild once detection says
    /// the guessed first hop is the source language itself.
    pub guessed: bool,
    /// The final hop back to the source was appended to an explicit chain, so
    /// it is dropped when the chain already ends in the detected source.
    pub appended_back: bool,
}

/// `[engine:]lang` (`google:zh`, `zh`, `auto:ja`).
pub fn parse_hop(spec: &str) -> Result<HopSpec> {
    let spec = spec.trim();
    let (engine, lang) = match spec.split_once(':') {
        Some((engine, lang)) => (Some(engine.trim().to_lowercase()), lang),
        None => (None, spec),
    };
    let engine = engine.filter(|e| !e.is_empty() && e != "auto");
    let code = normalize(lang);
    if code == "auto" || !is_known(&code) {
        return Err(Error::InvalidInput(format!(
            "chain hop {spec:?}: unknown language {:?}",
            lang.trim()
        )));
    }
    Ok(HopSpec {
        engine,
        to: Target::Lang(code),
    })
}

/// Hops for a request. `source` is the normalised source language (`auto`
/// allowed). Precedence: `chain` > `preset` > `depth` > non-auto `pivot` >
/// configured `roundtrip_preset`.
pub fn plan(req: &RoundtripRequest, settings: &Settings, source: &str) -> Result<Plan> {
    if !req.chain.is_empty() {
        return explicit(&req.chain, source);
    }
    if let Some(preset) = &req.preset {
        return expand_preset(preset, source, settings);
    }
    if let Some(depth) = req.depth {
        return expand_depth(depth, source, settings);
    }
    let pivot = normalize(&req.pivot);
    if pivot != "auto" {
        return single_pivot(&pivot, source);
    }
    expand_preset(&settings.roundtrip_preset(), source, settings)
}

fn explicit(chain: &[String], source: &str) -> Result<Plan> {
    let mut hops = chain
        .iter()
        .map(|s| parse_hop(s))
        .collect::<Result<Vec<_>>>()?;
    let mut prev = source.to_string();
    for hop in &hops {
        let Target::Lang(lang) = &hop.to else {
            continue;
        };
        if *lang == prev {
            return Err(Error::InvalidInput(format!(
                "chain hop into {lang} follows text already in {lang}"
            )));
        }
        prev = lang.clone();
    }
    let appended_back = prev != source || source == "auto";
    if appended_back {
        hops.push(HopSpec {
            engine: None,
            to: Target::Source,
        });
    }
    Ok(Plan {
        hops,
        guessed: false,
        appended_back,
    })
}

fn single_pivot(pivot: &str, source: &str) -> Result<Plan> {
    if !is_known(pivot) {
        return Err(Error::InvalidInput(format!(
            "unknown pivot language {pivot}"
        )));
    }
    // Sending text to its own language is not a round trip.
    let pivot = if pivot == source {
        alternate_pivot(pivot)
    } else {
        pivot
    };
    Ok(mt(vec![pivot.to_string()], false))
}

fn expand_depth(depth: u8, source: &str, settings: &Settings) -> Result<Plan> {
    if !(1..=3).contains(&depth) {
        return Err(Error::InvalidInput(format!(
            "depth must be 1-3, got {depth}"
        )));
    }
    let configured = settings.pivot_override().map(|p| normalize(&p));
    if depth == 1 {
        if let Some(pivot) = &configured {
            return single_pivot(pivot, source);
        }
    }
    // A configured pivot leads; the rest follow the default path, skipping the
    // source and repeats (a Japanese source must not hop into Japanese).
    let path: [&str; 4] = if is_chinese(source) {
        ["en", "ja", "fi", "ko"]
    } else {
        ["zh", "ja", "fi", "ko"]
    };
    let mut langs: Vec<String> = configured
        .into_iter()
        .filter(|p| p.as_str() != source)
        .collect();
    for lang in path {
        if langs.len() == depth as usize {
            break;
        }
        if lang != source && !langs.iter().any(|l| l == lang) {
            langs.push(lang.to_string());
        }
    }
    Ok(mt(langs, source == "auto"))
}

fn mt(langs: Vec<String>, guessed: bool) -> Plan {
    let mut hops: Vec<HopSpec> = langs
        .into_iter()
        .map(|lang| HopSpec {
            engine: None,
            to: Target::Lang(lang),
        })
        .collect();
    hops.push(HopSpec {
        engine: None,
        to: Target::Source,
    });
    Plan {
        hops,
        guessed,
        appended_back: false,
    }
}

fn expand_preset(name: &str, source: &str, settings: &Settings) -> Result<Plan> {
    match name.trim().to_lowercase().as_str() {
        "light" => expand_depth(2, source, settings),
        "standard" => expand_depth(3, source, settings),
        // Alternate two engines so each hop meets a different model.
        "strong" => {
            let mut plan = expand_depth(3, source, settings)?;
            for (i, hop) in plan.hops.iter_mut().enumerate() {
                hop.engine = Some(if i % 2 == 0 { "google" } else { "edge" }.to_string());
            }
            Ok(plan)
        }
        other => Err(Error::InvalidInput(format!(
            "unknown roundtrip preset {other:?}; valid: {}",
            PRESETS.join(", ")
        ))),
    }
}

/// Language for a hop back to the source: the detected language when known,
/// otherwise the recommended partner of the first hop. Never the language the
/// text is already in.
pub fn resolve_source(detected: &str, first_hop: &str, prev: &str) -> String {
    let back = if detected == "auto" || !is_known(detected) {
        default_pivot(first_hop).to_string()
    } else {
        detected.to_string()
    };
    if back == prev {
        alternate_pivot(prev).to_string()
    } else {
        back
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_preset_names_are_rejected() {
        let s = Settings::default();
        for old in ["mt", "mt-deep", "llm", "mt-fix", "llm-fix"] {
            let mut r = req();
            r.preset = Some(old.into());
            let err = plan(&r, &s, "en").unwrap_err();
            assert_eq!(err.code(), "INVALID_INPUT", "{old}");
            assert!(err.to_string().contains("light, standard, strong"), "{old}");
        }
    }

    #[test]
    fn default_preset_is_light() {
        let s = Settings::default();
        assert_eq!(s.roundtrip_preset(), "light");
        let p = plan(&req(), &s, "en").unwrap();
        assert_eq!(langs(&p), vec!["zh", "ja", "SRC"]);
    }

    fn req() -> RoundtripRequest {
        serde_json::from_str(r#"{"text":"hi"}"#).unwrap()
    }

    fn langs(plan: &Plan) -> Vec<String> {
        plan.hops
            .iter()
            .map(|h| match &h.to {
                Target::Lang(l) => l.clone(),
                Target::Source => "SRC".into(),
            })
            .collect()
    }

    fn engines(plan: &Plan) -> Vec<String> {
        plan.hops
            .iter()
            .map(|h| h.engine.clone().unwrap_or_else(|| "auto".into()))
            .collect()
    }

    #[test]
    fn hop_specs_parse() {
        assert_eq!(
            parse_hop("google:zh-CN").unwrap(),
            HopSpec {
                engine: Some("google".into()),
                to: Target::Lang("zh".into())
            }
        );
        assert_eq!(parse_hop(" ja ").unwrap().engine, None);
        assert_eq!(parse_hop("auto:fi").unwrap().engine, None);
        for bad in ["klingon", "google:", "edge:auto", ""] {
            assert_eq!(parse_hop(bad).unwrap_err().code(), "INVALID_INPUT", "{bad}");
        }
    }

    #[test]
    fn explicit_chain_appends_the_way_back() {
        let mut r = req();
        r.chain = vec!["google:zh".into(), "google:fi".into()];
        let p = plan(&r, &Settings::default(), "en").unwrap();
        assert_eq!(langs(&p), vec!["zh", "fi", "SRC"]);
        assert_eq!(engines(&p), vec!["google", "google", "auto"]);
        assert!(p.appended_back && !p.guessed);

        r.chain = vec!["zh".into(), "edge:en".into()];
        let p = plan(&r, &Settings::default(), "en").unwrap();
        assert_eq!(langs(&p), vec!["zh", "en"], "already ends in the source");
        assert!(!p.appended_back);
    }

    #[test]
    fn explicit_chain_rejects_repeated_languages() {
        let mut r = req();
        r.chain = vec!["zh".into(), "google:zh".into()];
        let err = plan(&r, &Settings::default(), "en").unwrap_err();
        assert_eq!(err.code(), "INVALID_INPUT");
        r.chain = vec!["en".into(), "zh".into()];
        let err = plan(&r, &Settings::default(), "en").unwrap_err();
        assert_eq!(err.code(), "INVALID_INPUT", "first hop equals the source");
    }

    #[test]
    fn depth_follows_the_source_family() {
        let mut r = req();
        r.depth = Some(3);
        let p = plan(&r, &Settings::default(), "en").unwrap();
        assert_eq!(langs(&p), vec!["zh", "ja", "fi", "SRC"]);
        assert_eq!(engines(&p), vec!["auto"; 4]);
        let p = plan(&r, &Settings::default(), "zh").unwrap();
        assert_eq!(langs(&p), vec!["en", "ja", "fi", "SRC"]);
        r.depth = Some(1);
        let p = plan(&r, &Settings::default(), "auto").unwrap();
        assert_eq!(langs(&p), vec!["zh", "SRC"]);
        assert!(p.guessed);
        for bad in [0, 4] {
            r.depth = Some(bad);
            assert_eq!(
                plan(&r, &Settings::default(), "en").unwrap_err().code(),
                "INVALID_INPUT"
            );
        }
    }

    #[test]
    fn presets_expand_for_both_families() {
        let mut r = req();
        let s = Settings::default();
        r.preset = Some("light".into());
        assert_eq!(langs(&plan(&r, &s, "en").unwrap()), vec!["zh", "ja", "SRC"]);
        r.preset = Some("standard".into());
        let p = plan(&r, &s, "en").unwrap();
        assert_eq!(langs(&p), vec!["zh", "ja", "fi", "SRC"]);
        assert_eq!(engines(&p), vec!["auto"; 4]);
        let p = plan(&r, &s, "zh").unwrap();
        assert_eq!(langs(&p), vec!["en", "ja", "fi", "SRC"]);

        r.preset = Some("strong".into());
        let p = plan(&r, &s, "en").unwrap();
        assert_eq!(langs(&p), vec!["zh", "ja", "fi", "SRC"]);
        assert_eq!(engines(&p), vec!["google", "edge", "google", "edge"]);

        r.preset = Some("bogus".into());
        let err = plan(&r, &s, "en").unwrap_err();
        assert_eq!(err.code(), "INVALID_INPUT");
        assert!(err.to_string().contains("light, standard, strong"));
    }

    #[test]
    fn precedence_is_chain_preset_depth_pivot_setting() {
        let s = Settings {
            roundtrip_preset: Some("strong".into()),
            ..Settings::default()
        };
        let mut r = req();
        r.chain = vec!["ko".into()];
        r.preset = Some("standard".into());
        r.depth = Some(2);
        r.pivot = "ja".into();
        assert_eq!(langs(&plan(&r, &s, "en").unwrap()), vec!["ko", "SRC"]);
        r.chain.clear();
        assert_eq!(engines(&plan(&r, &s, "en").unwrap()), vec!["auto"; 4]);
        r.preset = None;
        assert_eq!(langs(&plan(&r, &s, "en").unwrap()), vec!["zh", "ja", "SRC"]);
        r.depth = None;
        assert_eq!(langs(&plan(&r, &s, "en").unwrap()), vec!["ja", "SRC"]);
        r.pivot = "auto".into();
        assert_eq!(engines(&plan(&r, &s, "en").unwrap())[3], "edge");
    }

    #[test]
    fn configured_pivot_steers_the_default_path() {
        let s = Settings {
            roundtrip_pivot: Some("ja".into()),
            ..Settings::default()
        };
        let mut r = req();
        r.depth = Some(1);
        let p = plan(&r, &s, "auto").unwrap();
        assert_eq!(langs(&p), vec!["ja", "SRC"]);
        assert!(!p.guessed, "a configured pivot is not a guess");
        // The default preset (depth 2) still leads with the configured pivot.
        r.depth = None;
        let p = plan(&r, &s, "en").unwrap();
        assert_eq!(langs(&p), vec!["ja", "zh", "SRC"]);
    }

    #[test]
    fn depth_path_skips_the_source_language() {
        let mut r = req();
        r.depth = Some(2);
        let s = Settings::default();
        assert_eq!(langs(&plan(&r, &s, "ja").unwrap()), vec!["zh", "fi", "SRC"]);
        r.depth = Some(3);
        assert_eq!(
            langs(&plan(&r, &s, "fi").unwrap()),
            vec!["zh", "ja", "ko", "SRC"]
        );
    }

    #[test]
    fn pivot_equal_to_source_moves() {
        let mut r = req();
        r.pivot = "en".into();
        assert_eq!(
            langs(&plan(&r, &Settings::default(), "en").unwrap()),
            vec!["zh", "SRC"]
        );
        r.pivot = "klingon".into();
        assert_eq!(
            plan(&r, &Settings::default(), "en").unwrap_err().code(),
            "INVALID_INPUT"
        );
    }

    #[test]
    fn source_resolution_avoids_the_current_language() {
        assert_eq!(resolve_source("en", "zh", "fi"), "en");
        assert_eq!(resolve_source("auto", "zh", "zh"), "en");
        assert_eq!(resolve_source("zh", "zh", "zh"), "en");
    }
}
