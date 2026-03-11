import 'package:flutter_test/flutter_test.dart';
import 'package:done_with_debt/main.dart';

void main() {
  testWidgets('App smoke test', (WidgetTester tester) async {
    await tester.pumpWidget(const DoneWithDebtApp());
    expect(find.text('Done With Debt'), findsOneWidget);
  });
}
