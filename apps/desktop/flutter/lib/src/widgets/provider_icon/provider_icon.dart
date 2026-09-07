import 'package:flutter/material.dart' show Icons;
import 'package:flutter/widgets.dart';

import '../../services/runtime.dart' show ProviderType;
import '../../utils/r.dart';
import '../ui.dart' show ThemeDataBuildContextProps;

/// The identity mark for each supported provider type.
class ProviderIcon extends StatelessWidget {
  const ProviderIcon(
    this.type, {
    super.key,
    this.size = 22,
    this.color,
    this.border,
  });

  final ProviderType type;
  final double size;
  final Color? color;
  final Border? border;

  @override
  Widget build(BuildContext context) {
    return switch (type) {
      ProviderType.system =>
        Icon(Icons.computer_outlined, size: size, color: color),
      ProviderType.openAiCompatible =>
        Icon(Icons.api_outlined, size: size, color: color),
      ProviderType.anthropic =>
        _buildAsset(context, 'provider_icons/llm/anthropic.png'),
      ProviderType.baiduFanyiApi =>
        _buildAsset(context, 'provider_icons/traditional/baidu_fanyi_api.png'),
      ProviderType.caiyunPlatform =>
        _buildAsset(context, 'provider_icons/traditional/caiyun_platform.png'),
      ProviderType.deepLApi =>
        _buildAsset(context, 'provider_icons/traditional/deepl_api.png'),
      ProviderType.deepSeek =>
        _buildAsset(context, 'provider_icons/llm/deepseek.png'),
      ProviderType.doubao =>
        _buildAsset(context, 'provider_icons/llm/doubao.png'),
      ProviderType.gemini =>
        _buildAsset(context, 'provider_icons/llm/gemini.png'),
      ProviderType.googleCloud =>
        _buildAsset(context, 'provider_icons/traditional/google_cloud.png'),
      ProviderType.groq => _buildAsset(context, 'provider_icons/llm/groq.png'),
      ProviderType.moonshot =>
        _buildAsset(context, 'provider_icons/llm/moonshot.png'),
      ProviderType.ollama =>
        _buildAsset(context, 'provider_icons/llm/ollama.png'),
      ProviderType.openAi =>
        _buildAsset(context, 'provider_icons/llm/openai.png'),
      ProviderType.qwen => _buildAsset(context, 'provider_icons/llm/qwen.png'),
      ProviderType.tencentCloud =>
        _buildAsset(context, 'provider_icons/traditional/tencent_cloud.png'),
      ProviderType.xAi => _buildAsset(context, 'provider_icons/llm/xai.png'),
      ProviderType.youdaoZhiyun =>
        _buildAsset(context, 'provider_icons/traditional/youdao_zhiyun.png'),
      ProviderType.zhipu =>
        _buildAsset(context, 'provider_icons/llm/zhipu.png'),
    };
  }

  Widget _buildAsset(BuildContext context, String asset) {
    final vars = context.vars;
    final radius = BorderRadius.circular(vars.radiusSmall);
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        image: DecorationImage(
          image: AssetImage(R.image(asset)),
          fit: BoxFit.cover,
          colorFilter:
              color != null ? ColorFilter.mode(color!, BlendMode.color) : null,
        ),
        borderRadius: radius,
        border: border ??
            Border.all(
              color: vars.colorBorder,
              width: context.hairlineWidth,
            ),
      ),
    );
  }
}
