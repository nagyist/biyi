use std::{
    collections::BTreeMap,
    env, fs,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{from_yaml_str, load_from_file, EngineConfig, EngineError, ProviderConfig};

#[test]
fn loads_configured_provider() {
    let registry = from_yaml_str(
        r#"
providers:
  deepl:
    type: deepl
    api_key: test-key
"#,
    )
    .expect("valid config");

    assert_eq!(registry.names(), vec!["deepl"]);
    let provider = registry.require("deepl").expect("deepl provider");
    assert_eq!(provider.name(), "deepl_api");
    assert!(registry.require("missing").is_err());
}

#[test]
fn loads_canonical_platform_provider_ids() {
    for (id, provider_type) in [
        ("yandex-cloud", "yandex_cloud"),
        ("microsoft-azure", "microsoft_azure"),
        ("alibaba-cloud", "alibaba_cloud"),
        ("volcengine", "volcengine"),
        ("niutrans", "niutrans"),
    ] {
        let fields = match provider_type {
            "yandex_cloud" | "microsoft_azure" | "niutrans" => "apiKey: test-key",
            "alibaba_cloud" => "accessKeyId: test-id\n    accessKeySecret: test-secret",
            "volcengine" => "accessKey: test-access\n    secretKey: test-secret",
            _ => unreachable!(),
        };
        let yaml = format!("providers:\n  {id}:\n    type: {provider_type}\n    {fields}\n");
        let registry = from_yaml_str(&yaml).expect("canonical provider id should load");
        assert!(registry.require(id).is_ok());
    }
}

#[test]
fn rejects_legacy_platform_provider_ids() {
    for provider_type in ["yandex", "microsoft_translator", "aliyun"] {
        let yaml =
            format!("providers:\n  legacy:\n    type: {provider_type}\n    apiKey: test-key\n");
        assert!(
            from_yaml_str(&yaml).is_err(),
            "legacy id {provider_type} must be rejected"
        );
    }
}

#[test]
fn loads_camel_case_provider_config() {
    let registry = from_yaml_str(
        r#"
providers:
  deepl:
    type: deepl
    appKey: test-key
"#,
    )
    .expect("valid config");

    assert_eq!(registry.names(), vec!["deepl"]);
}

#[cfg(feature = "baidu_fanyi_api")]
#[test]
fn loads_baidu_fanyi_api_provider() {
    let registry = from_yaml_str(
        r#"
providers:
  baidu-main:
    type: baidu
    appId: test-id
    appKey: test-key
"#,
    )
    .expect("valid config");

    assert_eq!(registry.names(), vec!["baidu-main"]);
    let provider = registry.require("baidu-main").expect("baidu provider");
    assert_eq!(provider.name(), "baidu_fanyi_api");
}

#[cfg(feature = "openai")]
#[test]
fn loads_openai_provider() {
    let registry = from_yaml_str(
        r#"
providers:
  openai-main:
    type: openai
    apiKey: sk-test-key
    defaultModel: configured-model
"#,
    )
    .expect("valid config");

    assert_eq!(registry.names(), vec!["openai-main"]);
    let provider = registry.require("openai-main").expect("openai provider");
    assert_eq!(provider.name(), "openai");
}

#[cfg(feature = "openai")]
#[test]
fn loads_openai_provider_snake_case() {
    let registry = from_yaml_str(
        r#"
providers:
  openai-main:
    type: openai
    api_key: sk-test-key
    default_model: configured-model
"#,
    )
    .expect("valid config");

    assert_eq!(registry.names(), vec!["openai-main"]);
}

#[cfg(feature = "openai")]
#[test]
fn openai_provider_settings_roundtrip() {
    // Simulate what settings.json stores after user configures via UI
    let settings_json = r#"{"type":"openai","apiKey":"sk-test","baseUrl":"https://api.openai.com","defaultModel":"configured-model"}"#;
    let config: ProviderConfig = serde_json::from_str(settings_json).expect("parse settings JSON");

    // Build engine config (same flow as engine::build_from_settings)
    let mut providers = BTreeMap::new();
    providers.insert("openai-main".to_string(), config);
    let engine_config = EngineConfig { providers };
    let yaml = serde_yaml::to_string(&engine_config).expect("serialize to yaml");

    // Parse back (same as from_yaml_str)
    let registry = from_yaml_str(&yaml).expect("parse yaml");
    assert_eq!(registry.names(), vec!["openai-main"]);
}

#[cfg(feature = "deepseek")]
#[test]
fn loads_deepseek_provider() {
    let registry = from_yaml_str(
        r#"
providers:
  deepseek-main:
    type: deepseek
    apiKey: sk-test-key
    defaultModel: deepseek-chat
"#,
    )
    .expect("valid config");

    assert_eq!(registry.names(), vec!["deepseek-main"]);
    let provider = registry
        .require("deepseek-main")
        .expect("deepseek provider");
    assert_eq!(provider.name(), "deepseek");
}

#[cfg(feature = "openai-compatible")]
#[test]
fn openai_compatible_provider_requires_base_url() {
    let error = from_yaml_str(
        r#"
providers:
  local-llm:
    type: openai_compatible
    defaultModel: qwen3:8b
"#,
    )
    .expect_err("missing base url should fail");

    assert!(matches!(error, EngineError::ConfigValidationFailed { .. }));

    let registry = from_yaml_str(
        r#"
providers:
  local-llm:
    type: openai_compatible
    baseUrl: http://localhost:1234
    defaultModel: qwen3:8b
"#,
    )
    .expect("valid config");
    let provider = registry.require("local-llm").expect("compat provider");
    assert_eq!(provider.name(), "openai_compatible");
}

#[test]
fn rejects_unknown_provider() {
    let error = from_yaml_str(
        r#"
providers:
  unknown:
    type: unknown
    api_key: test-key
"#,
    )
    .expect_err("unknown provider should fail");

    assert!(matches!(error, EngineError::ParseConfig(_)));
}

#[test]
fn rejects_invalid_provider_config() {
    let error = from_yaml_str(
        r#"
providers:
  deepl:
    type: deepl
    base_url: https://api.deepl.com
"#,
    )
    .expect_err("missing api_key should fail");

    assert!(matches!(
        error,
        EngineError::InvalidProviderConfig { provider, .. } if provider == "deepl"
    ));
}

#[test]
fn rejects_empty_api_key() {
    let error = from_yaml_str(
        r#"
providers:
  deepl:
    type: deepl
    api_key: ""
"#,
    )
    .expect_err("empty api_key should fail");

    assert!(matches!(
        error,
        EngineError::ConfigValidationFailed { ref provider, ref reason }
            if provider == "deepl" && reason == "api_key must not be empty"
    ));
}

#[test]
fn rejects_invalid_yaml() {
    let error = from_yaml_str("providers: [").expect_err("invalid yaml should fail");

    assert!(matches!(error, EngineError::ParseConfig(_)));
}

#[test]
fn reads_yaml_from_file() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("valid time")
        .as_nanos();
    let path = env::temp_dir().join(format!("beyondtranslate-engine-{suffix}.yaml"));
    fs::write(
        &path,
        r#"
providers:
  deepl:
    type: deepl
    api_key: file-key
"#,
    )
    .expect("write temp config");

    let registry = load_from_file(&path).expect("load config from file");
    fs::remove_file(&path).expect("remove temp config");

    assert_eq!(registry.names(), vec!["deepl"]);
}

