// The kit's ThemeData used to ride on Material's theme as an extension, so a
// widget found the window's palette wherever it stood. The kit left material
// behind, and its own `Theme` is now the only thing that carries the tokens —
// a subtree without one silently falls back to Studio Light rather than
// throwing. Every window wraps its router in an `AppThemeProvider` from
// `WidgetsApp.builder`, which is above the navigator; this is what says so,
// since a dialog reading the wrong palette looks like a colour bug, not a
// wiring one.
import 'package:beyondtranslate_desktop/src/theme/app_theme.dart';
import 'package:beyondtranslate_desktop/src/theme/product_tokens.dart'
    show ProductPalette, ProductTokens, ProductTokensContext;
import 'package:beyondtranslate_desktop/src/widgets/custom_alert_dialog/show_dialog.dart';
import 'package:beyondtranslate_desktop/src/widgets/ui.dart'
    show ThemeDataBuildContextProps;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import 'harness.dart';

void main() {
  // A palette that is nothing like the Studio Light fallback, so a miss is
  // unmistakable.
  const theme = AppThemeName.emberDark;

  Widget app(Widget home) => appHarness(home, theme: theme);

  testWidgets('a page reads the window palette', (tester) async {
    late Color seen;
    await tester.pumpWidget(app(Builder(builder: (context) {
      seen = context.vars.accent;
      return const SizedBox.shrink();
    })));

    expect(seen, designThemeFor(theme).vars.accent);
    expect(seen, isNot(designThemeFor(AppThemeName.studioLight).vars.accent));
  });

  testWidgets('so does a dialog in the overlay', (tester) async {
    late Color seen;
    late ProductTokens product;
    await tester.pumpWidget(app(Builder(builder: (context) {
      return GestureDetector(
        onTap: () => showDialogInCurrentWindow<void>(
          context: context,
          builder: (dialogContext) {
            seen = dialogContext.vars.accent;
            product = dialogContext.product;
            return const SizedBox.shrink();
          },
        ),
        child: const Text('open'),
      );
    })));

    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();

    expect(seen, designThemeFor(theme).vars.accent);
    // The product's own tokens ride on their own scope instead, and have to
    // reach the same place.
    expect(product, ProductTokens.forTheme(theme));
  });
}
