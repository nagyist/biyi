use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
    sync::Arc,
};

use beyondtranslate_core::{
    DictionaryService, LlmService, OcrService, Provider, TranslationService,
};
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use thiserror::Error;

#[cfg(feature = "baidu_fanyi_api")]
use crate::provider::traditional::BaiduFanyiApiProvider;
use crate::provider::traditional::BaiduFanyiApiProviderConfig;
#[cfg(feature = "caiyun_platform")]
use crate::provider::traditional::CaiyunPlatformProvider;
use crate::provider::traditional::CaiyunPlatformProviderConfig;
use crate::provider::traditional::DeepLApiProvider;
use crate::provider::traditional::DeepLApiProviderConfig;
#[cfg(feature = "google_cloud")]
use crate::provider::traditional::GoogleCloudProvider;
use crate::provider::traditional::GoogleCloudProviderConfig;
#[cfg(feature = "anthropic")]
use crate::provider::AnthropicProvider;
use crate::provider::AnthropicProviderConfig;
#[cfg(feature = "ollama")]
use crate::provider::OllamaProvider;
use crate::provider::OllamaProviderConfig;
use crate::provider::OpenAiCompatibleProviderConfig;
#[allow(unused_imports)]
use crate::provider::{specs, OpenAiCompatibleProvider};

use crate::provider::traditional::SystemProvider;
#[cfg(feature = "tencent_cloud")]
use crate::provider::traditional::TencentCloudProvider;
use crate::provider::traditional::TencentCloudProviderConfig;
#[cfg(feature = "youdao_zhiyun")]
use crate::provider::traditional::YoudaoZhiyunProvider;
use crate::provider::traditional::YoudaoZhiyunProviderConfig;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("failed to read config file `{path}`: {source}")]
    ReadConfigFile {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config yaml: {0}")]
    ParseConfig(#[from] serde_yaml::Error),
    #[error("provider `{0}` is not supported")]
    UnknownProvider(String),
    #[error("provider `{0}` is not enabled in this build")]
    ProviderNotEnabled(String),
    #[error("provider `{provider}` config is invalid: {source}")]
    InvalidProviderConfig {
        provider: String,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("provider `{provider}` config validation failed: {reason}")]
    ConfigValidationFailed { provider: String, reason: String },
    #[error("provider `{0}` does not support translation")]
    TranslationNotSupported(String),
    #[error("provider `{0}` does not support dictionary lookup")]
    DictionaryNotSupported(String),
    #[error("provider `{0}` does not support ocr")]
    OcrNotSupported(String),
    #[error("provider `{0}` does not support llm")]
    LlmNotSupported(String),
}

// ── Registry ──────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct Engine {
    providers: HashMap<String, Arc<dyn Provider>>,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("names", &self.names())
            .finish()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the translation service for the named provider.
    pub fn translation(&self, name: &str) -> Result<&dyn TranslationService, EngineError> {
        self.require(name)?
            .translation()
            .ok_or_else(|| EngineError::TranslationNotSupported(name.to_owned()))
    }

    /// Returns the dictionary service for the named provider.
    pub fn dictionary(&self, name: &str) -> Result<&dyn DictionaryService, EngineError> {
        self.require(name)?
            .dictionary()
            .ok_or_else(|| EngineError::DictionaryNotSupported(name.to_owned()))
    }

    /// Returns the ocr service for the named provider.
    pub fn ocr(&self, name: &str) -> Result<&dyn OcrService, EngineError> {
        self.require(name)?
            .ocr()
            .ok_or_else(|| EngineError::OcrNotSupported(name.to_owned()))
    }

    /// Returns the llm service for the named provider.
    pub fn llm(&self, name: &str) -> Result<&dyn LlmService, EngineError> {
        self.require(name)?
            .llm()
            .ok_or_else(|| EngineError::LlmNotSupported(name.to_owned()))
    }

    /// Returns the raw provider by name. Prefer [`translation`] or [`dictionary`] for normal use.
    pub fn require(&self, name: &str) -> Result<&Arc<dyn Provider>, EngineError> {
        self.providers
            .get(name)
            .ok_or_else(|| EngineError::UnknownProvider(name.to_owned()))
    }

    /// Lists all registered provider names in alphabetical order.
    pub fn names(&self) -> Vec<&str> {
        let mut names = self
            .providers
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        names.sort_unstable();
        names
    }

    pub(crate) fn insert(&mut self, provider_id: String, provider: Arc<dyn Provider>) {
        self.providers.insert(provider_id, provider);
    }
}

