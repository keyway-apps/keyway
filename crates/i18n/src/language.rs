use std::cmp::Reverse;

use icu_locale_core::{
    LanguageIdentifier, Locale, langid,
    subtags::{Region, region},
};

const DEFAULT_LOCALE: &str = "en";

// Add a catalog and an entry here to support another language. The identifier
// describes the catalog's script and region even when its filename omits them.
// Table order decides ties, including requests for a language without a region.
const SUPPORTED_LOCALES: &[SupportedLocale] = &[
    SupportedLocale {
        name: "en",
        id: langid!("en"),
        region_aliases: &[],
    },
    SupportedLocale {
        name: "zh-CN",
        id: langid!("zh-Hans-CN"),
        region_aliases: &[region!("SG")],
    },
];

struct SupportedLocale {
    name: &'static str,
    id: LanguageIdentifier,
    // Regions that may use this catalog when the request omits its script.
    region_aliases: &'static [Region],
}

impl SupportedLocale {
    fn match_score(&self, requested: &LanguageIdentifier) -> Option<(bool, bool, bool)> {
        if requested.language != self.id.language {
            return None;
        }

        let region_matches = requested.region.is_some_and(|region| {
            self.id.region == Some(region) || self.region_aliases.contains(&region)
        });

        if let Some(script) = self.id.script {
            match requested.script {
                Some(requested_script) if requested_script != script => return None,
                // Infer a script only from the catalog's declared regions.
                None if requested.region.is_some() && !region_matches => return None,
                _ => {}
            }
        }

        // Exact identifiers win, then explicit script matches, then regions.
        // An explicit script can override the region, e.g. zh-Hans-TW.
        Some((
            *requested == self.id,
            requested.script.is_some() && requested.script == self.id.script,
            region_matches,
        ))
    }
}

pub(super) fn system_locale(locales: impl IntoIterator<Item = impl AsRef<str>>) -> &'static str {
    select_locale(locales, SUPPORTED_LOCALES)
}

fn select_locale(
    locales: impl IntoIterator<Item = impl AsRef<str>>,
    supported: &[SupportedLocale],
) -> &'static str {
    for locale in locales {
        let requested = locale.as_ref();
        let normalized = requested.replace('_', "-");
        let Ok(locale) = Locale::try_from_str(&normalized) else {
            tracing::debug!(requested_locale = requested, "Ignoring malformed system locale");
            continue;
        };
        let best_match = supported
            .iter()
            .filter_map(|candidate| {
                candidate
                    .match_score(&locale.id)
                    .map(|score| (candidate, score))
            })
            // min_by_key keeps the first entry when match scores are equal.
            .min_by_key(|(_, score)| Reverse(*score));
        if let Some((locale, _)) = best_match {
            tracing::debug!(
                requested_locale = normalized,
                selected_locale = locale.name,
                "Selected locale catalog from system preference"
            );
            return locale.name;
        }

        tracing::trace!(
            requested_locale = normalized,
            "No supported locale matches system preference"
        );
    }

    tracing::debug!(default_locale = DEFAULT_LOCALE, "Using default locale catalog");
    DEFAULT_LOCALE
}

#[cfg(test)]
mod tests {
    use super::{SUPPORTED_LOCALES, SupportedLocale, select_locale, system_locale};
    use icu_locale_core::{Locale, langid, subtags::region};

    #[test]
    fn selects_current_catalogs_without_crossing_scripts() {
        for locale in [
            "zh",
            "zh-CN",
            "zh_CN",
            "ZH-cn",
            "zh-Hans",
            "zh-Hans-CN",
            "zh-Hans-TW",
            "zh-SG",
        ] {
            assert_eq!(system_locale([locale]), "zh-CN", "{locale}");
        }
        for locale in [
            "en",
            "en-US",
            "fr-FR",
            "zh-TW",
            "zh-HK",
            "zh-Hant-CN",
            "zh-Latn-CN",
        ] {
            assert_eq!(system_locale([locale]), "en", "{locale}");
        }
    }

    #[test]
    fn ignores_extensions_and_private_use() {
        for locale in [
            "zh-u-nu-latn",
            "zh-x-hant",
            "zh-CN-u-co-pinyin",
            "zh-Hans-TW-x-test",
            "ZH_hAnS_cn_u_nu_latn",
        ] {
            assert_eq!(system_locale([locale]), "zh-CN", "{locale}");
        }
        for locale in ["en-x-zh-CN", "zh-TW-u-co-pinyin", "zh-Hant-CN-x-hans"] {
            assert_eq!(system_locale([locale]), "en", "{locale}");
        }
    }

