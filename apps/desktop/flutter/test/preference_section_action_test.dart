// 设置 panes put a control on a section heading — 添加提供商, 添加服务, 添加目标.
//
// The kit's own slot for it could not lay out for a while: it sized itself to
// the constraints it was handed rather than to its child, and a Row hands a
// non-flexible child an unbounded main axis, so the heading took an infinite
// width and the pane threw before it painted. It is fixed upstream and the app
// no longer draws that line itself — this is what tells us if a later sync
// brings the crash back, since nothing else in the suite renders one.
import 'package:beyondtranslate_desktop/src/theme/app_theme.dart'
    show AppThemeProvider;
import 'package:beyondtranslate_desktop/src/widgets/ui.dart'
    show Button, PreferenceRow, PreferenceSection, SectionLabel;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  Future<void> pump(WidgetTester tester, Widget child) {
    return tester.pumpWidget(
      AppThemeProvider(
        child: Directionality(
          textDirection: TextDirection.ltr,
          child: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(width: 470, child: child),
          ),
        ),
      ),
    );
  }

  testWidgets('a section heading carries a control', (tester) async {
    await pump(
      tester,
      PreferenceSection(
        label: '可用服务',
        action: Button(onPressed: () {}, child: const Text('添加服务')),
        children: const [PreferenceRow(title: '一行')],
      ),
    );

    expect(tester.takeException(), isNull);
    expect(find.text('添加服务'), findsOneWidget);
  });

  testWidgets('the control overhangs the line rather than setting its height', (
    tester,
  ) async {
    Future<double> labelBottom(Widget section) async {
      await pump(tester, section);
      return tester.getBottomLeft(find.byType(SectionLabel)).dy;
    }

    // The control is taller than the label beside it, and a heading that grew
    // for it would start its rows lower than a heading without one — two
    // sections on the same page would then not line up.
    final withAction = await labelBottom(
      PreferenceSection(
        label: '可用服务',
        action: Button(onPressed: () {}, child: const Text('添加服务')),
        children: const [PreferenceRow(title: '一行')],
      ),
    );
    final without = await labelBottom(
      const PreferenceSection(
        label: '可用服务',
        children: [PreferenceRow(title: '一行')],
      ),
    );

    expect(withAction, without);
  });
}