// ── Builder ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProviderType {
    #[serde(rename = "baidu_fanyi_api", alias = "baidu")]
    BaiduFanyiApi,
    #[serde(rename = "caiyun_platform", alias = "caiyun")]
    CaiyunPlatform,
    #[serde(rename = "deepl_api", alias = "deepl")]
    DeepLApi,
    #[serde(rename = "google_cloud", alias = "google")]
    GoogleCloud,

    #[serde(rename = "tencent_cloud", alias = "tencent")]
    TencentCloud,
    #[serde(rename = "youdao_zhiyun", alias = "youdao")]
    YoudaoZhiyun,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "xai")]
    XAi,
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "qwen")]
    Qwen,
    #[serde(rename = "zhipu")]
    Zhipu,
    #[serde(rename = "moonshot")]
    Moonshot,
    #[serde(rename = "doubao")]
    Doubao,
    #[serde(rename = "groq")]
    Groq,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "openai_compatible")]
    OpenAiCompatible,
    #[serde(rename = "system")]
    System,
}

impl ProviderType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BaiduFanyiApi => "baidu_fanyi_api",
            Self::CaiyunPlatform => "caiyun_platform",
            Self::DeepLApi => "deepl_api",
            Self::GoogleCloud => "google_cloud",

            Self::TencentCloud => "tencent_cloud",
            Self::YoudaoZhiyun => "youdao_zhiyun",
            Self::Anthropic => "anthropic",
            Self::OpenAi => "openai",
            Self::Ollama => "ollama",
            Self::XAi => "xai",
            Self::DeepSeek => "deepseek",
            Self::Qwen => "qwen",
            Self::Zhipu => "zhipu",
            Self::Moonshot => "moonshot",
            Self::Doubao => "doubao",
            Self::Groq => "groq",
            Self::Gemini => "gemini",
            Self::OpenAiCompatible => "openai_compatible",
            Self::System => "system",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProviderConfig {
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    #[serde(flatten, default)]
    pub options: BTreeMap<String, Value>,
}

impl ProviderConfig {
    pub fn decode<T>(&self, provider_id: &str) -> Result<T, EngineError>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_yaml::from_value::<T>(self.options_value()).map_err(|source| {
            EngineError::InvalidProviderConfig {
                provider: provider_id.to_owned(),
                source,
            }
        })
    }

    pub fn options_value(&self) -> Value {
        let mut mapping = Mapping::new();
        for (key, value) in &self.options {
            mapping.insert(Value::String(key.clone()), value.clone());
        }
        Value::Mapping(mapping)
    }
}

macro_rules! build_provider_fn {
    ($fn_name:ident, $feature:literal, $Provider:ty, $Config:ty) => {
        #[cfg(feature = $feature)]
        fn $fn_name(provider_id: &str, config: $Config) -> Result<Arc<dyn Provider>, EngineError> {
            let provider =
                <$Provider>::new(config).map_err(|reason| EngineError::ConfigValidationFailed {
                    provider: provider_id.to_owned(),
                    reason,
                })?;
            Ok(Arc::new(provider))
        }

        #[cfg(not(feature = $feature))]
        fn $fn_name(provider_id: &str, _config: $Config) -> Result<Arc<dyn Provider>, EngineError> {
            Err(EngineError::ProviderNotEnabled(provider_id.to_owned()))
        }
    };
}

