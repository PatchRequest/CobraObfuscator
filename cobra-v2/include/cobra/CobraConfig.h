#pragma once
#include <cstdint>
#include <string>
#include <vector>

namespace cobra {

struct CobraConfig {
    uint64_t seed = 0;
    int iterations = 1;
    std::vector<std::string> enabledPasses;   // empty = all
    std::vector<std::string> excludedPasses;
    bool verbose = false;
    bool stats = false;

    bool isPassEnabled(const std::string &name) const {
        if (!excludedPasses.empty()) {
            for (auto &e : excludedPasses)
                if (e == name) return false;
        }
        if (enabledPasses.empty()) return true;
        for (auto &p : enabledPasses)
            if (p == name) return true;
        return false;
    }
};

} // namespace cobra
