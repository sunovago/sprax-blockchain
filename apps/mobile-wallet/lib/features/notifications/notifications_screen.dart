import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../app/theme/app_theme.dart';
import '../../core/models/notification_models.dart';
import '../../services/notification_service.dart';

class NotificationsScreen extends StatelessWidget {
  const NotificationsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final notifService = context.watch<NotificationService>();

    return Scaffold(
      appBar: AppBar(
        title: const Text('Notifications'),
        actions: [
          if (notifService.notifications.isNotEmpty)
            TextButton(
              onPressed: () => notifService.markAllAsRead(),
              child: const Text('Mark all read', style: TextStyle(color: AppTheme.primaryCyan, fontSize: 13)),
            ),
        ],
      ),
      body: SafeArea(
        child: notifService.notifications.isEmpty
            ? const Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Icon(Icons.notifications_none, size: 48, color: AppTheme.textMuted),
                    SizedBox(height: 12),
                    Text('No notifications', style: TextStyle(color: AppTheme.textMuted, fontSize: 14)),
                  ],
                ),
              )
            : ListView.separated(
                padding: const EdgeInsets.all(16),
                itemCount: notifService.notifications.length,
                separatorBuilder: (_, __) => const SizedBox(height: 10),
                itemBuilder: (context, idx) {
                  final notif = notifService.notifications[idx];
                  return Card(
                    color: notif.isRead ? AppTheme.darkCard : AppTheme.darkCardElevated,
                    child: ListTile(
                      onTap: () => notifService.markAsRead(notif.id),
                      leading: CircleAvatar(
                        backgroundColor: _getColorForType(notif.type).withAlpha(30),
                        child: Icon(_getIconForType(notif.type), color: _getColorForType(notif.type), size: 20),
                      ),
                      title: Row(
                        children: [
                          Expanded(
                            child: Text(
                              notif.title,
                              style: TextStyle(
                                fontWeight: notif.isRead ? FontWeight.w600 : FontWeight.w800,
                                fontSize: 14,
                                color: notif.isRead ? Colors.white70 : Colors.white,
                              ),
                            ),
                          ),
                          if (!notif.isRead)
                            Container(
                              width: 8,
                              height: 8,
                              decoration: const BoxDecoration(
                                shape: BoxShape.circle,
                                color: AppTheme.primaryCyan,
                              ),
                            ),
                        ],
                      ),
                      subtitle: Padding(
                        padding: const EdgeInsets.only(top: 4.0),
                        child: Text(
                          notif.message,
                          style: const TextStyle(fontSize: 12, color: AppTheme.textMuted),
                        ),
                      ),
                    ),
                  );
                },
              ),
      ),
    );
  }

  IconData _getIconForType(NotificationType type) {
    switch (type) {
      case NotificationType.priceAlert:
        return Icons.trending_up;
      case NotificationType.orderFilled:
        return Icons.check_circle_outline;
      case NotificationType.positionWarning:
        return Icons.warning_amber_rounded;
      case NotificationType.txConfirmed:
        return Icons.receipt_long;
      case NotificationType.stakingReward:
        return Icons.savings_outlined;
      case NotificationType.systemNotice:
        return Icons.info_outline;
    }
  }

  Color _getColorForType(NotificationType type) {
    switch (type) {
      case NotificationType.priceAlert:
        return AppTheme.primaryCyan;
      case NotificationType.orderFilled:
        return AppTheme.successGreen;
      case NotificationType.positionWarning:
        return AppTheme.errorRed;
      case NotificationType.txConfirmed:
        return AppTheme.accentPurple;
      case NotificationType.stakingReward:
        return AppTheme.warningOrange;
      case NotificationType.systemNotice:
        return Colors.blueAccent;
    }
  }
}
