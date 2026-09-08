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
import 'ui.dart' show MenuItem, MenuPanel, MenuRow;

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
    final List<ContextMenuButtonItem> items = state.contextMenuButtonItems;
    if (items.isEmpty) return const SizedBox.shrink();

    final Offset anchor = state.contextMenuAnchors.primaryAnchor;

    // The kit's own panel and rows, not a hand-drawn pair of them: a selection
    // menu is a menu, and this way it keeps the corner, the wash and the row
    // metrics every other menu in the app has — including under Bright, where
    // a panel drawn on `radiusSmall` rounds into a lozenge.
    return Stack(
      children: [
        Positioned(
          left: anchor.dx,
          top: anchor.dy,
          child: MenuPanel(
            children: [
              for (final item in items)
                MenuRow(
                  item: MenuItem(
                    label: _labelFor(item),
                    onSelect: item.onPressed,
                  ),
                ),
            ],
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

/// Puts a string on the clipboard without going through a selection.
Future<void> copyToClipboard(String text) =>
    Clipboard.setData(ClipboardData(text: text));
