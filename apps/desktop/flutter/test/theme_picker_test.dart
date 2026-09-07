// 主题风格 —— the palette is chosen apart from the light/dark pair, so the two
// have to stay independent: switching one must not disturb the other.
import 'package:beyondtranslate_desktop/src/theme/app_theme.dart';
import 'package:beyondtranslate_desktop/src/widgets/theme_picker.dart';
import 'package:flutter/material.dart' as m;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  Future<void> pump(
    WidgetTester tester,
    DesignThemeFamily value,
    ValueChanged<DesignThemeFamily> onChanged, {
    AppThemeName theme = AppThemeName.brightLight,
  }) {
    return tester.pumpWidget(
      m.MaterialApp(
        theme: appThemeData(theme),
        // A Tooltip needs an Overlay overhead, which the app's MaterialApp
        // gives it.
        home: AppThemeProvider(
          theme: theme,
          child: Center(
            child: ThemeFamilyPicker(value: value, onChanged: onChanged),
          ),
        ),
      ),
    );
  }

  testWidgets('offers every family, and names each one', (tester) async {
    await pump(tester, DesignThemeFamily.bright, (_) {});

    for (final family in DesignThemeFamily.values) {
      expect(
        find.bySemanticsLabel(family.label),
        findsOneWidget,
        reason: '${family.label} should be offered',
      );
    }
  });

  testWidgets('picking a swatch reports its family', (tester) async {
    final picked = <DesignThemeFamily>[];
    await pump(tester, DesignThemeFamily.bright, picked.add);

    await tester.tap(find.bySemanticsLabel(DesignThemeFamily.ember.label));
    await tester.pumpAndSettle();

    expect(picked, [DesignThemeFamily.ember]);
  });

  test('a family and a brightness pick exactly one palette', () {
    for (final family in DesignThemeFamily.values) {
      for (final brightness in Brightness.values) {
        final name = family.themeFor(brightness);
        expect(name.family, family);
        expect(name.brightness, brightness);
        // And the kit has a token set for it.
        expect(designThemeFor(name).brightness, brightness);
      }
    }
  });

  test('an id that is no longer a family falls back rather than throwing', () {
    expect(DesignThemeFamily.fromId('studio'), DesignThemeFamily.studio);
    expect(DesignThemeFamily.fromId('ember'), DesignThemeFamily.ember);
    expect(DesignThemeFamily.fromId('nope'), DesignThemeFamily.bright);
  });
}
