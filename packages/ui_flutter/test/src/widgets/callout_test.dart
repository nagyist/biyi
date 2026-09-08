import 'package:beyondtranslate_ui/beyondtranslate_ui.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../host.dart';

void main() {
  Widget staged(Widget child, {double? height}) {
    return host(
      Align(
        alignment: Alignment.topCenter,
        // A bounded height, passed down loose, is the case that used to go
        // wrong — a banner in a window's column rather than a callout in a
        // scroll view, where the height is unbounded and the bug could not
        // show.
        child: SizedBox(
          width: 400,
          height: height,
          child: Align(alignment: Alignment.topCenter, child: child),
        ),
      ),
    );
  }

  testWidgets('Callout is as tall as its text, not as tall as it is let', (
    tester,
  ) async {
    const Widget callout = Callout(
      tint: CalloutTint.warning,
      message: Text('Two files are still open.'),
    );

    await tester.pumpWidget(staged(callout));
    final double intrinsic = tester.getSize(find.byType(Callout)).height;

    await tester.pumpWidget(staged(callout, height: 900));

    expect(tester.getSize(find.byType(Callout)).height, intrinsic);
    expect(intrinsic, lessThan(100));
  });
}
