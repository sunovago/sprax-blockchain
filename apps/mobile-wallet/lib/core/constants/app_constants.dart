/// Network and Blockchain Constants for Sprax Chain
class NetworkConfig {
  final String name;
  final String chainId;
  final String rpcUrl;
  final String explorerUrl;
  final String addressPrefix;
  final String symbol;
  final int decimals;
  final bool isTestnet;

  const NetworkConfig({
    required this.name,
    required this.chainId,
    required this.rpcUrl,
    required this.explorerUrl,
    this.addressPrefix = 'sprax',
    this.symbol = 'SPRX',
    this.decimals = 18,
    this.isTestnet = true,
  });
}

class AppConstants {
  static const String appName = 'SPRX Wallet';
  static const String appVersion = '1.0.0';
  static const String addressPrefix = 'sprax';
  static const int coinType = 9999;
  static const String defaultDerivationPath = "m/44'/9999'/0'/0";

  // Pre-configured Networks
  static const NetworkConfig mainnet = NetworkConfig(
    name: 'SPRX Mainnet',
    chainId: 'sprax-mainnet-1',
    rpcUrl: 'https://rpc.sprax.network',
    explorerUrl: 'https://explorer.sprax.network',
    isTestnet: false,
  );

  static const NetworkConfig testnet = NetworkConfig(
    name: 'SPRX Public Testnet',
    chainId: 'sprax-testnet-1',
    rpcUrl: 'https://testnet-rpc.sprax.network',
    explorerUrl: 'https://testnet.sprax.network',
    isTestnet: true,
  );

  static const NetworkConfig devnet = NetworkConfig(
    name: String.fromEnvironment(
      'SPRAX_DEVNET_NAME',
      defaultValue: 'Local Devnet (Desktop/iOS)',
    ),
    chainId: 'sprax-devnet-1',
    rpcUrl: String.fromEnvironment(
      'SPRAX_DEVNET_RPC_URL',
      defaultValue: 'http://127.0.0.1:26657',
    ),
    explorerUrl: String.fromEnvironment(
      'SPRAX_DEVNET_EXPLORER_URL',
      defaultValue: 'http://127.0.0.1:3000',
    ),
    isTestnet: true,
  );

  static const NetworkConfig androidEmulatorDevnet = NetworkConfig(
    name: 'Local Devnet (Android Emulator)',
    chainId: 'sprax-devnet-1',
    rpcUrl: 'http://10.0.2.2:26657',
    explorerUrl: 'http://10.0.2.2:3000',
    isTestnet: true,
  );

  static const NetworkConfig lanDevnet = NetworkConfig(
    name: 'Local Devnet (Wi-Fi / LAN)',
    chainId: 'sprax-devnet-1',
    rpcUrl: 'http://10.111.250.173:26657',
    explorerUrl: 'http://10.111.250.173:3000',
    isTestnet: true,
  );

  static const List<NetworkConfig> supportedNetworks = [
    devnet,
    lanDevnet,
    androidEmulatorDevnet,
    testnet,
    mainnet,
  ];

  // Default Fee (0.001 SPRX)
  static final BigInt defaultFeeAtto = BigInt.from(1000000000000000); // 0.001 * 10^18
}

enum FiatCurrency {
  inr('INR', '₹', 85.50),
  usd('USD', '\$', 1.00),
  eur('EUR', '€', 0.92),
  gbp('GBP', '£', 0.78),
  jpy('JPY', '¥', 155.20);

  final String code;
  final String symbol;
  final double rateToUsd;

  const FiatCurrency(this.code, this.symbol, this.rateToUsd);
}
