import 'package:flutter/foundation.dart';
import '../core/models/discover_models.dart';

class DiscoverService extends ChangeNotifier {
  EcosystemCategory _selectedCategory = EcosystemCategory.all;

  EcosystemCategory get selectedCategory => _selectedCategory;

  void selectCategory(EcosystemCategory cat) {
    _selectedCategory = cat;
    notifyListeners();
  }

  List<EcosystemProject> get allProjects => _ecosystemProjects;

  List<EcosystemProject> get filteredProjects {
    if (_selectedCategory == EcosystemCategory.all) {
      return _ecosystemProjects;
    }
    return _ecosystemProjects.where((p) => p.category == _selectedCategory).toList();
  }

  List<NetworkMetric> get networkMetrics => [
        const NetworkMetric(
          title: 'Current TPS',
          value: '4,850 TPS',
          change: '+12.4%',
          isPositive: true,
        ),
        const NetworkMetric(
          title: 'Avg Block Time',
          value: '480 ms',
          change: 'Sub-second',
          isPositive: true,
        ),
        const NetworkMetric(
          title: 'Total Staked',
          value: '68.4M SPRX',
          change: '68.4% Staked',
          isPositive: true,
        ),
        const NetworkMetric(
          title: 'Active Validators',
          value: '100 Nodes',
          change: 'BFT Consensus',
          isPositive: true,
        ),
        const NetworkMetric(
          title: 'Avg Gas Fee',
          value: '< \$0.0001',
          change: 'Ultra-low',
          isPositive: true,
        ),
      ];

  List<EcosystemGuide> get guides => const [
        EcosystemGuide(
          id: 'g_1',
          title: 'Securing Your SPRX Seed Phrase',
          description: 'Learn best practices for hardware isolation, PIN backup, and preventing unauthorized key extraction.',
          duration: '3 min read',
          readCategory: 'Security',
          icon: '🛡️',
        ),
        EcosystemGuide(
          id: 'g_2',
          title: 'Staking SPRX & Earning 12.5% APY',
          description: 'Delegate your native SPRX to top-ranked validators to participate in consensus security and earn automatic epoch rewards.',
          duration: '4 min read',
          readCategory: 'Staking',
          icon: '💎',
        ),
        EcosystemGuide(
          id: 'g_3',
          title: 'Understanding Sprax Chain Perps & Funding Rates',
          description: 'How 8-hour funding payments balance long and short open interest without counterparty risk.',
          duration: '5 min read',
          readCategory: 'Perps',
          icon: '📈',
        ),
        EcosystemGuide(
          id: 'g_4',
          title: 'Cross-Chain Bridges on Sprax Protocol',
          description: 'How native trust-minimized IBC and EVM bridge relayers verify cryptographic proofs instantly.',
          duration: '4 min read',
          readCategory: 'Bridges',
          icon: '🌉',
        ),
      ];

  static const List<EcosystemProject> _ecosystemProjects = [
    EcosystemProject(
      id: 'sprax_swap',
      name: 'SpraxSwap DEX',
      tag: 'DEX',
      description: 'Ultra-low slippage concentrated liquidity AMM native to Sprax Chain.',
      iconEmoji: '⚡',
      category: EcosystemCategory.defi,
      websiteUrl: 'https://swap.sprax.network',
      totalValueLockedUsd: 34500000.0,
      activeUsers24h: 12400,
    ),
    EcosystemProject(
      id: 'sprax_perp',
      name: 'SpraxPerp Protocol',
      tag: 'Perpetuals',
      description: 'Decentralized perpetual futures with up to 50x leverage and sub-second execution.',
      iconEmoji: '📈',
      category: EcosystemCategory.defi,
      websiteUrl: 'https://perps.sprax.network',
      totalValueLockedUsd: 28400000.0,
      activeUsers24h: 8900,
    ),
    EcosystemProject(
      id: 'sprax_lend',
      name: 'SpraxLend Money Market',
      tag: 'Lending',
      description: 'Permissionless lending & borrowing protocol for SPRX, sUSD, and wrapped assets.',
      iconEmoji: '🏦',
      category: EcosystemCategory.defi,
      websiteUrl: 'https://lend.sprax.network',
      totalValueLockedUsd: 19200000.0,
      activeUsers24h: 4500,
    ),
    EcosystemProject(
      id: 'sprax_bridge',
      name: 'Sprax Teleport Bridge',
      tag: 'Bridge',
      description: 'Zero-knowledge cross-chain relayer linking Sprax with Ethereum, Solana, and Cosmos.',
      iconEmoji: '🌉',
      category: EcosystemCategory.bridge,
      websiteUrl: 'https://bridge.sprax.network',
      totalValueLockedUsd: 42000000.0,
      activeUsers24h: 6200,
    ),
    EcosystemProject(
      id: 'sprax_explorer',
      name: 'SpraxScan Block Explorer',
      tag: 'Explorer',
      description: 'Real-time blockchain explorer for blocks, addresses, token transfers, and validator metrics.',
      iconEmoji: '🔍',
      category: EcosystemCategory.tools,
      websiteUrl: 'https://explorer.sprax.network',
    ),
    EcosystemProject(
      id: 'sprax_validator_hub',
      name: 'Validator Staking Portal',
      tag: 'Staking',
      description: 'Official staking registry with commission stats, voting history, and auto-compound tools.',
      iconEmoji: '🛡️',
      category: EcosystemCategory.validators,
      websiteUrl: 'https://validators.sprax.network',
      totalValueLockedUsd: 68400000.0,
      activeUsers24h: 3100,
    ),
    EcosystemProject(
      id: 'sprax_oracle',
      name: 'SpraxFeed Decentralized Oracles',
      tag: 'Oracles',
      description: 'High-frequency sub-second price feeds powering perps, lending, and synthetic assets.',
      iconEmoji: '📡',
      category: EcosystemCategory.infra,
      websiteUrl: 'https://oracles.sprax.network',
    ),
    EcosystemProject(
      id: 'sprax_domain',
      name: 'Sprax Name Service (.sprx)',
      tag: 'Domains',
      description: 'Human-readable decentralized handles for Sprax addresses and identity profiles.',
      iconEmoji: '🏷️',
      category: EcosystemCategory.tools,
      websiteUrl: 'https://sns.sprax.network',
      activeUsers24h: 1800,
    ),
  ];
}
