// Go test program: crypto, hashing, byte manipulation, error handling
// Tests obfuscator against Go's runtime patterns (GC, stack growth, etc.)
package main

import (
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"hash/crc32"
	"math/bits"
	"os"
	"sync/atomic"
)

var pass, fail int32

func check(cond bool, msg string) {
	if cond {
		fmt.Printf("  [OK] %s\n", msg)
		atomic.AddInt32(&pass, 1)
	} else {
		fmt.Printf("  [FAIL] %s\n", msg)
		atomic.AddInt32(&fail, 1)
	}
}

// --- XXTEA ---
func xxteaEncrypt(v []uint32, key [4]uint32) {
	n := len(v)
	if n < 2 {
		return
	}
	delta := uint32(0x9E3779B9)
	rounds := 6 + 52/n
	var sum uint32
	z := v[n-1]
	for ; rounds > 0; rounds-- {
		sum += delta
		e := (sum >> 2) & 3
		for p := 0; p < n-1; p++ {
			y := v[p+1]
			v[p] += ((z>>5 ^ y<<2) + (y>>3 ^ z<<4)) ^ ((sum ^ y) + (key[(uint32(p)&3)^e] ^ z))
			z = v[p]
		}
		y := v[0]
		v[n-1] += ((z>>5 ^ y<<2) + (y>>3 ^ z<<4)) ^ ((sum ^ y) + (key[(uint32(n-1)&3)^e] ^ z))
		z = v[n-1]
	}
}

func xxteaDecrypt(v []uint32, key [4]uint32) {
	n := len(v)
	if n < 2 {
		return
	}
	delta := uint32(0x9E3779B9)
	rounds := 6 + 52/n
	sum := uint32(rounds) * delta
	y := v[0]
	for ; rounds > 0; rounds-- {
		e := (sum >> 2) & 3
		for p := n - 1; p > 0; p-- {
			z := v[p-1]
			v[p] -= ((z>>5 ^ y<<2) + (y>>3 ^ z<<4)) ^ ((sum ^ y) + (key[(uint32(p)&3)^e] ^ z))
			y = v[p]
		}
		z := v[n-1]
		v[0] -= ((z>>5 ^ y<<2) + (y>>3 ^ z<<4)) ^ ((sum ^ y) + (key[0^e] ^ z))
		y = v[0]
		sum -= delta
	}
}

// --- Bit manipulation ---
func reverseBits32(x uint32) uint32 {
	return bits.Reverse32(x)
}

func popcount(x uint64) int {
	return bits.OnesCount64(x)
}

// --- Simple PRNG (xorshift64) ---
type Xorshift64 struct {
	state uint64
}

func (x *Xorshift64) Next() uint64 {
	x.state ^= x.state << 13
	x.state ^= x.state >> 7
	x.state ^= x.state << 17
	return x.state
}

// --- Byte buffer builder ---
type ByteBuilder struct {
	buf []byte
}

func (b *ByteBuilder) WriteU32LE(v uint32) {
	tmp := make([]byte, 4)
	binary.LittleEndian.PutUint32(tmp, v)
	b.buf = append(b.buf, tmp...)
}

func (b *ByteBuilder) WriteU64BE(v uint64) {
	tmp := make([]byte, 8)
	binary.BigEndian.PutUint64(tmp, v)
	b.buf = append(b.buf, tmp...)
}

func (b *ByteBuilder) WriteString(s string) {
	b.buf = append(b.buf, []byte(s)...)
}

func (b *ByteBuilder) Bytes() []byte {
	return b.buf
}

// --- XOR pad ---
func xorPad(data, key []byte) []byte {
	out := make([]byte, len(data))
	for i, b := range data {
		out[i] = b ^ key[i%len(key)]
	}
	return out
}

// --- Base16 encode/decode ---
func base16Encode(data []byte) string {
	return hex.EncodeToString(data)
}

func base16Decode(s string) ([]byte, error) {
	return hex.DecodeString(s)
}

