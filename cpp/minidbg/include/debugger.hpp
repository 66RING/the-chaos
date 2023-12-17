#ifndef __DEBUGGER_HPP
#define __DEBUGGER_HPP

#include "breakpoint.hpp"
#include "register.hpp"
#include <unordered_map>

class debugger {
public:
  debugger(std::string prog_name, pid_t pid)
      : m_prog_name{std::move(prog_name)}, m_pid{pid} {}

  void run();
  void continue_execution();
  void handle_command(const std::string &line);
  void set_breakpoint_at_address(std::intptr_t addr);
  void dump_registers();
  uint64_t read_memory(uint64_t address);
  void write_memory(uint64_t address, uint64_t value);
  uint64_t get_pc();
  void set_pc(uint64_t pc);
  void step_over_breakpoint();
  void wait_for_signal();

private:
  std::unordered_map<std::intptr_t, breakpoint> m_breakpoints;
  std::string m_prog_name;
  pid_t m_pid;
};

#endif // !__DEBUGGER_HPP
