enum EcosystemCategory {
  all('All'),
  defi('DeFi'),
  infra('Infrastructure'),
  bridge('Bridge'),
  validators('Validators'),
  tools('Tools & Analytics'),
  nft('NFT & Gaming');

  final String label;
  const EcosystemCategory(this.label);
}

class EcosystemProject {
  final String id;
  final String name;
  final String tag;
  final String description;
  final String iconEmoji;
  final EcosystemCategory category;
  final String websiteUrl;
  final String contractAddress;
  final bool isVerified;
  final double totalValueLockedUsd;
  final int activeUsers24h;

  const EcosystemProject({
    required this.id,
    required this.name,
    required this.tag,
    required this.description,
    required this.iconEmoji,
    required this.category,
    this.websiteUrl = 'https://sprax.network',
    this.contractAddress = '',
    this.isVerified = true,
    this.totalValueLockedUsd = 0.0,
    this.activeUsers24h = 0,
  });
}

class NetworkMetric {
  final String title;
  final String value;
  final String change;
  final bool isPositive;

  const NetworkMetric({
    required this.title,
    required this.value,
    required this.change,
    this.isPositive = true,
  });
}

class EcosystemGuide {
  final String id;
  final String title;
  final String description;
  final String duration;
  final String readCategory;
  final String icon;

  const EcosystemGuide({
    required this.id,
    required this.title,
    required this.description,
    required this.duration,
    required this.readCategory,
    required this.icon,
  });
}
