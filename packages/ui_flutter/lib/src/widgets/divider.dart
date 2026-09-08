// Copyright 2014 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import 'package:flutter/widgets.dart';
import '../theme/theme.dart';

/// How heavily a separator is drawn.
///
/// `muted` is the heavier rule, for a separator that has to hold its own
/// against a filled surface. It is `border-strong` — ink at a tenth — and
/// deliberately not the outline a checkbox or a radio is drawn in: that role
/// is an opaque grey around twice this weight, and a separator set in it
/// stops reading as a hairline and starts reading as a border.
enum DividerTone { normal, muted }

/// A separator is the line and nothing else.
///
/// `divider.css` gives the rule no margin and no box beyond its own weight:
/// the space around a separator belongs to whatever is laying it out, which
/// is the only way one rule can sit tight between two list rows and loose
/// between two sections. A default of its own would be a decision this
/// component is not the one making.
const double? _kDividerSpace = null;
const double _kDividerIndent = 0;
const double _kDividerEndIndent = 0;

/// A thin horizontal line, and nothing else.
///
/// The box's total height is [height], and the rule is centred in it. Left
/// alone, that height is the rule's own weight — the space around a separator
/// belongs to whatever is laying it out.
///
/// See also:
///
///  * [VerticalDivider], the vertical analog of this widget.
class Divider extends StatelessWidget {
  /// Creates a divider.
  ///
  /// The [height], [thickness], [indent], and [endIndent] must be null or
  /// non-negative.
  const Divider({
    super.key,
    this.height,
    this.thickness,
    this.indent,
    this.endIndent,
    this.color,
    this.tone = DividerTone.normal,
  }) : assert(height == null || height >= 0.0),
       assert(thickness == null || thickness >= 0.0),
       assert(indent == null || indent >= 0.0),
       assert(endIndent == null || endIndent >= 0.0);

  /// The divider's height extent.
  ///
  /// The divider itself is always drawn as a horizontal line that is centered
  /// within the height specified by this value.
  ///
  /// If this is null, the divider is as tall as [thickness] — the rule with
  /// no room around it.
  final double? height;

  /// The thickness of the line drawn within the divider.
  ///
  /// If this is null, the rule is one device pixel: `stroke.hairline`, halved
  /// on a Retina display.
  final double? thickness;

  /// The amount of empty space to the leading edge of the divider.
  ///
  /// If this is null, the rule runs edge to edge.
  final double? indent;

  /// The amount of empty space to the trailing edge of the divider.
  ///
  /// If this is null, the rule runs edge to edge.
  final double? endIndent;

  /// The color to use when painting the line.
  ///
  /// If this is null, the ink the [tone] names is used.
  final Color? color;

  /// How heavily the rule is drawn.
  final DividerTone tone;

  /// Computes the [BorderSide] that represents a divider.
  ///
  /// If [color] is null, the ink [tone] names is read off the theme. If
  /// [width] is null, the rule is one device pixel. If [context] is null there
  /// is no theme to read, so [BorderSide]'s own defaults stand.
  ///
  /// {@tool snippet}
  ///
  /// This example uses this method to create a box that has a divider above and
  /// below it. This is sometimes useful with lists, for instance, to separate a
  /// scrollable section from the rest of the interface.
  ///
  /// ```dart
  /// DecoratedBox(
  ///   decoration: BoxDecoration(
  ///     border: Border(
  ///       top: Divider.createBorderSide(context),
  ///       bottom: Divider.createBorderSide(context),
  ///     ),
  ///   ),
  ///   // child: ...
  /// )
  /// ```
  /// {@end-tool}
  static BorderSide createBorderSide(
    BuildContext? context, {
    Color? color,
    double? width,
    DividerTone tone = DividerTone.normal,
  }) {
    final ThemeData? theme = context != null ? Theme.of(context) : null;

    final Color? effectiveColor =
        color ??
        switch (tone) {
          DividerTone.normal => theme?.vars.colorBorder,
          DividerTone.muted => theme?.vars.colorBorderStrong,
        };
    final double effectiveWidth =
        width ?? (context != null ? context.hairlineWidth : 1.0);

    // Prevent assertion since it is possible that context is null and no color
    // is specified.
    if (effectiveColor == null) {
      return BorderSide(
        width: effectiveWidth,
      );
    }
    return BorderSide(
      color: effectiveColor,
      width: effectiveWidth,
    );
  }

  @override
  Widget build(BuildContext context) {
    final double thickness = this.thickness ?? context.hairlineWidth;
    final double height = this.height ?? _kDividerSpace ?? thickness;
    final double indent = this.indent ?? _kDividerIndent;
    final double endIndent = this.endIndent ?? _kDividerEndIndent;

    return SizedBox(
      height: height,
      child: Center(
        child: Container(
          height: thickness,
          margin: EdgeInsetsDirectional.only(start: indent, end: endIndent),
          // A fill rather than an edge, which is what `divider.css` draws:
          // a rule the height of its own weight, in the border ink.
          decoration: BoxDecoration(
            color: createBorderSide(
              context,
              color: color,
              width: thickness,
              tone: tone,
            ).color,
          ),
        ),
      ),
    );
  }
}

/// A thin vertical line, and nothing else.
///
/// The box's total width is [width], and the rule is centred in it. Left
/// alone, that width is the rule's own weight.
///
/// See also:
///
///  * [ListView.separated], which can be used to generate vertical dividers.
///  * [Divider], the horizontal analog of this widget.
class VerticalDivider extends StatelessWidget {
  /// Creates a vertical divider.
  ///
  /// The [width], [thickness], [indent], and [endIndent] must be null or
  /// non-negative.
  const VerticalDivider({
    super.key,
    this.width,
    this.thickness,
    this.indent,
    this.endIndent,
    this.color,
    this.tone = DividerTone.normal,
  }) : assert(width == null || width >= 0.0),
       assert(thickness == null || thickness >= 0.0),
       assert(indent == null || indent >= 0.0),
       assert(endIndent == null || endIndent >= 0.0);

  /// The divider's width.
  ///
  /// The divider itself is always drawn as a vertical line that is centered
  /// within the width specified by this value.
  ///
  /// If this is null, the divider is as wide as [thickness] — the rule with
  /// no room around it.
  final double? width;

  /// The thickness of the line drawn within the divider.
  ///
  /// If this is null, the rule is one device pixel: `stroke.hairline`, halved
  /// on a Retina display.
  final double? thickness;

  /// The amount of empty space on top of the divider.
  ///
  /// If this is null, the rule runs edge to edge.
  final double? indent;

  /// The amount of empty space under the divider.
  ///
  /// If this is null, the rule runs edge to edge.
  final double? endIndent;

  /// The color to use when painting the line.
  ///
  /// If this is null, the ink the [tone] names is used.
  final Color? color;

  /// How heavily the rule is drawn.
  final DividerTone tone;

  @override
  Widget build(BuildContext context) {
    final double thickness = this.thickness ?? context.hairlineWidth;
    final double width = this.width ?? _kDividerSpace ?? thickness;
    final double indent = this.indent ?? _kDividerIndent;
    final double endIndent = this.endIndent ?? _kDividerEndIndent;

    return SizedBox(
      width: width,
      child: Center(
        child: Container(
          width: thickness,
          margin: EdgeInsetsDirectional.only(top: indent, bottom: endIndent),
          // A fill rather than an edge, the way the horizontal one draws.
          decoration: BoxDecoration(
            color: Divider.createBorderSide(
              context,
              color: color,
              width: thickness,
              tone: tone,
            ).color,
          ),
        ),
      ),
    );
  }
}
