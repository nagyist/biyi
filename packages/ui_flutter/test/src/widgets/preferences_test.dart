import 'package:beyondtranslate_ui/beyondtranslate_ui.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../host.dart' as app;

/// The root's cap, and the heading's action slot — the two parts of this
/// column that are a layout of their own rather than an arrangement of shared
/// spacing.
void main() {
  Widget host(Widget child) {
    return app.host(
      Align(
        alignment: Alignment.topCenter,
        child: SizedBox(width: 400, child: child),
      ),
    );
  }

  group('Preferences', () {
    testWidgets('fills a narrow pane and stops at its width in a wide one', (
      tester,
    ) async {
      const Widget column = Preferences(
        children: [
          PreferenceSection(
            children: [PreferenceRow(title: 'Launch at login')],
          ),
        ],
      );

      await tester.pumpWidget(host(column));
      expect(tester.getSize(find.byType(Preferences)).width, 400);

      await tester.pumpWidget(
        app.host(const Align(alignment: Alignment.topCenter, child: column)),
      );
      expect(
        tester.getSize(find.byType(Preferences)).width,
        ThemeData.studioLight().vars.preferencesWidth,
      );
    });
  });

  group('PreferenceSection', () {
    testWidgets('takes an action without the heading sizing to it', (
      tester,
    ) async {
      await tester.pumpWidget(
        host(
          const PreferenceSection(
            label: 'General',
            children: [PreferenceRow(title: 'Launch at login')],
          ),
        ),
      );
      final double plain = tester
          .getSize(find.byType(PreferenceSection))
          .height;

      int pressed = 0;
      await tester.pumpWidget(
        host(
          PreferenceSection(
            label: 'General',
            action: Button(
              size: WidgetSize.medium,
              onPressed: () => pressed++,
              child: const Text('Add'),
            ),
            children: const [PreferenceRow(title: 'Launch at login')],
          ),
        ),
      );

      expect(tester.takeException(), isNull);
      expect(tester.getSize(find.byType(PreferenceSection)).height, plain);

      // The control overhangs a slot with no height, so it is only reachable
      // if the slot hit tests past its own bounds.
      await tester.tap(find.text('Add'));
      expect(pressed, 1);
    });
  });

  group('PreferenceGroup', () {
    testWidgets('takes an action too', (tester) async {
      await tester.pumpWidget(
        host(
          PreferenceGroup(
            title: 'Appearance',
            action: Button(onPressed: () {}, child: const Text('Reset')),
            children: const [
              PreferenceSection(
                children: [PreferenceRow(title: 'Theme')],
              ),
            ],
          ),
        ),
      );

      expect(tester.takeException(), isNull);
      expect(find.text('Reset'), findsOneWidget);
    });
  });
}
