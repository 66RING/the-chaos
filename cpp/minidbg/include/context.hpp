#ifndef __CONTEXT_HPP
#define __CONTEXT_HPP

#include <fcntl.h>
#include <linux/types.h>
#include <sstream>
#include <string>
#include <sys/stat.h>
#include <unordered_map>
#include <utility>

#include "dwarf/dwarf++.hh"
#include "elf/elf++.hh"


class ptrace_expr_context : public dwarf::expr_context {
public:
    ptrace_expr_context (pid_t pid) : m_pid{pid} {}

    dwarf::taddr reg (unsigned regnum) override {
        return get_register_value_from_dwarf_register(m_pid, regnum);
    }

    dwarf::taddr pc() override {
        struct user_regs_struct regs;
        ptrace(PTRACE_GETREGS, m_pid, nullptr, &regs);
        return regs.rip;
    }

    dwarf::taddr deref_size (dwarf::taddr address, unsigned size) override {
        //TODO take into account size
        return ptrace(PTRACE_PEEKDATA, m_pid, address, nullptr);
    }

private:
    pid_t m_pid;
};

#endif
