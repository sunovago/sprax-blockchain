import 'package:flutter/foundation.dart';
import '../core/constants/app_constants.dart';
import '../core/utils/crypto_utils.dart';
import 'secure_storage_service.dart';
import 'rpc_service.dart';
import 'signing_service.dart';
import 'market_price_service.dart';

class WalletAccount {
  final int index;
  final String name;
  final String address;
  final Uint8List publicKey;

  const WalletAccount({
    required this.index,
    required this.name,
    required this.address,
    required this.publicKey,
  });
}

class WalletService extends ChangeNotifier {
  final SecureStorageService _storage = SecureStorageService();
  final RpcService _rpc = RpcService();
  final MarketPriceService _market = MarketPriceService();

  bool _isInitialized = false;
  bool _isLocked = false;
  bool _isLoading = false;
  String? _mnemonic;

  NetworkConfig _currentNetwork = AppConstants.testnet;
  FiatCurrency _selectedCurrency = FiatCurrency.inr;

  List<WalletAccount> _accounts = [];
  int _activeAccountIndex = 0;
  BigInt _balanceAtto = BigInt.zero;
  int _nonce = 0;
  final List<TxHistoryItem> _transactions = [];
  List<ValidatorInfo> _validators = [];

  // Getters
  bool get isInitialized => _isInitialized;
  bool get isLocked => _isLocked;
  bool get isLoading => _isLoading;
  bool get hasWallet => _mnemonic != null && _mnemonic!.isNotEmpty;
  String? get mnemonic => _mnemonic;
  NetworkConfig get currentNetwork => _currentNetwork;
  FiatCurrency get selectedCurrency => _selectedCurrency;
  List<WalletAccount> get accounts => _accounts;
  WalletAccount? get activeAccount => _accounts.isNotEmpty ? _accounts[_activeAccountIndex] : null;
  BigInt get balanceAtto => _balanceAtto;
  int get nonce => _nonce;
  List<TxHistoryItem> get transactions => _transactions;
  List<ValidatorInfo> get validators => _validators;
  double get sprxUsdPrice => _market.sprxUsdPrice;

