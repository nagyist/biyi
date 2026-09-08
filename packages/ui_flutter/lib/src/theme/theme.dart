// ignore_for_file: annotate_overrides

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../foundation/theme_extension.dart';
import '../generated/theme_variables.dart';
import '../painting/varianted_widget_state_color.dart';
import '../painting/widget_property.dart';
import '../widgets/badge_theme.dart';
import '../widgets/button_theme.dart';
import '../widgets/callout_theme.dart';
import '../widgets/card_theme.dart';
import '../widgets/checkbox_theme.dart';
import '../widgets/key_cap_theme.dart';

/// A theme data.
class ThemeData extends ThemeExtension<ThemeData> with DiagnosticableTreeMixin {
  /// Studio Light — the baseline every other theme is written against.
  static ThemeData studioLight() {
    return ThemeData(
      vars: themeVariables,
      brightness: Brightness.light,
    );
  }

  /// Studio Dark — the same skeleton over a near-black canvas.
  static ThemeData studioDark() {
    return ThemeData(
      vars: themeVariablesStudioDark,
      brightness: Brightness.dark,
    );
  }

  /// Bright Light — warm paper, ink navy, acid green marker.
  static ThemeData brightLight() {
    return ThemeData(
      vars: themeVariablesBrightLight,
      brightness: Brightness.light,
    );
  }

  /// Bright Dark — the same pair with the roles swapped.
  static ThemeData brightDark() {
    return ThemeData(
      vars: themeVariablesBrightDark,
      brightness: Brightness.dark,
    );
  }

  /// Frost Light — slate-grey paper with a teal accent.
  static ThemeData frostLight() {
    return ThemeData(
      vars: themeVariablesFrostLight,
      brightness: Brightness.light,
    );
  }

  /// Frost Dark — the same teal over a deep blue-slate canvas.
  static ThemeData frostDark() {
    return ThemeData(
      vars: themeVariablesFrostDark,
      brightness: Brightness.dark,
    );
  }

  /// Graphite Light — monochrome, and the one family with tighter corners.
  static ThemeData graphiteLight() {
    return ThemeData(
      vars: themeVariablesGraphiteLight,
      brightness: Brightness.light,
    );
  }

  /// Graphite Dark — the same, inverted: a near-white accent on near-black.
  static ThemeData graphiteDark() {
    return ThemeData(
      vars: themeVariablesGraphiteDark,
      brightness: Brightness.dark,
    );
  }

  /// Ember Light — warm paper with a copper accent.
  static ThemeData emberLight() {
    return ThemeData(
      vars: themeVariablesEmberLight,
      brightness: Brightness.light,
    );
  }

  /// Ember Dark — a charcoal canvas with that copper lifted to an amber.
  static ThemeData emberDark() {
    return ThemeData(
      vars: themeVariablesEmberDark,
      brightness: Brightness.dark,
    );
  }

  /// Nocturne Light — cool white paper, blue-grey ink, a blurple accent.
  static ThemeData nocturneLight() {
    return ThemeData(
      vars: themeVariablesNocturneLight,
      brightness: Brightness.light,
    );
  }

  /// Nocturne Dark — a blue-grey near-black, with the same blurple lifted.
  static ThemeData nocturneDark() {
    return ThemeData(
      vars: themeVariablesNocturneDark,
      brightness: Brightness.dark,
    );
  }

  /// Creates a dark theme — Studio Dark under its older name.
  static ThemeData dark() => studioDark();

  /// Creates a light theme — Studio Light under its older name.
  static ThemeData light() => studioLight();

  const ThemeData({
    this.vars = themeVariables,
    required this.brightness,
    this.badgeTheme = const BadgeThemeData(),
    this.buttonTheme = const ButtonThemeData(),
    this.calloutTheme = const CalloutThemeData(),
    this.cardTheme = const CardThemeData(),
    this.checkboxTheme = const CheckboxThemeData(),
    this.keyCapTheme = const KeyCapThemeData(),
  });

