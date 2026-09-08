import 'package:flutter/widgets.dart';

/// Shows a dialog as an overlay within the **current window**, bypassing the
/// Flutter multi-window dialog-window behavior.
///
/// In Flutter 3.47+ (main channel), when the new multi-window API
/// (Flutter's multi-window API and [ViewCollection]) is used, the standard [showDialog]
/// function opens dialogs in a **separate native dialog window** instead of
/// rendering them as overlays over the current window. This is because
/// [showRawDialog] checks for a [WindowRegistry] in the context and, if found,
/// creates a [_DialogWindowRoute] — a new native window.
///
/// This function bypasses that behavior by pushing a [RawDialogRoute] directly
/// onto the current window's [Navigator], exactly as [showDialog] did before
/// the multi-window changes.
///
/// `RawDialogRoute` rather than material's `DialogRoute`: the two differ only
/// in the defaults material fills in — the scrim colour and the fade — and the
/// app names both itself, from its own tokens.
Future<T?> showDialogInCurrentWindow<T>({
  required BuildContext context,
  required WidgetBuilder builder,
  bool barrierDismissible = true,
  Color? barrierColor,
  String? barrierLabel,
  bool useSafeArea = true,
  RouteSettings? routeSettings,
  Offset? anchorPoint,
  TraversalEdgeBehavior? traversalEdgeBehavior,
  bool fullscreenDialog = false,
  bool? requestFocus,
}) {
  final NavigatorState navigator = Navigator.of(context, rootNavigator: true);

  // Capture inherited themes from the caller's context so that the dialog
  // (which renders inside the navigator's context) inherits the same theme,
  // text direction, and media query data.
  final CapturedThemes themes = InheritedTheme.capture(
    from: context,
    to: navigator.context,
  );

  return navigator.push<T>(
    RawDialogRoute<T>(
      pageBuilder: (context, animation, secondaryAnimation) {
        final Widget dialog = Builder(builder: builder);
        return themes.wrap(useSafeArea ? SafeArea(child: dialog) : dialog);
      },
      // The scrim material used to pick for us. Black at 40% is what it chose,
      // and it is the one colour here that is not a token: a scrim is the
      // absence of the window, not a shade of it.
      barrierColor: barrierColor ?? const Color(0x66000000),
      barrierDismissible: barrierDismissible,
      barrierLabel:
          barrierLabel ?? (barrierDismissible ? _kDismissLabel : null),
      transitionDuration: const Duration(milliseconds: 150),
      transitionBuilder: (context, animation, secondaryAnimation, child) =>
          FadeTransition(
        opacity: CurvedAnimation(parent: animation, curve: Curves.easeOut),
        child: child,
      ),
      settings: routeSettings,
      anchorPoint: anchorPoint,
      traversalEdgeBehavior:
          traversalEdgeBehavior ?? TraversalEdgeBehavior.closedLoop,
      requestFocus: requestFocus,
      fullscreenDialog: fullscreenDialog,
    ),
  );
}

/// What a screen reader calls the scrim. `MaterialLocalizations` supplied this;
/// with material gone the app says it, and there is one dialog language here
/// anyway — every sheet in the app is dismissed the same way.
const String _kDismissLabel = '关闭';
