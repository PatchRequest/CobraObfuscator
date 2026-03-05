// Tests: heavy bitwise operations, shifts, rotates, masks
// Stresses instruction substitution and dead-code passes
#include <stdio.h>

static unsigned int rotl32(unsigned int x, int n) {
    return (x << n) | (x >> (32 - n));
}

static unsigned int rotr32(unsigned int x, int n) {
    return (x >> n) | (x << (32 - n));
}

// SipHash-like mixing
static unsigned long long sip_round(unsigned long long v0, unsigned long long v1,
                                     unsigned long long v2, unsigned long long v3) {
    v0 += v1; v1 = ((v1 << 13) | (v1 >> 51)) ^ v0;
    v0 = ((v0 << 32) | (v0 >> 32));
    v2 += v3; v3 = ((v3 << 16) | (v3 >> 48)) ^ v2;
    v0 += v3; v3 = ((v3 << 21) | (v3 >> 43)) ^ v0;
    v2 += v1; v1 = ((v1 << 17) | (v1 >> 47)) ^ v2;
    v2 = ((v2 << 32) | (v2 >> 32));
    return v0 ^ v1 ^ v2 ^ v3;
}

// Bit reversal
static unsigned int reverse_bits(unsigned int x) {
    x = ((x >> 1) & 0x55555555) | ((x & 0x55555555) << 1);
    x = ((x >> 2) & 0x33333333) | ((x & 0x33333333) << 2);
    x = ((x >> 4) & 0x0F0F0F0F) | ((x & 0x0F0F0F0F) << 4);
    x = ((x >> 8) & 0x00FF00FF) | ((x & 0x00FF00FF) << 8);
    x = (x >> 16) | (x << 16);
    return x;
}

// Population count (software)
static int popcount32(unsigned int x) {
    x = x - ((x >> 1) & 0x55555555);
    x = (x & 0x33333333) + ((x >> 2) & 0x33333333);
    return (int)(((x + (x >> 4)) & 0x0F0F0F0F) * 0x01010101 >> 24);
}

// Count leading zeros (software)
static int clz32(unsigned int x) {
    if (x == 0) return 32;
    int n = 0;
    if ((x & 0xFFFF0000) == 0) { n += 16; x <<= 16; }
    if ((x & 0xFF000000) == 0) { n += 8;  x <<= 8; }
    if ((x & 0xF0000000) == 0) { n += 4;  x <<= 4; }
    if ((x & 0xC0000000) == 0) { n += 2;  x <<= 2; }
    if ((x & 0x80000000) == 0) { n += 1; }
    return n;
}

// XOR cipher round
static void xor_block(unsigned int *block, int len, unsigned int key) {
    for (int i = 0; i < len; i++) {
        block[i] ^= key;
        key = rotl32(key, 5) ^ block[i];
    }
}

// CRC32 (bit-by-bit, no table)
static unsigned int crc32_byte(unsigned int crc, unsigned char byte) {
    crc ^= byte;
    for (int i = 0; i < 8; i++) {
        if (crc & 1)
            crc = (crc >> 1) ^ 0xEDB88320;
        else
            crc >>= 1;
    }
    return crc;
}

static unsigned int crc32(const unsigned char *data, int len) {
    unsigned int crc = 0xFFFFFFFF;
    for (int i = 0; i < len; i++)
        crc = crc32_byte(crc, data[i]);
    return crc ^ 0xFFFFFFFF;
}

int main(void) {
    int pass = 0, fail = 0;
#define CHECK(cond, msg) do { \
    if (cond) { printf("  [OK] %s\n", msg); pass++; } \
    else { printf("  [FAIL] %s\n", msg); fail++; } \
} while(0)

    printf("=== Bitops Tests ===\n");

    // Rotate
    CHECK(rotl32(1, 4) == 16, "rotl32(1,4)");
    CHECK(rotl32(0x80000000, 1) == 1, "rotl32 wrap");
    CHECK(rotr32(16, 4) == 1, "rotr32(16,4)");
    CHECK(rotr32(1, 1) == 0x80000000, "rotr32 wrap");

    // Bit reversal
    CHECK(reverse_bits(1) == 0x80000000, "reverse_bits(1)");
    CHECK(reverse_bits(0x80000000) == 1, "reverse_bits(0x80000000)");
    CHECK(reverse_bits(0x12345678) == 0x1E6A2C48, "reverse_bits(0x12345678)");

    // Popcount
    CHECK(popcount32(0) == 0, "popcount(0)");
    CHECK(popcount32(0xFFFFFFFF) == 32, "popcount(0xFFFFFFFF)");
    CHECK(popcount32(0xAAAAAAAA) == 16, "popcount(0xAAAAAAAA)");
    CHECK(popcount32(0x12345678) == 13, "popcount(0x12345678)");

    // CLZ
    CHECK(clz32(0) == 32, "clz(0)");
    CHECK(clz32(1) == 31, "clz(1)");
    CHECK(clz32(0x80000000) == 0, "clz(0x80000000)");
    CHECK(clz32(0x00010000) == 15, "clz(0x10000)");

    // SipHash mixing
    unsigned long long h = sip_round(0x0706050403020100ULL, 0x0F0E0D0C0B0A0908ULL,
                                      0x1716151413121110ULL, 0x1F1E1D1C1B1A1918ULL);
    CHECK(h != 0, "sip_round non-zero");
    unsigned long long h2 = sip_round(0x0706050403020100ULL, 0x0F0E0D0C0B0A0908ULL,
                                       0x1716151413121110ULL, 0x1F1E1D1C1B1A1918ULL);
    CHECK(h == h2, "sip_round deterministic");

    // XOR cipher roundtrip
    unsigned int block[8] = {0x11111111, 0x22222222, 0x33333333, 0x44444444,
                              0x55555555, 0x66666666, 0x77777777, 0x88888888};
    unsigned int original[8];
    for (int i = 0; i < 8; i++) original[i] = block[i];

    xor_block(block, 8, 0xDEADBEEF);
    int differs = 0;
    for (int i = 0; i < 8; i++) if (block[i] != original[i]) differs++;
    CHECK(differs > 0, "xor_block encrypts");

    // CRC32
    CHECK(crc32((const unsigned char *)"", 0) == 0x00000000, "crc32 empty");
    CHECK(crc32((const unsigned char *)"123456789", 9) == 0xCBF43926, "crc32('123456789')");
    CHECK(crc32((const unsigned char *)"hello", 5) == 0x3610A686, "crc32('hello')");

    printf("\n%d passed, %d failed\n", pass, fail);
    return fail > 0 ? 1 : 0;
}
