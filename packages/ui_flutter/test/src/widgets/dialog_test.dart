import 'package:beyondtranslate_ui/beyondtranslate_ui.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../host.dart';

void main() {
  Widget sheet() {
    return Dialog(
      children: [
        const DialogHeader(title: 'Move 200 files'),
        DialogBody(
          children: [
            for (int i = 0; i < 60; i++) Text('Line $i'),
          ],
        ),
        DialogFooter(
          children: [Button(onPressed: () {}, child: const Text('Move'))],
        ),
      ],
    );
  }

  testWidgets('the sheet stops at the window even with nothing to bound it', (
    tester,
  ) async {
    // A scroll view hands its child an unbounded height, which is where the
    // sheet used to grow past the viewport and the body's own scroller never
    // took over.
    await tester.pumpWidget(
      host(SingleChildScrollView(child: Center(child: sheet()))),
    );

    final double window =
        tester.view.physicalSize.height / tester.view.devicePixelRatio;
    expect(tester.getSize(find.byType(Dialog)).height, lessThan(window));
  });

  testWidgets('the body scrolls while the header and footer stay put', (
    tester,
  ) async {
    await tester.pumpWidget(host(DialogScrim(child: sheet())));

    expect(find.text('Move 200 files'), findsOneWidget);
    expect(find.text('Move'), findsOneWidget);
    expect(find.byType(Scrollable), findsWidgets);
  });
}
