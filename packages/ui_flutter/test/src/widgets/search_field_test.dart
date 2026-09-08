import 'package:beyondtranslate_ui/beyondtranslate_ui.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../host.dart' as app;

void main() {
  Widget host(Widget child) {
    return app.host(Center(child: SizedBox(width: 320, child: child)));
  }

  testWidgets('the query is the host\'s to hold', (tester) async {
    String query = 'ink';
    await tester.pumpWidget(
      host(
        StatefulBuilder(
          builder: (context, setState) => SearchField(
            value: query,
            onChanged: (next) => setState(() => query = next),
          ),
        ),
      ),
    );

    expect(find.text('ink'), findsOneWidget);

    await tester.enterText(find.byType(TextField), 'ink navy');
    await tester.pump();
    expect(query, 'ink navy');
  });

  testWidgets('the clear button takes the trailing slot from the hint', (
    tester,
  ) async {
    String query = '';
    await tester.pumpWidget(
      host(
        StatefulBuilder(
          builder: (context, setState) => SearchField(
            value: query,
            hint: const KeyCap('/'),
            onChanged: (next) => setState(() => query = next),
          ),
        ),
      ),
    );

    expect(find.byType(KeyCap), findsOneWidget);

    await tester.enterText(find.byType(TextField), 'ink');
    await tester.pump();
    expect(find.byType(KeyCap), findsNothing);

    await tester.tap(find.bySemanticsLabel('Clear search'));
    await tester.pump();
    expect(query, '');
    expect(find.byType(KeyCap), findsOneWidget);
  });

  testWidgets('escape clears first and dismisses only when empty', (
    tester,
  ) async {
    String query = '';
    int dismissed = 0;
    await tester.pumpWidget(
      host(
        StatefulBuilder(
          builder: (context, setState) => SearchField(
            value: query,
            autofocus: true,
            onChanged: (next) => setState(() => query = next),
            onDismiss: () => dismissed++,
          ),
        ),
      ),
    );
    await tester.pump();

    await tester.enterText(find.byType(TextField), 'ink');
    await tester.pump();

    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pump();
    expect(query, '');
    expect(dismissed, 0);

    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pump();
    expect(dismissed, 1);
  });
}
