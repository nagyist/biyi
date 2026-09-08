import 'package:flutter/widgets.dart';

import '../widgets/ui.dart' as ui;
import 'product_tokens.dart'
    show ProductFonts, ProductTokens, ProductTypography;

/// The palette family the design system paints with.
///
/// Each family carries its own light and dark pair, so this is orthogonal to
/// [Brightness]: the family picks the character, the brightness picks the pair.
/// The two are chosen separately in 设置 › 外观 and stored separately, so a
/// theme survives a switch to 跟随系统 and back.
enum DesignThemeFamily {
  /// Muted violet on near-white / near-black — the kit's baseline.
  studio('studio', 'Studio'),

  /// Warm paper and ink navy, marked in acid green. The default.
  bright('bright', 'Bright'),

  /// Cool teal on a blue-grey ground.
  frost('frost', 'Frost'),

  /// Neutral greys, no hue at all beyond the state colours.
  graphite('graphite', 'Graphite'),

  /// Warm rust on a sand ground.
  ember('ember', 'Ember');

  const DesignThemeFamily(this.id, this.label);

  /// The value persisted in `appearance.theme`.
  final String id;

  /// What 设置 shows. A proper noun, so it is not translated — the palettes
  /// are named the same in every locale, the way a typeface is.
  final String label;

  static DesignThemeFamily fromId(String id) => DesignThemeFamily.values
      .firstWhere((family) => family.id == id, orElse: () => bright);

  /// This family's palette at a brightness.
  AppThemeName themeFor(Brightness brightness) =>
      AppThemeName.values.firstWhere(
        (name) => name.family == this && name.brightness == brightness,
      );
}

/// One of the kit's ten palettes: a family under a brightness.
///
/// The kit hands out its themes as constructors rather than as an enum, but
/// the product layer has tokens that vary by palette (see [ProductTokens]),
/// so the app keeps the name around and carries it on the theme it publishes.
enum AppThemeName {
  studioLight(DesignThemeFamily.studio, Brightness.light),
  studioDark(DesignThemeFamily.studio, Brightness.dark),
  brightLight(DesignThemeFamily.bright, Brightness.light),
  brightDark(DesignThemeFamily.bright, Brightness.dark),
  frostLight(DesignThemeFamily.frost, Brightness.light),
  frostDark(DesignThemeFamily.frost, Brightness.dark),
  graphiteLight(DesignThemeFamily.graphite, Brightness.light),
  graphiteDark(DesignThemeFamily.graphite, Brightness.dark),
  emberLight(DesignThemeFamily.ember, Brightness.light),
  emberDark(DesignThemeFamily.ember, Brightness.dark);

  const AppThemeName(this.family, this.brightness);

  final DesignThemeFamily family;
  final Brightness brightness;

  bool get isDark => brightness == Brightness.dark;
}

/// The kit's token set for a palette, in the app's own faces.
///
/// Everything visual comes from `beyondtranslate_ui`; this names which of its
/// themes a family and brightness map to, and re-points the two type faces it
/// carries. The kit names Apple faces and leaves the family slot empty, which
/// is right on a Mac and resolves to whatever the engine defaults to
/// everywhere else — Segoe UI with an Apple fallback list behind it on
/// Windows, so a kit label falls back to a Chinese face that is not installed.
/// Pointing `fontUi` and `fontDisplay` at the app's own stacks puts the kit's
/// widgets in the same type as the product's, on every platform.
ui.ThemeData designThemeFor(AppThemeName name) {
  final ui.ThemeData theme = _paletteFor(name);
  return theme.copyWith(
    vars: theme.vars.copyWith(
      fontUi: ProductFonts.ui,
      fontDisplay: ProductFonts.display,
    ),
  );
}