    #[test]
    fn tries_system_preferences_before_falling_back() {
        assert_eq!(system_locale(["fr-FR", "zh-SG", "en-US"]), "zh-CN");
        assert_eq!(system_locale(["en-GB", "zh-CN"]), "en");
        assert_eq!(system_locale(["zh-Hant-CN", "zh-Hans"]), "zh-CN");
        assert_eq!(system_locale(["fr-FR", "ja-JP"]), "en");
        assert_eq!(system_locale([] as [&str; 0]), "en");
    }

    #[test]
    fn skips_malformed_identifiers() {
        for locale in [
            "",
            "zh-",
            "zh--CN",
            "zh-abcdefghij",
            "zh-12",
            "zh-u",
            "zh-x",
            "zh-Hans-CN-TW",
        ] {
            assert_eq!(system_locale([locale, "zh-CN"]), "zh-CN", "{locale}");
            assert_eq!(system_locale([locale]), "en", "{locale}");
        }
    }

    #[test]
    fn new_languages_and_regions_only_need_table_entries() {
        // These catalogs exist only in this test table, not in the application.
        let supported = [
            SupportedLocale {
                name: "ja",
                id: langid!("ja"),
                region_aliases: &[],
            },
            SupportedLocale {
                name: "fr",
                id: langid!("fr"),
                region_aliases: &[],
            },
            SupportedLocale {
                name: "pt-BR",
                id: langid!("pt-BR"),
                region_aliases: &[],
            },
            SupportedLocale {
                name: "pt-PT",
                id: langid!("pt-PT"),
                region_aliases: &[],
            },
        ];
        for (locale, expected) in [
            ("ja-JP", "ja"),
            ("JA_jp_u_ca_japanese", "ja"),
            ("fr-CA", "fr"),
            ("fr-FR-x-test", "fr"),
            ("pt-PT", "pt-PT"),
            ("pt-BR", "pt-BR"),
            ("pt", "pt-BR"),
        ] {
            assert_eq!(select_locale([locale], &supported), expected, "{locale}");
        }
        assert_eq!(select_locale(["de-DE", "fr-CA", "ja-JP"], &supported), "fr");
    }

    #[test]
    fn script_and_region_mappings_are_data_driven() {
        let supported = [
            SupportedLocale {
                name: "zh-TW",
                id: langid!("zh-Hant-TW"),
                region_aliases: &[region!("HK"), region!("MO")],
            },
            SupportedLocale {
                name: "zh-CN",
                id: langid!("zh-Hans-CN"),
                region_aliases: &[region!("SG")],
            },
        ];
        for (locale, expected) in [
            ("zh-Hant-CN", "zh-TW"),
            ("zh-Hans-TW", "zh-CN"),
            ("zh-TW", "zh-TW"),
            ("zh-HK", "zh-TW"),
            ("zh-SG", "zh-CN"),
            ("zh-CN", "zh-CN"),
            ("zh", "zh-TW"),
            ("zh-Latn-CN", "en"),
        ] {
            assert_eq!(select_locale([locale], &supported), expected, "{locale}");
        }
    }

    #[test]
    fn exact_identifiers_beat_earlier_generic_entries() {
        let supported = [
            SupportedLocale {
                name: "en",
                id: langid!("en"),
                region_aliases: &[],
            },
            SupportedLocale {
                name: "en-GB",
                id: langid!("en-GB"),
                region_aliases: &[],
            },
        ];
        assert_eq!(select_locale(["en-GB"], &supported), "en-GB");
        assert_eq!(select_locale(["en"], &supported), "en");
        assert_eq!(select_locale(["en-AU"], &supported), "en");
    }

    #[test]
    fn explicit_script_matches_beat_region_only_matches() {
        let supported = [
            SupportedLocale {
                name: "zh-TW",
                id: langid!("zh-TW"),
                region_aliases: &[],
            },
            SupportedLocale {
                name: "zh-CN",
                id: langid!("zh-Hans-CN"),
                region_aliases: &[],
            },
        ];
        assert_eq!(select_locale(["zh-Hans-TW"], &supported), "zh-CN");
    }

    #[test]
    fn default_and_catalog_names_remain_consistent() {
        assert!(
            SUPPORTED_LOCALES
                .iter()
                .any(|locale| locale.name == super::DEFAULT_LOCALE)
        );
        for (index, locale) in SUPPORTED_LOCALES.iter().enumerate() {
            let catalog = Locale::try_from_str(locale.name).unwrap();
            assert_eq!(catalog.id.language, locale.id.language);
            assert!(
                catalog
                    .id
                    .script
                    .is_none_or(|script| Some(script) == locale.id.script)
            );
            assert!(
                catalog
                    .id
                    .region
                    .is_none_or(|region| Some(region) == locale.id.region)
            );
            assert!(
                SUPPORTED_LOCALES[..index]
                    .iter()
                    .all(|previous| previous.name != locale.name)
            );
        }
    }
}