func main() {
	fmt.Println("=== Go Crypto/Byte Tests ===")

	// XXTEA roundtrip
	key := [4]uint32{0x01234567, 0x89ABCDEF, 0xFEDCBA98, 0x76543210}
	data := []uint32{0xDEADBEEF, 0xCAFEBABE, 0x12345678, 0x9ABCDEF0}
	original := make([]uint32, len(data))
	copy(original, data)

	xxteaEncrypt(data, key)
	encrypted := make([]uint32, len(data))
	copy(encrypted, data)

	differs := false
	for i := range data {
		if data[i] != original[i] {
			differs = true
			break
		}
	}
	check(differs, "XXTEA encrypts (differs from original)")

	xxteaDecrypt(data, key)
	matches := true
	for i := range data {
		if data[i] != original[i] {
			matches = false
			break
		}
	}
	check(matches, "XXTEA roundtrip")

	// Different key produces different ciphertext
	key2 := [4]uint32{0x11111111, 0x22222222, 0x33333333, 0x44444444}
	data2 := []uint32{0xDEADBEEF, 0xCAFEBABE, 0x12345678, 0x9ABCDEF0}
	xxteaEncrypt(data2, key2)
	diffKeys := false
	for i := range encrypted {
		if encrypted[i] != data2[i] {
			diffKeys = true
			break
		}
	}
	check(diffKeys, "XXTEA different keys differ")

	// SHA-256
	h := sha256.Sum256([]byte("hello"))
	hexHash := hex.EncodeToString(h[:])
	check(hexHash == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
		"sha256('hello')")

	emptyHash := sha256.Sum256([]byte(""))
	emptyHex := hex.EncodeToString(emptyHash[:])
	check(emptyHex == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
		"sha256('')")

	// CRC32
	crc := crc32.ChecksumIEEE([]byte("123456789"))
	check(crc == 0xCBF43926, "crc32('123456789')")

	// Bit operations
	check(reverseBits32(1) == 0x80000000, "reverseBits(1)")
	check(reverseBits32(0x80000000) == 1, "reverseBits(0x80000000)")
	check(popcount(0xAAAAAAAAAAAAAAAA) == 32, "popcount(0xAAAA...)")
	check(popcount(0) == 0, "popcount(0)")
	check(popcount(0xFFFFFFFFFFFFFFFF) == 64, "popcount(all ones)")

	// Xorshift64 PRNG
	rng := &Xorshift64{state: 42}
	first := rng.Next()
	check(first != 0, "xorshift non-zero")
	// Determinism
	rng2 := &Xorshift64{state: 42}
	check(rng2.Next() == first, "xorshift deterministic")
	// 1000 iterations
	for i := 0; i < 1000; i++ {
		rng.Next()
	}
	check(rng.state != 0, "xorshift after 1000 iters")

	// ByteBuilder
	bb := &ByteBuilder{}
	bb.WriteU32LE(0xDEADBEEF)
	bb.WriteU64BE(0x0123456789ABCDEF)
	bb.WriteString("test")
	bytes := bb.Bytes()
	check(len(bytes) == 4+8+4, "byte builder length")
	check(bytes[0] == 0xEF && bytes[1] == 0xBE, "LE u32 byte order")
	check(bytes[4] == 0x01 && bytes[5] == 0x23, "BE u64 byte order")

	// XOR pad roundtrip
	plaintext := []byte("secret message here!")
	key3 := []byte("mykey")
	encrypted2 := xorPad(plaintext, key3)
	check(string(encrypted2) != string(plaintext), "xor pad encrypts")
	decrypted := xorPad(encrypted2, key3)
	check(string(decrypted) == string(plaintext), "xor pad roundtrip")

	// Base16
	encoded := base16Encode([]byte{0xDE, 0xAD, 0xBE, 0xEF})
	check(encoded == "deadbeef", "base16 encode")
	decoded, err := base16Decode("cafebabe")
	check(err == nil && len(decoded) == 4, "base16 decode")
	check(decoded[0] == 0xCA && decoded[1] == 0xFE, "base16 decode values")

	_, err = base16Decode("xyz")
	check(err != nil, "base16 decode invalid")

	// Slice manipulation
	s := make([]byte, 256)
	for i := range s {
		s[i] = byte(i)
	}
	// Reverse
	for i, j := 0, len(s)-1; i < j; i, j = i+1, j-1 {
		s[i], s[j] = s[j], s[i]
	}
	check(s[0] == 255 && s[255] == 0, "byte slice reverse")

	// Checksum of reversed
	var checksum uint32
	for _, b := range s {
		checksum += uint32(b)
	}
	check(checksum == 32640, "reversed slice checksum")

	fmt.Printf("\n%d passed, %d failed\n", pass, fail)
	if fail > 0 {
		os.Exit(1)
	}
}