#[test]
fn system_provider_advertises_dictionary() {
    let registry = from_yaml_str(
        r#"
providers:
  system:
    type: system
"#,
    )
    .expect("valid config");

    let provider = registry.require("system").expect("system provider");

    assert!(provider.translation().is_some());
    assert!(provider.ocr().is_some());
    assert!(provider.dictionary().is_some());
}

#[test]
fn cloud_providers_with_an_ocr_product_advertise_it() {
    let registry = from_yaml_str(
        r#"
providers:
  google:
    type: google_cloud
    api_key: key
  tencent:
    type: tencent_cloud
    secret_id: id
    secret_key: key
  volcengine:
    type: volcengine
    access_key: ak
    secret_key: sk
  alibaba:
    type: alibaba_cloud
    access_key_id: ak
    access_key_secret: sk
  yandex:
    type: yandex_cloud
    api_key: key
    folder_id: folder
"#,
    )
    .expect("valid config");

    for name in ["google", "tencent", "volcengine", "alibaba", "yandex"] {
        let provider = registry.require(name).expect(name);
        assert!(provider.translation().is_some(), "{name} translates");
        assert!(provider.ocr().is_some(), "{name} recognizes text");
        assert!(provider.dictionary().is_none(), "{name} has no dictionary");
    }
}

#[test]
fn ocr_base_url_is_optional_and_accepts_both_spellings() {
    let registry = from_yaml_str(
        r#"
providers:
  a:
    type: google_cloud
    api_key: key
    ocr_base_url: http://localhost:1
  b:
    type: google_cloud
    apiKey: key
    ocrBaseUrl: http://localhost:2
"#,
    )
    .expect("valid config");
    assert!(registry.require("a").is_ok());
    assert!(registry.require("b").is_ok());
}

#[cfg(not(feature = "baidu_fanyi_api"))]
#[test]
fn errors_when_feature_is_disabled() {
    let error = from_yaml_str(
        r#"
providers:
  baidu-main:
    type: baidu
    app_id: test-id
    app_key: test-key
"#,
    )
    .expect_err("disabled provider should fail");

    assert!(matches!(
        error,
        EngineError::ProviderNotEnabled(name) if name == "baidu-main"
    ));
}

#[test]
fn traditional_provider_ids_accept_legacy_names_and_write_canonical_names() {
    for (legacy, canonical) in [
        ("baidu", "baidu_fanyi_api"),
        ("caiyun", "caiyun_platform"),
        ("deepl", "deepl_api"),
        ("google", "google_cloud"),
        ("tencent", "tencent_cloud"),
        ("youdao", "youdao_zhiyun"),
    ] {
        let old: crate::ProviderType = serde_yaml::from_str(legacy).unwrap();
        let new: crate::ProviderType = serde_yaml::from_str(canonical).unwrap();
        assert_eq!(old, new);
        assert_eq!(old.as_str(), canonical);
        assert_eq!(serde_json::to_value(old).unwrap(), canonical);
    }
}
