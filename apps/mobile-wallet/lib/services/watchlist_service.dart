import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'secure_storage_service.dart';

class WatchlistService extends ChangeNotifier {
  final SecureStorageService _storage = SecureStorageService();
  final Set<String> _watchedAssetIds = {'sprx', 'btc', 'eth'};
  bool _isInitialized = false;

  Set<String> get watchedAssetIds => _watchedAssetIds;
  bool get isInitialized => _isInitialized;

  Future<void> initialize() async {
    try {
      final saved = await _storage.getString('sprax_watchlist');
      if (saved != null && saved.isNotEmpty) {
        final List<dynamic> list = jsonDecode(saved);
        _watchedAssetIds.clear();
        _watchedAssetIds.addAll(list.map((e) => e.toString()));
      }
    } catch (_) {}
    _isInitialized = true;
    notifyListeners();
  }

  bool isWatched(String assetId) {
    return _watchedAssetIds.contains(assetId.toLowerCase());
  }

  Future<void> toggleWatchlist(String assetId) async {
    final cleanId = assetId.toLowerCase();
    if (_watchedAssetIds.contains(cleanId)) {
      _watchedAssetIds.remove(cleanId);
    } else {
      _watchedAssetIds.add(cleanId);
    }
    await _save();
    notifyListeners();
  }

  Future<void> addToWatchlist(String assetId) async {
    final cleanId = assetId.toLowerCase();
    if (!_watchedAssetIds.contains(cleanId)) {
      _watchedAssetIds.add(cleanId);
      await _save();
      notifyListeners();
    }
  }

  Future<void> removeFromWatchlist(String assetId) async {
    final cleanId = assetId.toLowerCase();
    if (_watchedAssetIds.contains(cleanId)) {
      _watchedAssetIds.remove(cleanId);
      await _save();
      notifyListeners();
    }
  }

  Future<void> _save() async {
    try {
      await _storage.saveString(
        'sprax_watchlist',
        jsonEncode(_watchedAssetIds.toList()),
      );
    } catch (_) {}
  }
}
