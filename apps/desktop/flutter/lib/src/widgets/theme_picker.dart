import 'package:flutter/widgets.dart';

import '../theme/app_theme.dart' show DesignThemeFamily, designThemeFor;
import '../theme/product_tokens.dart' show ProductPalette;
import 'ui.dart'
    show Pressable, ThemeDataBuildContextProps, ThemeVariables, Tooltip;

/// The palettes, as the colours they paint with.
///
/// A name does not tell you what Frost or Ember look like and the choice is
/// entirely about the colour, so each family shows its own paper with its own
/// marker hue on it — Bright by its acid green rather than by the ink it fills
/// buttons with, which is the half of it you actually notice. The swatches
/// preview at the brightness the window is in, and picking one repaints
/// everything at once, so the row is its own preview.
class ThemeFamilyPicker extends StatelessWidget {
  const ThemeFamilyPicker({
    super.key,
    required this.value,
    required this.onChanged,
  });

  final DesignThemeFamily value;
  final ValueChanged<DesignThemeFamily> onChanged;

  @override
  Widget build(BuildContext context) {
    final ThemeVariables vars = context.vars;
    final Brightness brightness = context.themeData.brightness;

    return Row(
      mainAxisSize: MainAxisSize.min,
      spacing: vars.spacing15,
      children: [
        for (final family in DesignThemeFamily.values)
          _ThemeFamilySwatch(
            family: family,
            brightness: brightness,
            selected: family == value,
            onPressed: () => onChanged(family),
          ),
      ],
    );
  }
}

class _ThemeFamilySwatch extends StatelessWidget {
  const _ThemeFamilySwatch({
    required this.family,
    required this.brightness,
    required this.selected,
    required this.onPressed,
  });

  final DesignThemeFamily family;
  final Brightness brightness;
  final bool selected;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final ThemeVariables vars = context.vars;
    // The family's own tokens, not the ones in force — that is the point.
    final ThemeVariables theirs =
        designThemeFor(family.themeFor(brightness)).vars;

    return Tooltip(
      message: family.label,
      child: Pressable(
        onPressed: onPressed,
        semanticsLabel: family.label,
        checked: selected,
        borderRadius: BorderRadius.circular(vars.radiusFull),
        builder: (context, states) {
          final bool hovered = states.contains(WidgetState.hovered);
          // The ring sits outside the swatch with a gap, so it still reads on
          // the family whose own accent it is drawn in.
          return AnimatedContainer(
            duration: vars.motionDuration,
            curve: vars.motionEasing,
            padding: const EdgeInsets.all(2),
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              border: Border.all(
                color: selected
                    ? vars.accent
                    : (hovered ? vars.colorBorderStrong : vars.colorBorder),
                width: selected ? 2 : context.hairlineWidth,
              ),
            ),
            child: Container(
              width: 18,
              height: 18,
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: theirs.colorSurface,
                shape: BoxShape.circle,
                border: Border.all(
                  color: theirs.colorBorderStrong,
                  width: context.hairlineWidth,
                ),
              ),
              child: Container(
                width: 10,
                height: 10,
                decoration: BoxDecoration(
                  color: theirs.highlight,
                  shape: BoxShape.circle,
                ),
              ),
            ),
          );
        },
      ),
    );
  }
}