  /// The variables of the theme.
  final ThemeVariables vars;

  /// The brightness of the design theme.
  final Brightness brightness;

  /// The badge theme of the design theme.
  final BadgeThemeData badgeTheme;

  /// The button theme of the design theme.
  final ButtonThemeData buttonTheme;

  /// The callout theme of the design theme.
  final CalloutThemeData calloutTheme;

  /// The card theme of the design theme.
  final CardThemeData cardTheme;

  /// The checkbox theme of the design theme.
  final CheckboxThemeData checkboxTheme;

  /// The kbd theme of the design theme.
  final KeyCapThemeData keyCapTheme;

  /// Creates a copy of this object but with the given fields replaced with the
  /// new values.
  @override
  ThemeData copyWith({
    ThemeVariables? vars,
    Brightness? brightness,
    BadgeThemeData? badgeTheme,
    ButtonThemeData? buttonTheme,
    CalloutThemeData? calloutTheme,
    CardThemeData? cardTheme,
    CheckboxThemeData? checkboxTheme,
    KeyCapThemeData? keyCapTheme,
  }) {
    return ThemeData(
      vars: vars ?? this.vars,
      brightness: brightness ?? this.brightness,
      badgeTheme: badgeTheme ?? this.badgeTheme,
      buttonTheme: buttonTheme ?? this.buttonTheme,
      calloutTheme: calloutTheme ?? this.calloutTheme,
      cardTheme: cardTheme ?? this.cardTheme,
      checkboxTheme: checkboxTheme ?? this.checkboxTheme,
      keyCapTheme: keyCapTheme ?? this.keyCapTheme,
    );
  }

  /// Linearly interpolates between this [ThemeData] and another.
  @override
  ThemeData lerp(
    covariant ThemeExtension<ThemeData>? other,
    double t,
  ) {
    if (other is! ThemeData) return this;
    return ThemeData(
      vars: t < 0.5 ? vars : other.vars,
      brightness: t < 0.5 ? brightness : other.brightness,
      badgeTheme: t < 0.5 ? badgeTheme : other.badgeTheme,
      buttonTheme: t < 0.5 ? buttonTheme : other.buttonTheme,
      calloutTheme: t < 0.5 ? calloutTheme : other.calloutTheme,
      cardTheme: t < 0.5 ? cardTheme : other.cardTheme,
      checkboxTheme: t < 0.5 ? checkboxTheme : other.checkboxTheme,
      keyCapTheme: t < 0.5 ? keyCapTheme : other.keyCapTheme,
    );
  }

  @override
  int get hashCode => Object.hash(
    vars,
    brightness,
    badgeTheme,
    buttonTheme,
    calloutTheme,
    checkboxTheme,
    keyCapTheme,
  );

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) {
      return true;
    }
    if (other.runtimeType != runtimeType) {
      return false;
    }
    return other is ThemeData &&
        other.vars == vars &&
        other.brightness == brightness &&
        other.badgeTheme == badgeTheme &&
        other.buttonTheme == buttonTheme &&
        other.calloutTheme == calloutTheme &&
        other.cardTheme == cardTheme &&
        other.checkboxTheme == checkboxTheme &&
        other.keyCapTheme == keyCapTheme;
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties
      ..add(DiagnosticsProperty('brightness', brightness))
      ..add(DiagnosticsProperty('badgeTheme', badgeTheme))
      ..add(DiagnosticsProperty('buttonTheme', buttonTheme))
      ..add(DiagnosticsProperty('calloutTheme', calloutTheme))
      ..add(DiagnosticsProperty('cardTheme', cardTheme))
      ..add(DiagnosticsProperty('checkboxTheme', checkboxTheme))
      ..add(DiagnosticsProperty('keyCapTheme', keyCapTheme));
  }
}

/// A extension that provides the design theme data.
extension ThemeDataBuildContextProps on BuildContext {
  /// The design theme data.
  ThemeData get themeData => Theme.of(this);

