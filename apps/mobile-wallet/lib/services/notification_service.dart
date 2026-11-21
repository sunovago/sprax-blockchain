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
    // Real notifications will be triggered by blockchain transactions and user activity
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