ui.ThemeData _paletteFor(AppThemeName name) => switch (name) {
      AppThemeName.studioLight => ui.ThemeData.studioLight(),
      AppThemeName.studioDark => ui.ThemeData.studioDark(),
      AppThemeName.brightLight => ui.ThemeData.brightLight(),
      AppThemeName.brightDark => ui.ThemeData.brightDark(),
      AppThemeName.frostLight => ui.ThemeData.frostLight(),
      AppThemeName.frostDark => ui.ThemeData.frostDark(),
      AppThemeName.graphiteLight => ui.ThemeData.graphiteLight(),
      AppThemeName.graphiteDark => ui.ThemeData.graphiteDark(),
      AppThemeName.emberLight => ui.ThemeData.emberLight(),
      AppThemeName.emberDark => ui.ThemeData.emberDark(),
    };

/// Scopes a palette to a subtree and establishes the root defaults below it:
/// the body face, the primary foreground colour, and the product's own tokens.
///
/// This is the only thing that carries them. The kit left material behind, so
/// its [ui.ThemeData] is not something a Material theme can hold — and the app
/// has no Material theme to hold it in any case. A subtree with no
/// [AppThemeProvider] over it falls back to Studio Light rather than to
/// whatever the window is set to, so every window wraps its router in one,
/// above the navigator, which is what puts a dialog or a menu in the overlay
/// inside it too.
class AppThemeProvider extends StatelessWidget {
  const AppThemeProvider({
    super.key,
    this.theme = AppThemeName.studioLight,
    this.data,
    required this.child,
  });

  /// The palette to scope, ignored when [data] is given.
  final AppThemeName theme;

  /// A token set to scope directly, for a subtree that varies from its parent.
  final ui.ThemeData? data;

  final Widget child;

  @override
  Widget build(BuildContext context) {
    final ui.ThemeData resolved = data ?? designThemeFor(theme);
    final ui.ThemeVariables vars = resolved.vars;

    return ui.Theme(
      data: resolved,
      child: ProductScope(
        tokens:
            ProductTokens.forTheme(data == null ? theme : _nameOf(resolved)),
        child: DefaultTextStyle(
          style: vars.sansStyle(color: vars.colorContent),
          child: IconTheme(
            data: IconThemeData(color: vars.colorContent),
            child: child,
          ),
        ),
      ),
    );
  }
}

/// Which palette a token set came from.
///
/// A subtree given a [ui.ThemeData] directly still needs the product tokens
/// that go with it, and the only thing the kit's theme carries is its
/// variables — so the name is recovered by matching them.
AppThemeName _nameOf(ui.ThemeData data) => AppThemeName.values.firstWhere(
      (name) => designThemeFor(name).vars == data.vars,
      orElse: () => AppThemeName.studioLight,
    );

/// Carries [ProductTokens] down the tree.
///
/// They used to ride on Material's theme as an extension. With material gone
/// they need a scope of their own, which [AppThemeProvider] installs beside
/// the kit's.
class ProductScope extends InheritedWidget {
  const ProductScope({super.key, required this.tokens, required super.child});

  final ProductTokens tokens;

  static ProductTokens of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<ProductScope>()?.tokens ??
      const ProductTokens();

  @override
  bool updateShouldNotify(ProductScope oldWidget) => tokens != oldWidget.tokens;
}

/// Light, dark, or whatever the OS is set to.
///
/// `MaterialApp` used to resolve this from its `themeMode`; with the shell on
/// `WidgetsApp` the app resolves it, which is a `MediaQuery` lookup and the
/// stored preference.
enum AppThemeMode {
  light('light'),
  dark('dark'),
  system('system');

  const AppThemeMode(this.id);

  /// The value persisted in `appearance.themeMode`.
  final String id;

  static AppThemeMode fromId(String id) => AppThemeMode.values
      .firstWhere((mode) => mode.id == id, orElse: () => system);

  Brightness resolve(BuildContext context) => switch (this) {
        AppThemeMode.light => Brightness.light,
        AppThemeMode.dark => Brightness.dark,
        AppThemeMode.system => MediaQuery.platformBrightnessOf(context),
      };
}
