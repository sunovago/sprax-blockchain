import {
  AddressUtil,
  HDWallet,
  KeyAlgorithm,
  MnemonicUtil,
  TransactionBuilder,
  WalletVault,
} from "../src";

describe("SPRX Wallet Core Test Suite", () => {
  const TEST_MNEMONIC =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

  describe("1. BIP-39 Mnemonic Generation & Validation", () => {
    it("generates a valid 12-word mnemonic phrase", () => {
      const mnemonic = MnemonicUtil.generate(12);
      const words = mnemonic.split(" ");
      expect(words.length).toBe(12);
      expect(MnemonicUtil.validate(mnemonic)).toBe(true);
    });

    it("generates a valid 24-word mnemonic phrase", () => {
      const mnemonic = MnemonicUtil.generate(24);
      const words = mnemonic.split(" ");
      expect(words.length).toBe(24);
      expect(MnemonicUtil.validate(mnemonic)).toBe(true);
    });

    it("rejects invalid mnemonic words or corrupted checksums", () => {
      const invalidWords = "foo bar baz invalid word count list test random fake nonword";
      expect(MnemonicUtil.validate(invalidWords)).toBe(false);

      // Mutate one word in a valid phrase to corrupt checksum
      const corrupted =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
      expect(MnemonicUtil.validate(corrupted)).toBe(false);
    });
  });

  describe("2. Deterministic HD Wallet Derivation & Recovery", () => {
    it("derives consistent Ed25519 addresses from the same seed", () => {
      const wallet1 = HDWallet.fromMnemonic(TEST_MNEMONIC);
      const wallet2 = HDWallet.fromMnemonic(TEST_MNEMONIC);

      const acc1 = wallet1.deriveAccount(0);
      const acc2 = wallet2.deriveAccount(0);

      expect(acc1.account.addressBech32).toBe(acc2.account.addressBech32);
      expect(acc1.account.addressHex).toBe(acc2.account.addressHex);
      expect(acc1.account.publicKeyHex).toBe(acc2.account.publicKeyHex);
      expect(acc1.account.addressBech32.startsWith("sprax1")).toBe(true);
      expect(acc1.account.addressHex.startsWith("0x")).toBe(true);
    });

    it("derives distinct accounts for different indices", () => {
      const wallet = HDWallet.fromMnemonic(TEST_MNEMONIC);
      const acc0 = wallet.deriveAccount(0);
      const acc1 = wallet.deriveAccount(1);

      expect(acc0.account.addressBech32).not.toBe(acc1.account.addressBech32);
      expect(acc0.account.index).toBe(0);
      expect(acc1.account.index).toBe(1);
    });

    it("derives Secp256k1 interoperability accounts", () => {
      const wallet = HDWallet.fromMnemonic(TEST_MNEMONIC);
      const secpAcc = wallet.deriveSecp256k1Account(0);

      expect(secpAcc.account.algorithm).toBe(KeyAlgorithm.Secp256k1);
      expect(secpAcc.account.addressBech32.startsWith("sprax1")).toBe(true);
    });
  });

  describe("3. Address Encoding & Validation", () => {
    it("correctly encodes and validates Bech32 and Hex formats", () => {
      const rawBytes = new Uint8Array(20).fill(7);
      const bech32Str = AddressUtil.toBech32(rawBytes);
      const hexStr = AddressUtil.toHex(rawBytes);

      expect(AddressUtil.isValidAddress(bech32Str)).toBe(true);
      expect(AddressUtil.isValidAddress(hexStr)).toBe(true);

      const parsedFromBech32 = AddressUtil.parseToBytes(bech32Str);
      const parsedFromHex = AddressUtil.parseToBytes(hexStr);

      expect(parsedFromBech32).toEqual(rawBytes);
      expect(parsedFromHex).toEqual(rawBytes);
    });

    it("rejects invalid address strings", () => {
      expect(AddressUtil.isValidAddress("not-an-address")).toBe(false);
      expect(AddressUtil.isValidAddress("sprax1invalidchecksum")).toBe(false);
      expect(AddressUtil.isValidAddress("0x123")).toBe(false); // too short
    });
  });

  describe("4. Transaction Construction & Offline Signing", () => {
    it("converts SPRX decimal and atomic atto units with full precision", () => {
      expect(TransactionBuilder.sprxToAtto("1")).toBe(1000000000000000000n);
      expect(TransactionBuilder.sprxToAtto("0.5")).toBe(500000000000000000n);
      expect(TransactionBuilder.sprxToAtto("10.000000000000000001")).toBe(10000000000000000001n);

      expect(TransactionBuilder.attoToSprx(1000000000000000000n)).toBe("1");
      expect(TransactionBuilder.attoToSprx(500000000000000000n)).toBe("0.5");
    });

    it("signs transaction payload offline with Ed25519", () => {
      const wallet = HDWallet.fromMnemonic(TEST_MNEMONIC);
      const sender = wallet.deriveAccount(0);
      const recipient = wallet.deriveAccount(1);

      const signedTx = TransactionBuilder.sign(
        {
          fromAddress: sender.account.addressBech32,
          toAddress: recipient.account.addressBech32,
          amountSprx: "250.75",
          nonce: 3,
          memo: "Payment for services",
        },
        "sprax-devnet-1",
        sender.privateKey,
        KeyAlgorithm.Ed25519
      );

      expect(signedTx.body.chainId).toBe("sprax-devnet-1");
      expect(signedTx.body.sender).toBe(sender.account.addressBech32);
      expect(signedTx.body.nonce).toBe(3);
      expect(signedTx.body.messages[0].amount).toBe("250750000000000000000");
      expect(signedTx.keyType).toBe("Ed25519");
      expect(signedTx.signature.startsWith("0x")).toBe(true);
      expect(signedTx.publicKey.startsWith("0x")).toBe(true);
    });

    it("rejects transaction construction with invalid recipient or negative amount", () => {
      const wallet = HDWallet.fromMnemonic(TEST_MNEMONIC);
      const sender = wallet.deriveAccount(0);

      expect(() => {
        TransactionBuilder.sign(
          {
            fromAddress: sender.account.addressBech32,
            toAddress: "invalid_recipient_address",
            amountSprx: "100",
            nonce: 0,
          },
          "sprax-devnet-1",
          sender.privateKey
        );
      }).toThrow(/Invalid recipient address/);

      expect(() => {
        TransactionBuilder.sign(
          {
            fromAddress: sender.account.addressBech32,
            toAddress: sender.account.addressBech32,
            amountSprx: "0",
            nonce: 0,
          },
          "sprax-devnet-1",
          sender.privateKey
        );
      }).toThrow(/strictly greater than 0/);
    });
  });

  describe("5. Password-Protected Keystore Vault", () => {
    it("encrypts and decrypts mnemonic phrase with password", async () => {
      const password = "SuperSecretPassword123!";
      const vault = await WalletVault.encrypt(TEST_MNEMONIC, password, 2);

      expect(vault.version).toBe(1);
      expect(vault.accounts.length).toBe(2);
      expect(vault.cipherTextHex).toBeDefined();

      const decryptedMnemonic = await WalletVault.decrypt(vault, password);
      expect(decryptedMnemonic).toBe(TEST_MNEMONIC);
    });
  });
});
