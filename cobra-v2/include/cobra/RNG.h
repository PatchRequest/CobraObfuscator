#pragma once
#include <cstdint>
#include <random>

namespace cobra {

class RNG {
public:
    explicit RNG(uint64_t seed) : engine(seed) {}

    uint64_t next() { return dist(engine); }

    uint32_t nextU32() { return static_cast<uint32_t>(next()); }

    uint32_t nextInRange(uint32_t lo, uint32_t hi) {
        return lo + (nextU32() % (hi - lo));
    }

    bool chance(double probability) {
        return (nextU32() % 10000) < static_cast<uint32_t>(probability * 10000);
    }

    RNG fork() { return RNG(next()); }

private:
    std::mt19937_64 engine;
    std::uniform_int_distribution<uint64_t> dist;
};

} // namespace cobra
