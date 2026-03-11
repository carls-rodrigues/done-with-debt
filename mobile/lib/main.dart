import 'package:flutter/material.dart';

void main() {
  runApp(const DoneWithDebtApp());
}

class DoneWithDebtApp extends StatelessWidget {
  const DoneWithDebtApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Done With Debt',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF6366F1)),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF6366F1),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      themeMode: ThemeMode.system,
      home: const Scaffold(
        body: Center(child: Text('Done With Debt')),
      ),
    );
  }
}