  /// Initializes the wallet state from secure device storage on app launch.
  Future<void> initialize() async {
    _isLoading = true;
    notifyListeners();

    try {
      final savedMnemonic = await _storage.getMnemonic();
      final hasPin = await _storage.hasPin();

      if (savedMnemonic != null && savedMnemonic.isNotEmpty) {
        _mnemonic = savedMnemonic;
        if (hasPin) {
          _isLocked = true;
        }

        // Restore network
        final savedChainId = await _storage.getNetworkChainId();
        if (savedChainId != null) {
          _currentNetwork = AppConstants.supportedNetworks.firstWhere(
            (n) => n.chainId == savedChainId,
            orElse: () => AppConstants.testnet,
          );
        }

        // Restore currency
        final savedCurr = await _storage.getFiatCurrency();
        if (savedCurr != null) {
          _selectedCurrency = FiatCurrency.values.firstWhere(
            (c) => c.code == savedCurr,
            orElse: () => FiatCurrency.inr,
          );
        }

        // Derive accounts
        final accountsCount = await _storage.getAccountsCount();
        _accounts = [];
        for (int i = 0; i < accountsCount; i++) {
          final kp = CryptoUtils.deriveAccountFromMnemonic(_mnemonic!, i);
          _accounts.add(WalletAccount(
            index: i,
            name: "Account ${i + 1}",
            address: kp.address,
            publicKey: kp.publicKeyBytes,
          ));
        }

        _activeAccountIndex = await _storage.getActiveAccountIndex();
        if (_activeAccountIndex >= _accounts.length) {
          _activeAccountIndex = 0;
        }

        await refreshBalance();
      }
    } catch (_) {
      // Graceful fallback
    } finally {
      _isInitialized = true;
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Creates a new wallet with fresh cryptographically secure 12-word mnemonic.
  Future<String> createNewWallet() async {
    final phrase = CryptoUtils.generateMnemonic();
    _mnemonic = phrase;
    return phrase;
  }

  /// Finalizes wallet setup after user confirms backup and sets security PIN.
  Future<void> finalizeWalletSetup(String pin) async {
    if (_mnemonic == null) throw Exception("No mnemonic generated");

    await _storage.saveMnemonic(_mnemonic!);
    await _storage.setPin(pin);
    await _storage.setAccountsCount(1);
    await _storage.setActiveAccountIndex(0);

    final kp = CryptoUtils.deriveAccountFromMnemonic(_mnemonic!, 0);
    _accounts = [
      WalletAccount(
        index: 0,
        name: "Primary Account",
        address: kp.address,
        publicKey: kp.publicKeyBytes,
      )
    ];
    _activeAccountIndex = 0;
    _isLocked = false;

    await refreshBalance();
    notifyListeners();
  }

  /// Imports an existing wallet from a 12 or 24-word recovery phrase.
  Future<void> importWallet(String phrase, String pin) async {
    if (!CryptoUtils.validateMnemonic(phrase)) {
      throw Exception("Invalid recovery phrase checksum");
    }

    _mnemonic = phrase.trim();
    await finalizeWalletSetup(pin);
  }

  /// Unlocks the app using security PIN or biometric authentication.
  Future<bool> unlockWithPin(String pin) async {
    final isValid = await _storage.verifyPin(pin);
    if (isValid) {
      _isLocked = false;
      notifyListeners();
      return true;
    }
    return false;
  }

  void lockApp() {
    _isLocked = true;
    notifyListeners();
  }

  /// Refreshes the on-chain account balance and transaction status via Sprax Chain RPC.
  Future<void> refreshBalance() async {
    if (activeAccount == null) return;
    _isLoading = true;
    notifyListeners();

    try {
      final accInfo = await _rpc.getAccount(_currentNetwork.rpcUrl, activeAccount!.address);
      _balanceAtto = accInfo.balance;
      _nonce = accInfo.nonce;
      _validators = await _rpc.getValidators(_currentNetwork.rpcUrl);
      await _market.fetchLatestMarketPrice();
    } catch (_) {
      // Offline fallback
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Switches active account index.
  Future<void> switchAccount(int index) async {
    if (index >= 0 && index < _accounts.length) {
      _activeAccountIndex = index;
      await _storage.setActiveAccountIndex(index);
      await refreshBalance();
      notifyListeners();
    }
  }

  /// Adds a new deterministically derived account.
  Future<void> addNewAccount(String name) async {
    if (_mnemonic == null) return;
    final nextIndex = _accounts.length;
    final kp = CryptoUtils.deriveAccountFromMnemonic(_mnemonic!, nextIndex);
    _accounts.add(WalletAccount(
      index: nextIndex,
      name: name.isNotEmpty ? name : "Account ${nextIndex + 1}",
      address: kp.address,
      publicKey: kp.publicKeyBytes,
    ));
    await _storage.setAccountsCount(_accounts.length);
    await switchAccount(nextIndex);
  }

  /// Switches active network (Mainnet, Testnet, Devnet).
  Future<void> switchNetwork(NetworkConfig network) async {
    _currentNetwork = network;
    await _storage.setNetworkChainId(network.chainId);
    await refreshBalance();
    notifyListeners();
  }

  /// Changes display fiat currency.
  Future<void> setFiatCurrency(FiatCurrency currency) async {
    _selectedCurrency = currency;
    await _storage.setFiatCurrency(currency.code);
    notifyListeners();
  }

  /// Signs and submits an on-chain transfer transaction.
  Future<String> sendTransfer({
    required String recipientAddress,
    required BigInt amountAtto,
    BigInt? feeAtto,
    String memo = "SPRX Mobile Transfer",
  }) async {
    if (_mnemonic == null || activeAccount == null) {
      throw Exception("Wallet is not unlocked");
    }

    if (!CryptoUtils.isValidSpraxAddress(recipientAddress)) {
      throw Exception("Invalid Sprax recipient address format");
    }

    final fee = feeAtto ?? AppConstants.defaultFeeAtto;
    if (_balanceAtto < (amountAtto + fee)) {
      throw Exception("Insufficient SPRX balance for amount and fee");
    }

    final kp = CryptoUtils.deriveAccountFromMnemonic(_mnemonic!, _activeAccountIndex);

    // Build and sign transaction locally
    final signedTx = SigningService.buildAndSignTransferTx(
      chainId: _currentNetwork.chainId,
      senderAddress: activeAccount!.address,
      recipientAddress: recipientAddress,
      amountAtto: amountAtto,
      feeAtto: fee,
      nonce: _nonce,
      timeoutHeight: 1000000,
      privateKeyBytes: kp.privateKeyBytes,
      publicKeyBytes: kp.publicKeyBytes,
      memo: memo,
    );

    // Broadcast signed payload to RPC
    final txHash = await _rpc.broadcastTx(_currentNetwork.rpcUrl, signedTx);

    // Record local transaction history
    _transactions.insert(
      0,
      TxHistoryItem(
        txHash: txHash,
        blockHeight: 0,
        sender: activeAccount!.address,
        recipient: recipientAddress,
        amount: amountAtto,
        fee: fee,
        status: "CONFIRMED",
        timestampUnix: DateTime.now().millisecondsSinceEpoch ~/ 1000,
      ),
    );

    // Optimistically update local nonce & balance
    _nonce += 1;
    _balanceAtto = _balanceAtto - amountAtto - fee;
    notifyListeners();

    return txHash;
  }

  /// Wipes all wallet keys and resets app to initial onboarding state.
  Future<void> wipeWallet() async {
    await _storage.wipeWallet();
    _mnemonic = null;
    _accounts = [];
    _activeAccountIndex = 0;
    _balanceAtto = BigInt.zero;
    _isLocked = false;
    notifyListeners();
  }
}
