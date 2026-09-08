import 'package:flutter/widgets.dart' hide FormField;
import 'package:go_router/go_router.dart';

import '../../services/runtime.dart';
import '../../theme/product_tokens.dart' show ProductPalette, ProductTypography;
import '../../utils/platform_util.dart';
import '../../widgets/selectable_text.dart';
import '../../widgets/ui.dart'
    show
        Button,
        ButtonVariant,
        FormField,
        SegmentedControl,
        SegmentedItem,
        Spinner,
        TextField,
        ThemeDataBuildContextProps,
        ThemeVariables,
        WidgetSize;
import 'debug_scaffold.dart';

List<RouteBase> get $appRoutes => <RouteBase>[
      GoRoute(
        path: '/debug/runtime',
        builder: (BuildContext context, GoRouterState state) {
          return const RuntimeDebugRoutePage();
        },
      ),
    ];

class RuntimeDebugRoutePage extends StatelessWidget {
  const RuntimeDebugRoutePage({super.key});

  @override
  Widget build(BuildContext context) {
    return const DebugScaffold(
      title: 'Runtime Debug',
      body: RuntimeDebugPage(),
    );
  }
}

class RuntimeDebugPage extends StatefulWidget {
  const RuntimeDebugPage({super.key});

  @override
  State<RuntimeDebugPage> createState() => _RuntimeDebugPageState();
}

class _RuntimeDebugPageState extends State<RuntimeDebugPage> {
  final _formKey = GlobalKey<FormState>();
  final _sourceLanguageController = TextEditingController(text: 'en');
  final _targetLanguageController = TextEditingController(text: 'zh');
  final _textController = TextEditingController(text: 'Hello');

  List<ProviderConfigEntry> _providers = const [];
  String? _providerId;
  bool _loadingProviders = true;
  bool _submitting = false;
  TranslateResponse? _response;
  String? _errorText;

  @override
  void initState() {
    super.initState();
    _loadProviders();
  }

  @override
  void dispose() {
    _sourceLanguageController.dispose();
    _targetLanguageController.dispose();
    _textController.dispose();
    super.dispose();
  }

  Future<void> _loadProviders() async {
    try {
      final providers = await runtime.settings().listProviders();
      if (!mounted) {
        return;
      }
      setState(() {
        _providers = providers;
        _providerId = providers.isEmpty ? null : providers.first.id;
        _loadingProviders = false;
      });
    } catch (error) {
      if (!mounted) {
        return;
      }
      setState(() {
        _providers = const [];
        _providerId = null;
        _loadingProviders = false;
        _errorText = error.toString();
      });
    }
  }

  Future<void> _submit() async {
    final formState = _formKey.currentState;
    final providerId = _providerId;
    if (providerId == null || formState == null || !formState.validate()) {
      return;
    }

    setState(() {
      _submitting = true;
      _errorText = null;
      _response = null;
    });

    try {
      final response =
          await runtime.translation(providerId: providerId).translate(
                request: TranslateRequest(
                  sourceLanguage: _sourceLanguageController.text.trim().isEmpty
                      ? null
                      : _sourceLanguageController.text.trim(),
                  targetLanguage: _targetLanguageController.text.trim(),
                  text: _textController.text,
                ),
              );

      if (!mounted) {
        return;
      }
      setState(() {
        _response = response;
      });
    } catch (error) {
      if (!mounted) {
        return;
      }
      setState(() {
        _response = null;
        _errorText = error.toString();
      });
    } finally {
      if (mounted) {
        setState(() {
          _submitting = false;
        });
      }
    }
  }

  Widget _buildProviderPicker(BuildContext context) {
    if (_loadingProviders) {
      return const Center(child: Spinner());
    }
    if (_providers.isEmpty) {
      return Text(
        'No configured providers found. Save a provider in settings first.',
        style: _body(context),
      );
    }

    return SegmentedControl<String>(
      items: _providers
          .map(
            (provider) => SegmentedItem<String>(
              value: provider.id,
              label: '${provider.id} (${provider.type})',
            ),
          )
          .toList(),
      value: _providerId ?? _providers.first.id,
      onChanged: (value) => setState(() => _providerId = value),
    );
  }

  Widget _buildResultCard(BuildContext context) {
    final ThemeVariables vars = context.vars;
    final hasResult = _response != null;
    final isError = _errorText != null;

    final title = isError
        ? 'Rust runtime error'
        : hasResult
            ? 'Rust runtime response'
            : 'Result';
    final content = isError
        ? _errorText!
        : _response != null
            ? _formatResponse(_response!)
            : 'Submit a request to see the Rust runtime response here.';

    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: vars.colorSurfaceMuted,
        borderRadius: BorderRadius.circular(vars.radiusMedium),
        border: Border.all(
          color: isError ? vars.dangerFg : vars.colorBorder,
          width: context.hairlineWidth,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(title, style: _heading(context)),
          const SizedBox(height: 12),
          SelectableTextBlock(
            content,
            style: vars.monoStyle(fontSize: 12, height: 1.4),
          ),
        ],
      ),
    );
  }

  String _formatResponse(TranslateResponse response) {
    final buffer = StringBuffer()
      ..writeln('provider: ${_providerId ?? 'null'}')
      ..writeln('translations:');

    for (final translation in response.translations) {
      buffer.writeln('- ${translation.text}');
      buffer.writeln(
        '  detected_source_language: ${translation.detectedSourceLanguage ?? 'null'}',
      );
      buffer.writeln('  audio_url: ${translation.audioUrl ?? 'null'}');
    }

    return buffer.toString().trimRight();
  }

  @override
  Widget build(BuildContext context) {
    return Form(
      key: _formKey,
      child: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          Text(
            'Call the Rust runtime directly from the desktop app to verify the integration.',
            style: _body(context),
          ),
          if (!kIsMacOS) ...[
            const SizedBox(height: 12),
            Text(
              'This debug page currently targets macOS. Other desktop platforms may compile, but they are outside this verification scope.',
              style: _body(context, fontSize: 11),
            ),
          ],
          const SizedBox(height: 20),
          Text('Provider', style: _heading(context)),
          const SizedBox(height: 12),
          _buildProviderPicker(context),
          const SizedBox(height: 20),
          Text('Request', style: _heading(context)),
          const SizedBox(height: 12),
          FormField(
              label: 'Source language (optional)',
              child: TextField(controller: _sourceLanguageController)),
          const SizedBox(height: 14),
          FormField(
              label: 'Target language',
              child: TextField(controller: _targetLanguageController)),
          const SizedBox(height: 14),
          FormField(
              label: 'Text',
              child: TextField(
                  controller: _textController, minLines: 4, maxLines: 8)),
          const SizedBox(height: 20),
          Row(
            children: [
              Button(
                  variant: ButtonVariant.filled,
                  onPressed:
                      _submitting || _loadingProviders || _providerId == null
                          ? null
                          : _submit,
                  child: _submitting
                      ? const Spinner(size: WidgetSize.small, onAccent: true)
                      : const Text('Translate with Rust runtime')),
            ],
          ),
          const SizedBox(height: 24),
          _buildResultCard(context),
        ],
      ),
    );
  }
}

/// A section heading on a debug page.
TextStyle _heading(BuildContext context) => context.vars.sansStyle(
      fontSize: 14,
      fontWeight: FontWeight.w600,
      color: context.vars.colorContent,
    );

TextStyle _body(BuildContext context, {double fontSize = 12}) =>
    context.vars.sansStyle(
      fontSize: fontSize,
      height: 1.5,
      color: context.vars.colorContentSecondary,
    );
