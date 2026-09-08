/// @docImport '../theme/theme.dart';
library;

/// The shape every part of a [ThemeData] is written to: a `copyWith` that
/// names each field, and a `lerp` that animates between two of them.
///
/// This is `material.ThemeExtension` with material left out of it. The
/// contract is the whole of what those classes used it for — the ability to
/// hang one on a material `ThemeData` and read it back with
/// `extension<T>()` was never how this package's [Theme] finds its data, and
/// pulling in the whole material library to keep that door open is the trade
/// this package no longer makes. A host on `MaterialApp` wraps its subtree in
/// this package's [Theme] instead, which is one widget rather than a library.
abstract class ThemeExtension<T extends ThemeExtension<T>> {
  const ThemeExtension();

  /// The key this extension is stored under. Two instances of one type are
  /// the same extension, which is what makes an override an override.
  Object get type => T;

  /// A copy of this object with the given fields replaced.
  ThemeExtension<T> copyWith();

  /// Linearly interpolates towards [other].
  ThemeExtension<T> lerp(covariant ThemeExtension<T>? other, double t);
}
