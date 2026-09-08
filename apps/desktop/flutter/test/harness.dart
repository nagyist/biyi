/// The shell a widget test mounts a specimen under.
///
/// `MaterialApp(home: Scaffold(body: …))` used to be the shorthand, and none of
/// what it carried was material: a `Directionality`, a `MediaQuery`, a
/// `Navigator` with the `Overlay` that anything floating needs, and a surface
/// to paint on. `WidgetsApp` carries the first three, and the palette comes
/// from [AppThemeProvider] the way it does in the app — which is the point of
/// pumping under a shell at all rather than under a bare `Directionality`.
library;

import 'package:beyondtranslate_desktop/src/theme/app_theme.dart'
    show AppThemeName, AppThemeProvider, designThemeFor;
import 'package:beyondtranslate_desktop/src/theme/product_tokens.dart'
    show ProductPalette;
import 'package:beyondtranslate_desktop/src/widgets/ui.dart'
    show ThemeDataBuildContextProps;
import 'package:flutter/widgets.dart';

/// Mounts [child] under the app's shell.
///
/// [size] boxes the specimen, standing in for the window a page would fill;
/// leave it null to let the child take the whole view.
Widget appHarness(
  Widget child, {
  Size? size,
  AppThemeName theme = AppThemeName.studioLight,
}) {
  return AppThemeProvider(
    theme: theme,
    child: WidgetsApp(
      // The colour the OS shows the app by. Nothing in a test looks at it, but
      // `WidgetsApp` asks for it outright.
      color: designThemeFor(theme).vars.accent,
      pageRouteBuilder: <T>(RouteSettings settings, WidgetBuilder builder) =>
          PageRouteBuilder<T>(
        settings: settings,
        pageBuilder: (context, _, __) => builder(context),
      ),
      home: Builder(
        builder: (context) => ColoredBox(
          color: context.vars.colorSurface,
          child: size == null
              ? child
              // Top-left, not centred: a test that measures where something
              // landed reads absolute view coordinates, and a centred box
              // shifts every one of them by half the slack.
              : Align(
                  alignment: Alignment.topLeft,
                  child: SizedBox(
                    width: size.width,
                    height: size.height,
                    child: child,
                  ),
                ),
        ),
      ),
    ),
  );
}