fn build_provider(
    provider_id: &str,
    config: ProviderConfig,
) -> Result<Arc<dyn Provider>, EngineError> {
    match config.provider_type {
        ProviderType::BaiduFanyiApi => {
            build_baidu_fanyi_api_provider(provider_id, config.decode(provider_id)?)
        }
        ProviderType::CaiyunPlatform => {
            build_caiyun_platform_provider(provider_id, config.decode(provider_id)?)
        }
        ProviderType::DeepLApi => {
            build_deepl_api_provider(provider_id, config.decode(provider_id)?)
        }
        ProviderType::GoogleCloud => {
            build_google_cloud_provider(provider_id, config.decode(provider_id)?)
        }

        ProviderType::TencentCloud => {
            build_tencent_cloud_provider(provider_id, config.decode(provider_id)?)
        }
        ProviderType::YoudaoZhiyun => {
            build_youdao_zhiyun_provider(provider_id, config.decode(provider_id)?)
        }
        ProviderType::Anthropic => {
            build_anthropic_provider(provider_id, config.decode(provider_id)?)
        }
        ProviderType::OpenAi => build_openai_provider(provider_id, config.decode(provider_id)?),
        ProviderType::Ollama => build_ollama_provider(provider_id, config.decode(provider_id)?),
        ProviderType::XAi => build_xai_provider(provider_id, config.decode(provider_id)?),
        ProviderType::DeepSeek => build_deepseek_provider(provider_id, config.decode(provider_id)?),
        ProviderType::Qwen => build_qwen_provider(provider_id, config.decode(provider_id)?),
        ProviderType::Zhipu => build_zhipu_provider(provider_id, config.decode(provider_id)?),
        ProviderType::Moonshot => build_moonshot_provider(provider_id, config.decode(provider_id)?),
        ProviderType::Doubao => build_doubao_provider(provider_id, config.decode(provider_id)?),
        ProviderType::Groq => build_groq_provider(provider_id, config.decode(provider_id)?),
        ProviderType::Gemini => build_gemini_provider(provider_id, config.decode(provider_id)?),
        ProviderType::OpenAiCompatible => {
            build_openai_compatible_provider(provider_id, config.decode(provider_id)?)
        }
        ProviderType::System => build_system_provider(provider_id),
    }
}

macro_rules! build_openai_compatible_provider_fn {
    ($fn_name:ident, $feature:literal, $spec:expr) => {
        #[cfg(feature = $feature)]
        fn $fn_name(
            provider_id: &str,
            config: OpenAiCompatibleProviderConfig,
        ) -> Result<Arc<dyn Provider>, EngineError> {
            let provider = OpenAiCompatibleProvider::new(&$spec, config).map_err(|reason| {
                EngineError::ConfigValidationFailed {
                    provider: provider_id.to_owned(),
                    reason,
                }
            })?;
            Ok(Arc::new(provider))
        }

        #[cfg(not(feature = $feature))]
        fn $fn_name(
            provider_id: &str,
            _config: OpenAiCompatibleProviderConfig,
        ) -> Result<Arc<dyn Provider>, EngineError> {
            Err(EngineError::ProviderNotEnabled(provider_id.to_owned()))
        }
    };
}

build_openai_compatible_provider_fn!(build_openai_provider, "openai", specs::OPENAI);
build_openai_compatible_provider_fn!(build_xai_provider, "xai", specs::XAI);
build_openai_compatible_provider_fn!(build_deepseek_provider, "deepseek", specs::DEEPSEEK);
build_openai_compatible_provider_fn!(build_qwen_provider, "qwen", specs::QWEN);
build_openai_compatible_provider_fn!(build_zhipu_provider, "zhipu", specs::ZHIPU);
build_openai_compatible_provider_fn!(build_moonshot_provider, "moonshot", specs::MOONSHOT);
build_openai_compatible_provider_fn!(build_doubao_provider, "doubao", specs::DOUBAO);
build_openai_compatible_provider_fn!(build_groq_provider, "groq", specs::GROQ);
build_openai_compatible_provider_fn!(build_gemini_provider, "gemini", specs::GEMINI);
build_openai_compatible_provider_fn!(
    build_openai_compatible_provider,
    "openai-compatible",
    specs::OPENAI_COMPATIBLE
);

