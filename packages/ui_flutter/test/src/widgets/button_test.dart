import 'package:beyondtranslate_ui/beyondtranslate_ui.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../host.dart';

void main() {
  testWidgets('Button', (WidgetTester tester) async {
    await tester.pumpWidget(
      host(
        Button(
          onPressed: () {},
          child: const Text('Button'),
        ),
      ),
    );
    expect(find.text('Button'), findsOneWidget);
  });
}