  /// The design theme's variables.
  ThemeVariables get vars => Theme.of(this).vars;

  /// A separator is one *logical* pixel, whatever the display scale — the
  /// 1px the token holds, drawn as written.
  ///
  /// It used to halve on Retina, to land on one device pixel; the Nocturne
  /// mockups the system is drawn to rule at a full 1px, and against them a
  /// halved rule read as too thin to be a decision. Drawn control outlines
  /// use `stroke.control` instead, and always did.
  double get hairlineWidth => vars.strokeHairline;
}

/// A inherited widget that provides the design theme data.
///
/// It also sets the ambient text style the subtree is read in, which is the
/// one thing leaving material behind would otherwise have cost: Flutter has
/// no default face below an app the way a browser has one below `<body>`, and
/// a `Text` with nothing above it is drawn in the engine's error style. A
/// `Material` used to carry it; the theme carries it now, the way every other
/// kit's theme does.
class Theme extends InheritedTheme {
  Theme({
    super.key,
    required this.data,
    required Widget child,
  }) : super(
         child: _Ambient(data: data, child: child),
       );

  final ThemeData data;

  @override
  bool updateShouldNotify(Theme oldWidget) {
    return data != oldWidget.data;
  }

  @override
  Widget wrap(BuildContext context, Widget child) {
    return Theme(
      data: data,
      child: child,
    );
  }

  /// The theme the current subtree is rendered under, falling back to Studio
  /// Light so a widget dropped into a bare app still paints correctly rather
  /// than throwing on a missing ancestor.
  ///
  /// There is no second place to look: this widget is the only thing that
  /// carries a theme. A host whose app is someone else's — a `MaterialApp`,
  /// a `CupertinoApp` — wraps the subtree it draws this kit in, which is what
  /// the playground does around one window rather than around the process.
  static ThemeData of(BuildContext context) {
    final theme = context.dependOnInheritedWidgetOfExactType<Theme>();
    return theme?.data ?? ThemeData.studioLight();
  }
}

/// The body face, merged in rather than set, and that is the whole of the
/// contract: `bodyMedium` names no family of its own by default — the design
/// asks for the platform's own UI face — so a family a host sets above the
/// theme reaches every label, and a component's own style merges over this
/// in turn.
class _Ambient extends StatelessWidget {
  const _Ambient({required this.data, required this.child});

  final ThemeData data;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return DefaultTextStyle.merge(
      style: data.vars.bodyMedium.copyWith(color: data.vars.colorContent),
      child: child,
    );
  }
}

extension ThemeVariablesX on ThemeVariables {
  /// The control seed color.
  WidgetProperty<ColorSwatch<int>> get controlColor {
    return TintedWidgetProperty<ColorSwatch<int>>(
      primary: colorPrimary,
      neutral: colorNeutral,
      success: colorSuccess,
      info: colorInfo,
      warning: colorWarning,
      danger: colorDanger,
    );
  }

  WidgetProperty<Color> get controlColorSurface {
    return VariantedWidgetStateColor(
      normal: controlColorNormalSurface,
      recessed: controlColorRecessedSurface,
      filled: controlColorFilledSurface,
      tinted: controlColorTintedSurface,
      outlined: controlColorOutlinedSurface,
      plain: controlColorPlainSurface,
    );
  }

  WidgetProperty<Color> get controlColorContent {
    return VariantedWidgetStateColor(
      normal: controlColorNormalContent,
      recessed: controlColorRecessedContent,
      filled: controlColorFilledContent,
      tinted: controlColorTintedContent,
      outlined: controlColorOutlinedContent,
      plain: controlColorPlainContent,
    );
  }

  WidgetProperty<Color> get controlColorBorder {
    return VariantedWidgetStateColor(
      normal: controlColorNormalBorder,
      recessed: controlColorRecessedBorder,
      filled: controlColorFilledBorder,
      tinted: controlColorTintedBorder,
      outlined: controlColorOutlinedBorder,
      plain: controlColorPlainBorder,
    );
  }
}
