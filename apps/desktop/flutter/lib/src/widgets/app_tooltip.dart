/// A hover label.
///
/// Material's `Tooltip` went with material, and the kit ships none — a tooltip
/// is chrome, and the kit draws no chrome. This is the same affordance in the
/// app's own paint: it waits out a brush past before it appears, hangs under
/// the control it names, and never takes the pointer, so the control below it
/// keeps its hover.
library;

import 'dart:async';

import 'package:flutter/widgets.dart';

import '../theme/product_tokens.dart' show ProductTypography;
import 'ui.dart' show ThemeDataBuildContextProps, ThemeVariables;

/// How long the pointer has to rest before the label appears. Long enough that
/// crossing a toolbar does not light up every button on the way.
const Duration _kDelay = Duration(milliseconds: 500);

class AppTooltip extends StatefulWidget {
  const AppTooltip({super.key, required this.message, required this.child});

  final String message;
  final Widget child;

  @override
  State<AppTooltip> createState() => _AppTooltipState();
}

class _AppTooltipState extends State<AppTooltip> {
  final OverlayPortalController _controller = OverlayPortalController();
  final LayerLink _link = LayerLink();
  Timer? _timer;

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  void _schedule() {
    _timer?.cancel();
    _timer = Timer(_kDelay, () {
      if (mounted) _controller.show();
    });
  }

  void _cancel() {
    _timer?.cancel();
    _timer = null;
    if (_controller.isShowing) _controller.hide();
  }

  @override
  Widget build(BuildContext context) {
    final ThemeVariables vars = context.vars;

    return MouseRegion(
      onEnter: (_) => _schedule(),
      onExit: (_) => _cancel(),
      child: CompositedTransformTarget(
        link: _link,
        child: OverlayPortal(
          controller: _controller,
          overlayChildBuilder: (overlayContext) => Positioned(
            // The follower is positioned by the link; the Positioned only
            // gives the overlay a box to lay it out in.
            left: 0,
            top: 0,
            child: CompositedTransformFollower(
              link: _link,
              showWhenUnlinked: false,
              targetAnchor: Alignment.bottomCenter,
              followerAnchor: Alignment.topCenter,
              offset: Offset(0, vars.spacing15),
              // The label is a label, not a target: a pointer that reaches it
              // is still hovering the control underneath.
              child: IgnorePointer(
                child: Container(
                  padding: EdgeInsets.symmetric(
                    horizontal: vars.spacing2,
                    vertical: vars.spacing1,
                  ),
                  decoration: BoxDecoration(
                    color: vars.colorSurfaceRaised,
                    borderRadius: BorderRadius.circular(vars.radiusSmall),
                    border: Border.all(
                      color: vars.colorBorderStrong,
                      width: context.hairlineWidth,
                    ),
                    boxShadow: vars.shadowSm,
                  ),
                  child: Text(
                    widget.message,
                    style: vars.sansStyle(
                      fontSize: 12,
                      height: 1,
                      color: vars.colorContentSecondary,
                    ),
                  ),
                ),
              ),
            ),
          ),
          child: widget.child,
        ),
      ),
    );
  }
}
