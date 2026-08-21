# SPRX Protocol: Design System & Brand Identity Specification
**Document Version:** 1.0.0-DESIGN  
**Date:** 2026-08-21  
**Target:** SPRX (Scalable Protocol for Real-world X) Design System ("SPRX Quantum Design System")

---

## 1. Brand Philosophy & Visual Positioning

SPRX stands for **Scalable Protocol for Real-world X**—a foundational Layer-1 blockchain engineered to connect cryptographic consensus with real-world economic utility, payments, tokenized assets, and enterprise systems.

### Core Visual Tenets:
1. **Precision & Engineering Rigor**: Minimalist, technical, high-contrast, and deeply structured. Avoids flashy neon overload, casino tropes, generic floating 3D coins, and meme aesthetic.
2. **Distinct Originality**: No diamond iconography (Ethereum), no circular gradient blobs (Solana/Polkadot), and no purple monoculture. SPRX uses a proprietary visual identity based on **Convergence Topology & Layered Data Rails**.
3. **The "X" Structural Motif**: The letter **X** is incorporated subtly as an intersection of coordinates—representing the convergence of *Digital Consensus* and *Real-world Economic Activity*.
4. **Dark-First FinTech Architecture with Flawless Light Mode**: Built primarily in deep obsidian and titanium with crisp hyper-cyan and emerald data signals, transitioning seamlessly to a pristine high-contrast light mode adhering to WCAG 2.2 AAA accessibility standards.

---

## 2. Color Architecture & Token Tokens

### 2.1 Dark Mode Palette (Default)
| Token Name | Hex Value | CSS Variable | Semantic Usage |
| :--- | :--- | :--- | :--- |
| **`bg-void`** | `#080A0F` | `--color-bg-void` | Deepest canvas background |
| **`bg-base`** | `#0E121A` | `--color-bg-base` | Standard page backdrop |
| **`bg-surface`** | `#141A26` | `--color-bg-surface` | Default card & container surface |
| **`bg-surface-elevated`**| `#1B2333` | `--color-bg-elevated` | Hover states, modals, dropdowns |
| **`border-subtle`** | `#232D42` | `--color-border-subtle`| Card borders, dividers, subtle lines |
| **`border-prominent`** | `#344260` | `--color-border-prominent`| Active inputs, selected cards |
| **`text-primary`** | `#F8FAFC` | `--color-text-primary` | Main titles, primary copy, values |
| **`text-secondary`** | `#94A3B8` | `--color-text-secondary`| Subheadings, labels, metadata |
| **`text-muted`** | `#64748B` | `--color-text-muted` | Footnotes, inactive tabs, timestamps |
| **`sprx-cyan` (Primary)**| `#00F2FE` | `--color-sprx-cyan` | Brand primary, hero accents, active states |
| **`sprx-teal` (Secondary)**| `#4FACFE` | `--color-sprx-teal` | Gradients, interactive links, secondary CTAs |
| **`signal-emerald`** | `#10B981` | `--color-signal-emerald`| Success, healthy validator, positive change |
| **`signal-amber`** | `#F59E0B` | `--color-signal-amber` | Warning, sync catchup, unbonding queue |
| **`signal-coral`** | `#EF4444` | `--color-signal-coral` | Error, slashed validator, rejected tx |
| **`signal-violet`** | `#8B5CF6` | `--color-signal-violet`| Staking pools, governance proposals |

### 2.2 Light Mode Palette
| Token Name | Hex Value | CSS Variable | Semantic Usage |
| :--- | :--- | :--- | :--- |
| **`bg-void-light`** | `#F8FAFC` | `--color-bg-void` | Clean white/slate background |
| **`bg-base-light`** | `#FFFFFF` | `--color-bg-base` | Pure white cards and containers |
| **`bg-surface-light`** | `#F1F5F9` | `--color-bg-surface` | Secondary section backdrops |
| **`border-light`** | `#E2E8F0` | `--color-border-subtle`| Crisp dividers and card borders |
| **`text-primary-light`**| `#0F172A` | `--color-text-primary` | Deep obsidian high-contrast text |
| **`text-secondary-light`**| `#475569` | `--color-text-secondary`| Slate secondary copy |
| **`sprx-cyan-light`** | `#0284C7` | `--color-sprx-cyan` | Deep sky blue for readable contrast on white |
| **`sprx-teal-light`** | `#0D9488` | `--color-sprx-teal` | Deep teal interactive accents |

---

## 3. Typography & Monospace Hierarchy

### 3.1 Typeface Families
- **Primary Display & Interface Font**: `Inter` / `Plus Jakarta Sans` — ultra-clean geometric sans-serif optimized for crisp rendering across micro-labels and massive display headers.
- **Data & Monospace Font**: `JetBrains Mono` / `Fira Code` — high-legibility monospace with distinct glyphs for `0` vs `O`, `1` vs `l`, engineered specifically for **Block Hashes (`0x...`)**, **Bech32 Addresses (`sprax1...`)**, **Gas Numbers**, and **Code Blocks**.

