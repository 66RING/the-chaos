#include <cstdint>
#include <iostream>
#include <sstream>
#include <sys/ptrace.h>
#include <sys/wait.h>
#include <sys/personality.h>
#include <unistd.h>
#include <unordered_map>
#include <vector>

#include "linenoise.h"

class breakpoint;

// TODO: review helper function
std::vector<std::string> split(const std::string &s, char delimiter) {
  std::vector<std::string> out{};
  std::stringstream ss{s};
  std::string item;

  while (std::getline(ss, item, delimiter)) {
    out.push_back(item);
  }

  return out;
}

// TODO: review helper function
bool is_prefix(const std::string &s, const std::string &of) {
  if (s.size() > of.size())
    return false;
  return std::equal(s.begin(), s.end(), of.begin());
}

class debugger {
public:
  debugger(std::string prog_name, pid_t pid)
      : m_prog_name{std::move(prog_name)}, m_pid{pid} {}

  void run();
  void continue_execution();
  void handle_command(const std::string &line);
  void set_breakpoint_at_address(std::intptr_t addr);

private:
  std::unordered_map<std::intptr_t, breakpoint> m_breakpoints;
  std::string m_prog_name;
  pid_t m_pid;
};

void debugger::run() {
  int wait_status;
  auto options = 0;
  // Wait until the child process has finished launching.
  // When the traced process is launched, it will be sent a SIGTRAP signal,
  // which is a trace or breakpoint trap. We can wait until this signal is sent
  // using the waitpid function
  waitpid(m_pid, &wait_status, options);

  char *line = nullptr;
  while ((line = linenoise("minidbg> ")) != nullptr) {
    handle_command(line);
    linenoiseHistoryAdd(line);
    linenoiseFree(line);
  }
}

void debugger::handle_command(const std::string& line) {
    auto args = split(line,' ');
    auto command = args[0];

    if (is_prefix(command, "cont")) {
        continue_execution();
    }
    else if(is_prefix(command, "break")) {
        std::string addr {args[1], 2}; //naively assume that the user has written 0xADDRESS
        set_breakpoint_at_address(std::stol(addr, 0, 16));
    }
    else {
        std::cerr << "Unknown command\n";
    }
}

void debugger::continue_execution() {
  // Use ptrace to tell the process to continue, then waitpid until it’s
  // signalled
  ptrace(PTRACE_CONT, m_pid, nullptr, nullptr);

  int wait_status;
  auto options = 0;
  waitpid(m_pid, &wait_status, options);
}

class breakpoint {
public:
  breakpoint() = default;
  breakpoint(pid_t pid, std::intptr_t addr)
      : m_pid{pid}, m_addr{addr}, m_enabled{false}, m_saved_data{} {}

  void enable();
  void disable();

  auto is_enabled() const -> bool { return m_enabled; }
  auto get_address() const -> std::intptr_t { return m_addr; }

private:
  pid_t m_pid;
  std::intptr_t m_addr;
  bool m_enabled;
  uint8_t m_saved_data; // data which used to be at the breakpoint address
};

void breakpoint::enable() {
  auto data = ptrace(PTRACE_PEEKDATA, m_pid, m_addr, nullptr);
  m_saved_data = static_cast<uint8_t>(data & 0xff); // save bottom byte
  uint64_t int3 = 0xcc;
  // NOTE:
  uint64_t data_with_int3 = ((data & ~0xff) | int3); // set bottom byte to 0xcc
  ptrace(PTRACE_POKEDATA, m_pid, m_addr, data_with_int3);

  m_enabled = true;
}

void breakpoint::disable() {
  auto data = ptrace(PTRACE_PEEKDATA, m_pid, m_addr, nullptr);
  auto restored_data = ((data & ~0xff) | m_saved_data);
  ptrace(PTRACE_POKEDATA, m_pid, m_addr, restored_data);

  m_enabled = false;
}

void debugger::set_breakpoint_at_address(std::intptr_t addr) {
    std::cout << "Set breakpoint at address 0x" << std::hex << addr << std::endl;
    breakpoint bp {m_pid, addr};
    bp.enable();
    m_breakpoints[addr] = bp;
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    std::cerr << "Program name not specified";
    return -1;
  }

  auto prog = argv[1];

  auto pid = fork();
  if (pid == 0) {
    // we're in the child process

    // PTRACE_TRACEME allow parent process to trace this process.
    ptrace(PTRACE_TRACEME, 0, nullptr, nullptr);
    personality(ADDR_NO_RANDOMIZE);
    // execute debugee.
    execl(prog, prog, nullptr);
  } else if (pid >= 1) {
    // we're in the parent process
    // execute debugger
    // a loop for listening to user input.
    std::cout << "Started debugging process " << pid << '\n';
    debugger dbg{prog, pid};
    dbg.run();
  }
}
