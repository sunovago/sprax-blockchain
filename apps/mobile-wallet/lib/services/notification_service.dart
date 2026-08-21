import 'package:flutter/foundation.dart';
import '../core/models/notification_models.dart';

class NotificationService extends ChangeNotifier {
  final List<AppNotification> _notifications = [];

  List<AppNotification> get notifications => _notifications;
  int get unreadCount => _notifications.where((n) => !n.isRead).length;

  NotificationService() {
    _initDefaultNotifications();
  }

  void _initDefaultNotifications() {
    final now = DateTime.now();
    _notifications.addAll([
      AppNotification(
        id: 'notif_1',
        title: 'Price Alert: SPRX +9.65%',
        message: 'SPRX broke above \$1.25 with surging 24h volume on Sprax Chain.',
        type: NotificationType.priceAlert,
        timestamp: now.subtract(const Duration(minutes: 15)),
        isRead: false,
      ),
      AppNotification(
        id: 'notif_2',
        title: 'Testnet Perps Order Executed',
        message: 'Long 2,000 SPRX at \$1.18 filled successfully in Demo Mode.',
        type: NotificationType.orderFilled,
        timestamp: now.subtract(const Duration(hours: 3)),
        isRead: false,
      ),
      AppNotification(
        id: 'notif_3',
        title: 'Staking Epoch Reward Received',
        message: 'You received +0.485 SPRX from Core Foundation Validator staking delegation.',
        type: NotificationType.stakingReward,
        timestamp: now.subtract(const Duration(hours: 18)),
        isRead: true,
      ),
      AppNotification(
        id: 'notif_4',
        title: 'Network Upgrade: Sprax v1.2',
        message: 'Sub-second BFT finality and lower gas parameters are now active.',
        type: NotificationType.systemNotice,
        timestamp: now.subtract(const Duration(days: 1)),
        isRead: true,
      ),
    ]);
  }

  void addNotification({
    required String title,
    required String message,
    required NotificationType type,
    String? routePayload,
  }) {
    _notifications.insert(
      0,
      AppNotification(
        id: 'notif_${DateTime.now().millisecondsSinceEpoch}',
        title: title,
        message: message,
        type: type,
        timestamp: DateTime.now(),
        isRead: false,
        routePayload: routePayload,
      ),
    );
    notifyListeners();
  }

  void markAsRead(String id) {
    final idx = _notifications.indexWhere((n) => n.id == id);
    if (idx >= 0 && !_notifications[idx].isRead) {
      _notifications[idx] = _notifications[idx].copyWith(isRead: true);
      notifyListeners();
    }
  }

  void markAllAsRead() {
    for (int i = 0; i < _notifications.length; i++) {
      _notifications[i] = _notifications[i].copyWith(isRead: true);
    }
    notifyListeners();
  }

  void clearAll() {
    _notifications.clear();
    notifyListeners();
  }
}
