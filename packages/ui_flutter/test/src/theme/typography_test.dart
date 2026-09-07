import 'package:beyondtranslate_ui/beyondtranslate_ui.dart';
import 'package:flutter/material.dart' as material;
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

/// Whether a host can say what this package is set in.
///
/// The design asks for the platform's own UI face, which is `fontFamily:
/// null` in Flutter — right on a Mac, and nothing at all on Windows, in a
/// widget test, or in an app shipping a font of its own. So there are two
/// ways in, and both have to work: an ambient text style the components
/// merge with, and the face fields the type styles are built from.
void main() {
  Widget host(Widget child, {String? family, ThemeVariables? vars}) {
    Widget content = Center(child: child);
    if (family != null) {
      // Below the `Material`, which sets a text style of its own: this is
      // where a host's own ambient style lands.
      content = DefaultTextStyle(
        style: TextStyle(fontFamily: family),
        child: content,
      );
    }
    content = material.Material(
      type: material.MaterialType.transparency,
      child: content,
    );
    return material.MaterialApp(
      home: Theme(
        data: vars == null
            ? ThemeData.studioLight()
            : ThemeData.studioLight().copyWith(vars: vars),
        child: content,
      ),
    );
  }

  String? familyOf(WidgetTester tester, String text) {
    return tester
        .renderObject<RenderParagraph>(find.text(text))
        .text
        .style
        ?.fontFamily;
  }

  group('the host\'s face', () {
    testWidgets('reaches the labels this package sets a style on', (
      tester,
    ) async {
      await tester.pumpWidget(
        host(
          Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Badge(child: Text('Beta')),
              const Callout(message: Text('Two files are still open.')),
              Checkbox(
                value: true,
                onChanged: (_) {},
                label: const Text('Launch at login'),
              ),
            ],
          ),
          family: 'AmbientFace',
        ),
      );

      expect(familyOf(tester, 'Beta'), 'AmbientFace');
      expect(familyOf(tester, 'Two files are still open.'), 'AmbientFace');
      expect(familyOf(tester, 'Launch at login'), 'AmbientFace');
    });

    testWidgets('reaches a button label, which animates its style', (
      tester,
    ) async {
      await tester.pumpWidget(
        host(
          Button(onPressed: () {}, child: const Text('Save')),
          family: 'AmbientFace',
        ),
      );

      expect(familyOf(tester, 'Save'), 'AmbientFace');
    });
  });

  group('the type faces', () {
    test('are one field per face, and copyWith re-points them', () {
      const ThemeVariables vars = themeVariables;
      expect(vars.fontUi.family, isNull);
      expect(vars.fontUi.fallback, const ['SF Pro Text', 'PingFang SC']);
      expect(vars.bodyMedium.fontFamily, isNull);

      final ThemeVariables shipped = vars.copyWith(
        fontUi: const FontFace(
          family: 'HostSans',
          fallback: ['PingFang SC'],
        ),
      );

      expect(shipped.bodyMedium.fontFamily, 'HostSans');
      expect(shipped.labelSmall.fontFamily, 'HostSans');
      expect(shipped.bodyMedium.fontFamilyFallback, const ['PingFang SC']);
      // The display cut is its own face and stays where it was.
      expect(shipped.headlineSmall.fontFamily, isNull);
      // And nothing else moved.
      expect(shipped.bodyMedium.fontSize, vars.bodyMedium.fontSize);
      expect(shipped.colorSurface, vars.colorSurface);
    });

    testWidgets('a re-pointed face reaches the widgets', (tester) async {
      await tester.pumpWidget(
        host(
          const Badge(child: Text('Beta')),
          vars: themeVariables.copyWith(
            fontUi: const FontFace(family: 'HostSans'),
            fontDisplay: const FontFace(family: 'HostDisplay'),
          ),
        ),
      );

      expect(familyOf(tester, 'Beta'), 'HostDisplay');
    });
  });

  test('copyWith leaves everything it is not given alone', () {
    const ThemeVariables vars = themeVariables;
    final ThemeVariables moved = vars.copyWith(
      colorSurface: const Color(0xFF102030),
    );

    expect(moved.colorSurface, const Color(0xFF102030));
    expect(moved.colorCanvas, vars.colorCanvas);
    expect(moved.spacing4, vars.spacing4);
    expect(moved.controlMediumSize, vars.controlMediumSize);
    // A derived recipe follows the field it reads.
    expect(
      moved.controlColorNormalSurface.normalColor,
      const Color(0xFF102030),
    );
  });
}
