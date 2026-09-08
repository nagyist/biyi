import 'package:flutter/widgets.dart';

import '../../theme/product_tokens.dart' show ProductTypography;
import '../../widgets/ui.dart' show ThemeDataBuildContextProps, ThemeVariables;

/// The chrome the two `/debug/*` pages share.
///
/// They used to sit in a `Scaffold` under a material `AppBar`; the app has
/// neither now. Nothing about a debug page needs more than a titled band and a
/// body, so this is that, in the kit's own surfaces.
class DebugScaffold extends StatelessWidget {
  const DebugScaffold({super.key, required this.title, required this.body});

  final String title;
  final Widget body;

  @override
  Widget build(BuildContext context) {
    final ThemeVariables vars = context.vars;

    return ColoredBox(
      color: vars.colorSurface,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            padding: EdgeInsets.symmetric(
              horizontal: vars.spacing5,
              vertical: vars.spacing4,
            ),
            decoration: BoxDecoration(
              border: Border(
                bottom: BorderSide(
                  color: vars.colorBorder,
                  width: context.hairlineWidth,
                ),
              ),
            ),
            child: Text(
              title,
              style: vars.sansStyle(
                fontSize: 15,
                fontWeight: FontWeight.w600,
                color: vars.colorContent,
              ),
            ),
          ),
          Expanded(child: body),
        ],
      ),
    );
  }
}
