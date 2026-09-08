/// Text the reader can select and copy.
///
/// `SelectableText` and `SelectionArea` are both material's, and the toolbar
/// they raise is material's too. The selection machinery underneath them is
/// not: [SelectableRegion] is a widgets-layer widget, and it takes the menu as
/// a builder. So the app keeps the behaviour and draws the menu in the kit's
/// own paint — which is what the rest of the app's overlays look like anyway.
///
/// On macOS 译文 goes through `NativeText` and never reaches this; see
/// [TranslationText]. This is the other platforms' path, and the one an error
/// message anywhere takes.
library;

import 'package:flutter/services.dart' show Clipboard, ClipboardData;
import 'package:flutter/widgets.dart';

import '../i18n/i18n.dart';
import '../theme/product_tokens.dart' show ProductTypography;
import 'ui.dart' show Pressable, ThemeDataBuildContextProps, ThemeVariables;

class SelectableTextBlock extends StatefulWidget {
  const SelectableTextBlock(
    this.data, {
    super.key,
    this.style,
    this.textAlign,
    this.maxLines,
  });

  final String data;
  final TextStyle? style;
  final TextAlign? textAlign;
  final int? maxLines;

  @override
  State<SelectableTextBlock> createState() => _SelectableTextBlockState();
}

class _SelectableTextBlockState extends State<SelectableTextBlock> {
  final FocusNode _focusNode = FocusNode();

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return SelectableRegion(
      focusNode: _focusNode,
      // The handles are for a touchscreen; on a desktop the pointer is the
      // handle. The menu below is what a reader actually reaches for.
      selectionControls: emptyTextSelectionControls,
      contextMenuBuilder: (context, state) => _CopyMenu(state: state),
      child: Text(
        widget.data,
        style: widget.style,
        textAlign: widget.textAlign,
        maxLines: widget.maxLines,
      ),
    );
  }
}

/// The one thing a selection is for here.
class _CopyMenu extends StatelessWidget {
  const _CopyMenu({required this.state});

  final SelectableRegionState state;

  @override
  Widget build(BuildContext context) {
    final ThemeVariables vars = context.vars;
    final List<ContextMenuButtonItem> items = state.contextMenuButtonItems;
    if (items.isEmpty) return const SizedBox.shrink();

    final Offset anchor = state.contextMenuAnchors.primaryAnchor;

    return Stack(
      children: [
        Positioned(
          left: anchor.dx,
          top: anchor.dy,
          child: Container(
            padding: EdgeInsets.all(vars.spacing05),
            decoration: BoxDecoration(
              color: vars.colorSurfaceRaised,
              borderRadius: BorderRadius.circular(vars.radiusSmall),
              border: Border.all(
                color: vars.colorBorderStrong,
                width: context.hairlineWidth,
              ),
              boxShadow: vars.shadowMd,
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                for (final item in items)
                  _MenuRow(label: _labelFor(item), onPressed: item.onPressed),
              ],
            ),
          ),
        ),
      ],
    );
  }

  /// `MaterialLocalizations` named these; the app names the two it can raise.
  String _labelFor(ContextMenuButtonItem item) => switch (item.type) {
        ContextMenuButtonType.copy => t.common.ui.button.copy,
        ContextMenuButtonType.selectAll => t.common.ui.button.select_all,
        _ => item.label ?? '',
      };
}

class _MenuRow extends StatelessWidget {
  const _MenuRow({required this.label, required this.onPressed});

  final String label;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    final ThemeVariables vars = context.vars;
    final BorderRadius radius = BorderRadius.circular(vars.radiusTiny);

    return Pressable(
      onPressed: onPressed,
      borderRadius: radius,
      builder: (context, states) => Container(
        padding: EdgeInsets.symmetric(
          horizontal: vars.spacing25,
          vertical: vars.spacing1,
        ),
        decoration: BoxDecoration(
          color: states.contains(WidgetState.hovered)
              ? vars.colorSurfaceMuted
              : null,
          borderRadius: radius,
        ),
        child: Text(
          label,
          style: vars.sansStyle(fontSize: 12, color: vars.colorContent),
        ),
      ),
    );
  }
}

/// Puts a string on the clipboard without going through a selection.
Future<void> copyToClipboard(String text) =>
    Clipboard.setData(ClipboardData(text: text));
