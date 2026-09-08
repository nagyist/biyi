import 'package:beyondtranslate_ui/beyondtranslate_ui.dart';
import 'package:flutter/cupertino.dart' show DefaultCupertinoLocalizations;

/// The plumbing a test's widget is pumped into, with material left out of it.
///
/// `MaterialApp` used to stand here for the four things it brought along —
/// a `Directionality`, a `MediaQuery`, a `Navigator` for the overlay a dialog
/// or a menu is pushed into, and the localizations the text field reads its
/// button labels out of. None of those are material's; they are `WidgetsApp`'s,
/// which is what this is. The cupertino delegate is the one addition: the text
/// field this kit draws is a cupertino one underneath, and it asks
/// `CupertinoLocalizations` for the word on its clear button.
///
/// The transparent `Material` that used to sit under it is gone with no
/// replacement here: the ambient text style it carried is the [Theme]'s own
/// now, which is where a test wants it anyway.
Widget host(Widget child, {ThemeData? theme}) {
  final ThemeData data = theme ?? ThemeData.studioLight();

  return WidgetsApp(
    color: data.vars.colorCanvas,
    debugShowCheckedModeBanner: false,
    localizationsDelegates: const [DefaultCupertinoLocalizations.delegate],
    pageRouteBuilder: <T>(RouteSettings settings, WidgetBuilder builder) =>
        PageRouteBuilder<T>(
          settings: settings,
          pageBuilder: (context, _, _) => builder(context),
        ),
    home: Theme(data: data, child: child),
  );
}
