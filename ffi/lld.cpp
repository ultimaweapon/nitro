#include "nitro.hpp"

#include <lld/Common/Driver.h>

#include <string>
#include <vector>

LLD_HAS_DRIVER(coff)
LLD_HAS_DRIVER(elf)
LLD_HAS_DRIVER(macho)

extern "C" bool lld_link(const char *flavor, const char *args[], nitro_string &err)
{
    // Setup arguments.
    std::vector<const char *> vec{flavor};

    for (auto i = 0; args[i]; i++) {
        vec.push_back(args[i]);
    }

    // Run LLD.
    llvm::raw_null_ostream os;
    std::string buf;
    llvm::raw_string_ostream es(buf);
    auto res = lld::lldMain(vec, os, es, {
        { lld::Darwin, &lld::macho::link },
        { lld::Gnu, &lld::elf::link },
        { lld::WinLink, &lld::coff::link }
    });

    if (res.retCode) {
        nitro_string_set(err, buf.c_str());
        return false;
    } else {
        return true;
    }
}