build_provider_fn!(
    build_baidu_fanyi_api_provider,
    "baidu_fanyi_api",
    BaiduFanyiApiProvider,
    BaiduFanyiApiProviderConfig
);
build_provider_fn!(
    build_caiyun_platform_provider,
    "caiyun_platform",
    CaiyunPlatformProvider,
    CaiyunPlatformProviderConfig
);
build_provider_fn!(
    build_deepl_api_provider,
    "deepl_api",
    DeepLApiProvider,
    DeepLApiProviderConfig
);
build_provider_fn!(
    build_google_cloud_provider,
    "google_cloud",
    GoogleCloudProvider,
    GoogleCloudProviderConfig
);

build_provider_fn!(
    build_tencent_cloud_provider,
    "tencent_cloud",
    TencentCloudProvider,
    TencentCloudProviderConfig
);
build_provider_fn!(
    build_youdao_zhiyun_provider,
    "youdao_zhiyun",
    YoudaoZhiyunProvider,
    YoudaoZhiyunProviderConfig
);
fn build_system_provider(provider_id: &str) -> Result<Arc<dyn Provider>, EngineError> {
    let provider = SystemProvider::new().map_err(|reason| EngineError::ConfigValidationFailed {
        provider: provider_id.to_owned(),
        reason,
    })?;
    Ok(Arc::new(provider))
}

#[cfg(feature = "anthropic")]
fn build_anthropic_provider(
    provider_id: &str,
    config: AnthropicProviderConfig,
) -> Result<Arc<dyn Provider>, EngineError> {
    let provider =
        AnthropicProvider::new(config).map_err(|reason| EngineError::ConfigValidationFailed {
            provider: provider_id.to_owned(),
            reason,
        })?;
    Ok(Arc::new(provider))
}

#[cfg(not(feature = "anthropic"))]
fn build_anthropic_provider(
    provider_id: &str,
    _config: AnthropicProviderConfig,
) -> Result<Arc<dyn Provider>, EngineError> {
    Err(EngineError::ProviderNotEnabled(provider_id.to_owned()))
}

#[cfg(feature = "ollama")]
fn build_ollama_provider(
    provider_id: &str,
    config: OllamaProviderConfig,
) -> Result<Arc<dyn Provider>, EngineError> {
    let provider =
        OllamaProvider::new(config).map_err(|reason| EngineError::ConfigValidationFailed {
            provider: provider_id.to_owned(),
            reason,
        })?;
    Ok(Arc::new(provider))
}

#[cfg(not(feature = "ollama"))]
fn build_ollama_provider(
    provider_id: &str,
    _config: OllamaProviderConfig,
) -> Result<Arc<dyn Provider>, EngineError> {
    Err(EngineError::ProviderNotEnabled(provider_id.to_owned()))
}

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct EngineConfig {
    #[serde(default)]
    pub providers: BTreeMap<String, ProviderConfig>,
}

pub fn load_from_file(path: impl AsRef<Path>) -> Result<Engine, EngineError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|source| EngineError::ReadConfigFile {
        path: path.display().to_string(),
        source,
    })?;

    from_yaml_str(&content)
}

pub fn from_yaml_str(content: &str) -> Result<Engine, EngineError> {
    let config: EngineConfig = serde_yaml::from_str(content)?;
    from_config(config)
}

fn from_config(config: EngineConfig) -> Result<Engine, EngineError> {
    let mut registry = Engine::new();

    for (provider_id, config) in config.providers {
        let provider = build_provider(&provider_id, config)?;
        registry.insert(provider_id, provider);
    }

    Ok(registry)
}
