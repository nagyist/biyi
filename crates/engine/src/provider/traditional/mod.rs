//! Traditional (non-LLM) translation providers.

pub mod baidu_fanyi_api;
pub mod caiyun_platform;
pub mod deepl_api;
pub mod google_cloud;
pub mod system;
pub mod tencent_cloud;
pub mod youdao_zhiyun;

#[cfg(feature = "baidu_fanyi_api")]
pub use baidu_fanyi_api::BaiduFanyiApiProvider;
pub use baidu_fanyi_api::BaiduFanyiApiProviderConfig;
#[cfg(feature = "caiyun_platform")]
pub use caiyun_platform::CaiyunPlatformProvider;
pub use caiyun_platform::CaiyunPlatformProviderConfig;
pub use deepl_api::{DeepLApiProvider, DeepLApiProviderConfig};
#[cfg(feature = "google_cloud")]
pub use google_cloud::GoogleCloudProvider;
pub use google_cloud::GoogleCloudProviderConfig;
pub use system::SystemProvider;
pub use system::SystemTranslationService;
#[cfg(feature = "tencent_cloud")]
pub use tencent_cloud::TencentCloudProvider;
pub use tencent_cloud::TencentCloudProviderConfig;
#[cfg(feature = "youdao_zhiyun")]
pub use youdao_zhiyun::YoudaoZhiyunProvider;
pub use youdao_zhiyun::YoudaoZhiyunProviderConfig;
