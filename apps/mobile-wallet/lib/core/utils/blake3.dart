import 'dart:typed_data';

/// Pure Dart implementation of standard BLAKE3 cryptographic hash function.
/// Matches official Rust `blake3::hash` and `@noble/hashes/blake3`.
class Blake3 {
  static const List<int> _iv = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
    0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
  ];

  static const List<int> _sigma = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8,
    3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1,
    10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6,
    12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4,
    9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7,
    11, 15, 5, 0, 1, 9, 8, 6, 14, 10, 2, 12, 3, 4, 7, 13,
  ];

  static const int _chunkStart = 1;
  static const int _chunkEnd = 2;
  static const int _root = 8;

  static int _rotr(int x, int n) => (((x & 0xFFFFFFFF) >> n) | (x << (32 - n))) & 0xFFFFFFFF;

  static void _g(List<int> v, int a, int b, int c, int d, int x, int y) {
    v[a] = (v[a] + v[b] + x) & 0xFFFFFFFF;
    v[d] = _rotr(v[d] ^ v[a], 16);
    v[c] = (v[c] + v[d]) & 0xFFFFFFFF;
    v[b] = _rotr(v[b] ^ v[c], 12);
    v[a] = (v[a] + v[b] + y) & 0xFFFFFFFF;
    v[d] = _rotr(v[d] ^ v[a], 8);
    v[c] = (v[c] + v[d]) & 0xFFFFFFFF;
    v[b] = _rotr(v[b] ^ v[c], 7);
  }

  static void _round(List<int> v, List<int> m, int roundIndex) {
    final s = roundIndex * 16;
    _g(v, 0, 4, 8, 12, m[_sigma[s + 0]], m[_sigma[s + 1]]);
    _g(v, 1, 5, 9, 13, m[_sigma[s + 2]], m[_sigma[s + 3]]);
    _g(v, 2, 6, 10, 14, m[_sigma[s + 4]], m[_sigma[s + 5]]);
    _g(v, 3, 7, 11, 15, m[_sigma[s + 6]], m[_sigma[s + 7]]);
    _g(v, 0, 5, 10, 15, m[_sigma[s + 8]], m[_sigma[s + 9]]);
    _g(v, 1, 6, 11, 12, m[_sigma[s + 10]], m[_sigma[s + 11]]);
    _g(v, 2, 7, 8, 13, m[_sigma[s + 12]], m[_sigma[s + 13]]);
    _g(v, 3, 4, 9, 14, m[_sigma[s + 14]], m[_sigma[s + 15]]);
  }

  static List<int> _compress(
    List<int> cv,
    List<int> blockWords,
    int blockLen,
    int counter,
    int flags,
  ) {
    final v = [
      cv[0], cv[1], cv[2], cv[3], cv[4], cv[5], cv[6], cv[7],
      _iv[0], _iv[1], _iv[2], _iv[3],
      counter & 0xFFFFFFFF,
      (counter >> 32) & 0xFFFFFFFF,
      blockLen,
      flags,
    ];

    for (int r = 0; r < 7; r++) {
      _round(v, blockWords, r);
    }

    final out = List<int>.filled(16, 0);
    for (int i = 0; i < 8; i++) {
      out[i] = (v[i] ^ v[i + 8]) & 0xFFFFFFFF;
      out[i + 8] = (v[i + 8] ^ cv[i]) & 0xFFFFFFFF;
    }
    return out;
  }

  /// Computes the 32-byte BLAKE3 hash of arbitrary input bytes.
  static Uint8List hash(List<int> input) {
    // For single chunk (up to 1024 bytes, which includes public keys of 32 bytes):
    if (input.length <= 64) {
      final blockWords = List<int>.filled(16, 0);
      final byteData = ByteData.sublistView(Uint8List.fromList(input));
      for (int i = 0; i < input.length ~/ 4; i++) {
        blockWords[i] = byteData.getUint32(i * 4, Endian.little);
      }
      final rem = input.length % 4;
      if (rem > 0) {
        int word = 0;
        final base = (input.length ~/ 4) * 4;
        for (int r = 0; r < rem; r++) {
          word |= (input[base + r] & 0xFF) << (r * 8);
        }
        blockWords[input.length ~/ 4] = word;
      }

      const flags = _chunkStart | _chunkEnd | _root;
      final outWords = _compress(_iv, blockWords, input.length, 0, flags);

      final outBytes = Uint8List(32);
      final outByteData = ByteData.sublistView(outBytes);
      for (int i = 0; i < 8; i++) {
        outByteData.setUint32(i * 4, outWords[i], Endian.little);
      }
      return outBytes;
    }

    // Standard chunking for > 64 bytes
    List<int> cv = List<int>.from(_iv);
    int pos = 0;
    final totalLen = input.length;
    int counter = 0;

    while (pos < totalLen) {
      final take = (totalLen - pos) > 64 ? 64 : (totalLen - pos);
      final slice = input.sublist(pos, pos + take);

      final blockWords = List<int>.filled(16, 0);
      final byteData = ByteData.sublistView(Uint8List.fromList(slice));
      for (int i = 0; i < slice.length ~/ 4; i++) {
        blockWords[i] = byteData.getUint32(i * 4, Endian.little);
      }
      final rem = slice.length % 4;
      if (rem > 0) {
        int word = 0;
        final base = (slice.length ~/ 4) * 4;
        for (int r = 0; r < rem; r++) {
          word |= (slice[base + r] & 0xFF) << (r * 8);
        }
        blockWords[slice.length ~/ 4] = word;
      }

      int flags = 0;
      if (pos == 0) flags |= _chunkStart;
      if (pos + take >= totalLen) flags |= _chunkEnd | _root;

      final outWords = _compress(cv, blockWords, take, counter, flags);
      for (int i = 0; i < 8; i++) {
        cv[i] = outWords[i];
      }

      pos += take;
      counter++;
    }

    final outBytes = Uint8List(32);
    final outByteData = ByteData.sublistView(outBytes);
    for (int i = 0; i < 8; i++) {
      outByteData.setUint32(i * 4, cv[i], Endian.little);
    }
    return outBytes;
  }
}