### 3.2 Type Scale
| Level | Font Size | Line Height | Weight | Tracking | Usage |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Display 1** | `56px` (`3.5rem`) | `1.1` | `800` (ExtraBold) | `-0.03em` | Hero main headlines |
| **Display 2** | `40px` (`2.5rem`) | `1.15` | `700` (Bold) | `-0.025em`| Section hero headlines |
| **Heading 1** | `32px` (`2.0rem`) | `1.2` | `700` (Bold) | `-0.02em` | Page titles, primary sections |
| **Heading 2** | `24px` (`1.5rem`) | `1.3` | `600` (SemiBold) | `-0.015em`| Card headers, sub-modules |
| **Heading 3** | `20px` (`1.25rem`)| `1.4` | `600` (SemiBold) | `-0.01em` | Subsection titles, modal titles |
| **Body Large** | `18px` (`1.125rem`)| `1.6` | `400` / `500` | `0` | Lead paragraphs, key summaries |
| **Body Standard**| `15px` (`0.9375rem`)| `1.6` | `400` (Regular) | `0` | Default body text |
| **Body Small** | `13px` (`0.8125rem`)| `1.5` | `400` / `500` | `0.01em` | Table data, metadata labels |
| **Monospace Standard**| `14px` (`0.875rem`)| `1.5` | `500` (Medium) | `0` | Addresses, Hashes, Code blocks |
| **Monospace Small**| `12px` (`0.75rem`)| `1.4` | `500` (Medium) | `0` | Truncated hashes, block heights |

---

## 4. Reusable Component Matrix (40+ Design Primitives)

```
+----------------------------------------------------------------------------------------------------+
|                                    SPRX DESIGN SYSTEM PRIMITIVES                                   |
+----------------------------------------------------------------------------------------------------+
| [Core UI]           [Data & Web3]           [Navigation]          [Feedback & Status]             |
|  - Button (5 types)  - AddressComponent      - MegaMenu            - StatusIndicator (Pulsing)    |
|  - IconButton        - HashComponent         - CommandPalette ⌘K   - NetworkBadge                 |
|  - Card & Container  - MetricCard            - Breadcrumb          - Toast Notification           |
|  - Modal Dialog      - ValidatorCard         - TabGroup            - Skeleton Loader              |
|  - Drawer / Sheet    - EcosystemCard         - Pagination          - Empty State                  |
|  - Accordion         - CodeBlock (Multi-tab) - NetworkSelector     - Error Boundary Fallback      |
|  - Tooltip           - PriceTicker / Chart   - CurrencySelector    - CopyButton                   |
+----------------------------------------------------------------------------------------------------+
```

### 4.1 Detailed Component Specifications

1. **`Button`**:
   - *Primary*: Hyper-Cyan to Electric Teal gradient background, obsidian bold text, glowing focus ring.
   - *Secondary*: Surface elevated background, subtle border, primary text, cyan border on hover.
   - *Ghost / Outline*: Transparent backdrop, prominent border, hover fill.
   - *Danger*: Coral accent for critical actions (e.g., wallet disconnect, unbonding confirmation).
   - *Icon-Leading / Icon-Trailing*: Integrated Lucide React icons with precise 8px spacing.

2. **`MetricCard`**:
   - High-contrast card showing value (e.g. `1,240 TPS`, `1.5s Finality`, `100 Validators`), descriptive label, sparkline indicator, and live pulse dot.

3. **`Interactive Architecture Visualizer`**:
   - Multi-layer interactive diagram depicting the 5 SPRX layers (Applications -> Smart Contracts/WASM -> Consensus/CometBFT -> P2P Network -> Storage/Redb).
   - Interactive hover, expand, step-by-step data flow animation, and click-to-learn documentation modals.

4. **`Live Network Strip`**:
   - Compact status bar at top/hero indicating Chain ID (`sprax-mainnet-1` / `sprax-testnet-1`), current block height, average block time, active validator count, and connection health.

5. **`CodeBlock`**:
   - Tabbed interface supporting **JavaScript / TypeScript**, **Rust**, **cURL**, **Python**, and **Go**.
   - Syntax highlighting, one-click copy button with animated checkmark feedback, and line numbers.

6. **`Filterable Ecosystem Grid`**:
   - Search bar + Category pills (`All`, `DeFi`, `RWA & Payments`, `Wallets`, `Infrastructure`, `Gaming & NFT`, `Enterprise`).
   - Cards with project badge, verified icon, network compatibility indicator, short description, and deep link.

7. **`Omni-Search Command Palette (`⌘K`)`**:
   - Universal search dialog with instant heuristic classification:
     - `sprax1...` -> Address view
     - `0x...` (64 chars) -> Transaction view
     - `#...` / pure integer -> Block view
     - Moniker string -> Validator profile
     - Topic string -> Documentation article or ecosystem app.

---

## 5. Motion & Animation Principles

- **Performance-First**: Use CSS GPU-accelerated transforms (`translate3d`, `opacity`, `scale`) and lightweight Framer Motion springs. Zero heavy 3D WebGL scenes that cause battery drain or lag on mobile devices.
- **Data Flow Transitions**: Soft glowing data packets traveling across SVG connector lines in the architecture diagram to symbolize high-velocity real-world transaction flow.
- **Accessibility Gate (`prefers-reduced-motion`)**: All motion is automatically downgraded to instant opacity switches whenever the user's OS indicates reduced motion preferences.
- **Number Counters**: Smooth easing on metric changes (e.g. Block Height counting up dynamically as new blocks finalize).

---

## 6. Accessibility & Responsiveness Targets

- **WCAG 2.2 AA / AAA Compliance**: Text contrast ratios exceeding `4.5:1` for standard text and `7:1` for titles.
- **Keyboard Navigation**: Full tab navigation support across all mega-menus, modals, data tables, and search commands with high-visibility focus indicators.
- **Screen Reader Semantics**: Proper ARIA landmarks (`role="banner"`, `role="navigation"`, `role="main"`, `aria-expanded`, `aria-live="polite"` for block updates).
- **Responsive Viewport Spectrum**:
  - `Mobile Small`: 320px – 375px
  - `Mobile Standard`: 390px – 430px
  - `Tablet`: 768px – 1024px
  - `Desktop`: 1280px – 1440px
  - `Ultra-Wide`: 1920px+
