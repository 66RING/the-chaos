#ifndef __DEBUGGER_HPP
#define __DEBUGGER_HPP

#include <fcntl.h>
#include <linux/types.h>
#include <sstream>
#include <string>
#include <sys/stat.h>
#include <unordered_map>
#include <utility>

#include "breakpoint.hpp"
#include "register.hpp"
#include "symbol.hpp"

#include "dwarf/dwarf++.hh"
#include "elf/elf++.hh"

class debugger {
public:
  debugger(std::string prog_name, pid_t pid)
      : m_prog_name{std::move(prog_name)}, m_pid{pid} {
    auto fd = open(m_prog_name.c_str(), O_RDONLY);

    m_elf = elf::elf{elf::create_mmap_loader(fd)};
    m_dwarf = dwarf::dwarf{dwarf::elf::create_loader(m_elf)};
  }

  void run();
  void continue_execution();
  void handle_command(const std::string &line);
  void set_breakpoint_at_address(std::intptr_t addr);
  void set_breakpoint_at_function(const std::string& name);
  void set_breakpoint_at_source_line(const std::string& file, unsigned line);
  void remove_breakpoint(std::intptr_t addr);
  void dump_registers();
  uint64_t read_memory(uint64_t address);
  void write_memory(uint64_t address, uint64_t value);
  uint64_t get_pc();
  uint64_t get_offset_pc();
  void set_pc(uint64_t pc);
  void step_over_breakpoint();
  void step_out();
  void step_in();
  void step_over();
  void single_step_instruction();
  void single_step_instruction_with_breakpoint_check();
  void wait_for_signal();
  // loop through the sections of the ELF looking for symbol tables
  std::vector<symbol> lookup_symbol(const std::string& name);

  uint64_t offset_load_address(uint64_t addr);
  // offset addresses from DWARF info
  uint64_t offset_dwarf_address(uint64_t addr);
  void initialise_load_address();
  void print_source(const std::string &file_name, unsigned line,
                    unsigned n_lines_context = 2);

  void handle_sigtrap(siginfo_t info);
  siginfo_t get_signal_info();
  dwarf::die get_function_from_pc(uint64_t pc);
  dwarf::line_table::iterator get_line_entry_from_pc(uint64_t pc);

private:
  std::unordered_map<std::intptr_t, breakpoint> m_breakpoints;
  std::string m_prog_name;
  pid_t m_pid;
  dwarf::dwarf m_dwarf;
  elf::elf m_elf;
  uint64_t m_load_address;
};


#endif // !__DEBUGGER_HPP
