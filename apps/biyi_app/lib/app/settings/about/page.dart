import 'package:biyi_app/generated/locale_keys.g.dart';
import 'package:biyi_app/widgets/widgets.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/material.dart';

class AboutSettingPage extends StatelessWidget {
  const AboutSettingPage({super.key});

  Widget _buildBody(BuildContext context) {
    return Container();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CustomAppBar(
        title: Text(LocaleKeys.app_settings_about_title.tr()),
      ),
      body: _buildBody(context),
    );
  }
}
