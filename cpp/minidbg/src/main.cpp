#include <cstdint>
#include <iomanip>
#include <iostream>
#include <fstream>
#include <sys/personality.h>
#include <sys/ptrace.h>
#include <sys/wait.h>
#include <unistd.h>
#include <unordered_map>
#include <vector>

#include "debugger.hpp"
#include "linenoise.h"

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

// you can type c.cpp for a/b/c.cpp
bool is_suffix(const std::string& s, const std::string& of) {
    if (s.size() > of.size()) return false;
    auto diff = of.size() - s.size();
    return std::equal(s.begin(), s.end(), of.begin() + diff);
}

void debugger::run() {
  // Wait until the child process has finished launching.
  // When the traced process is launched, it will be sent a SIGTRAP signal,
  // which is a trace or breakpoint trap. We can wait until this signal is sent
  // using the waitpid function
  wait_for_signal();
  // TODO: review
  initialise_load_address();

  char *line = nullptr;
  // TODO: review linenoise lib
  while ((line = linenoise("minidbg> ")) != nullptr) {
    handle_command(line);
    linenoiseHistoryAdd(line);
    linenoiseFree(line);
  }
}

uint64_t debugger::offset_load_address(uint64_t addr) {
   return addr - m_load_address;
}

void debugger::initialise_load_address() {
   //If this is a dynamic library (e.g. PIE)
   if (m_elf.get_hdr().type == elf::et::dyn) {
      //The load address is found in /proc/&lt;pid&gt;/maps
      std::ifstream map("/proc/" + std::to_string(m_pid) + "/maps");

      //Read the first address from the file
      std::string addr;
      std::getline(map, addr, '-');

      m_load_address = std::stol(addr, 0, 16);
   }
}

siginfo_t debugger::get_signal_info() {
    siginfo_t info;
    ptrace(PTRACE_GETSIGINFO, m_pid, nullptr, &info);
    return info;
}

// TODO: review
void debugger::print_source(const std::string& file_name, unsigned line, unsigned n_lines_context) {
    std::ifstream file {file_name};

    //Work out a window around the desired line
    auto start_line = line <= n_lines_context ? 1 : line - n_lines_context;
    auto end_line = line + n_lines_context + (line < n_lines_context ? n_lines_context - line : 0) + 1;

    char c{};
    auto current_line = 1u;
    //Skip lines up until start_line
    while (current_line != start_line && file.get(c)) {
        if (c == '\n') {
            ++current_line;
        }
    }

    //Output cursor if we're at the current line
    std::cout << (current_line==line ? "> " : "  ");

    //Write lines up until end_line
    while (current_line <= end_line && file.get(c)) {
        std::cout << c;
        if (c == '\n') {
            ++current_line;
            //Output cursor if we're at the current line
            std::cout << (current_line==line ? "> " : "  ");
        }
    }

    //Write newline and make sure that the stream is flushed properly
    std::cout << std::endl;
}

// TODO: review
void debugger::dump_registers() {
  for (const auto &rd : g_register_descriptors) {
    std::cout << rd.name << " 0x" << std::setfill('0') << std::setw(16)
              << std::hex << get_register_value(m_pid, rd.r) << std::endl;
  }
}

void debugger::handle_command(const std::string &line) {
  auto args = split(line, ' ');
  auto command = args[0];

  if (is_prefix(command, "cont")) {
    continue_execution();
  } else if (is_prefix(command, "break")) {
    if (args[1][0] == '0' && args[1][1] == 'x') {
      // 0x<hexadecimal> -> address breakpoint
      std::string addr {args[1], 2};
      set_breakpoint_at_address(std::stol(addr, 0, 16));
    } else if (args[1].find(':') != std::string::npos) {
      // <line>:<filename> -> line number breakpoint
      auto file_and_line = split(args[1], ':');
      set_breakpoint_at_source_line(file_and_line[0], std::stoi(file_and_line[1]));
    } else {
      // <anything else> -> function name breakpoint
      set_breakpoint_at_function(args[1]);
    }
  } else if (is_prefix(command, "register")) {
    if (is_prefix(args[1], "dump")) {
      dump_registers();
    } else if (is_prefix(args[1], "read")) {
      std::cout << get_register_value(m_pid, get_register_from_name(args[2]))
                << std::endl;
    } else if (is_prefix(args[1], "write")) {
      std::string val{args[3], 2}; // assume 0xVAL
      set_register_value(m_pid, get_register_from_name(args[2]),
                         std::stol(val, 0, 16));
    }
  } else if (is_prefix(command, "memory")) {
    std::string addr{args[2], 2}; // assume 0xADDRESS

    if (is_prefix(args[1], "read")) {
      std::cout << std::hex << read_memory(std::stol(addr, 0, 16)) << std::endl;
    }
    if (is_prefix(args[1], "write")) {
      std::string val{args[3], 2}; // assume 0xVAL
      write_memory(std::stol(addr, 0, 16), std::stol(val, 0, 16));
    }
  } else if(is_prefix(command, "stepi")) {
    single_step_instruction_with_breakpoint_check();
    auto line_entry = get_line_entry_from_pc(get_pc());
    print_source(line_entry->file->path, line_entry->line);
  } else if(is_prefix(command, "step")) {
    step_in();
  } else if(is_prefix(command, "next")) {
    step_over();
  } else if(is_prefix(command, "finish")) {
    step_out();
  } else if(is_prefix(command, "symbol")) {
    // print symbol type
    auto syms = lookup_symbol(args[1]);
    for (auto&& s : syms) {
        std::cout << s.name << ' ' << to_string(s.type) << " 0x" << std::hex << s.addr << std::endl;
    }
  } else {
    std::cerr << "Unknown command\n";
  }
}

// loop through the sections of the ELF looking for symbol tables
std::vector<symbol> debugger::lookup_symbol(const std::string& name) {
    // collect any symbols found
    std::vector<symbol> syms;

    for (auto &sec : m_elf.sections()) {
        if (sec.get_hdr().type != elf::sht::symtab && sec.get_hdr().type != elf::sht::dynsym)
            continue;

        for (auto sym : sec.as_symtab()) {
            if (sym.get_name() == name) {
                auto &d = sym.get_data();
                syms.push_back(symbol{to_symbol_type(d.type()), sym.get_name(), d.value});
            }
        }
    }

    return syms;
}

void debugger::set_breakpoint_at_source_line(const std::string& file, unsigned line) {
    for (const auto& cu : m_dwarf.compilation_units()) {
        if (is_suffix(file, at_name(cu.root()))) {
            const auto& lt = cu.get_line_table();

            for (const auto& entry : lt) {
                if (entry.is_stmt && entry.line == line) {
                    set_breakpoint_at_address(offset_dwarf_address(entry.address));
                    return;
                }
            }
        }
    }
}

// match against DW_AT_name and use 
// DW_AT_low_pc (the start address of the function) 
// to set our breakpoint
void debugger::set_breakpoint_at_function(const std::string& name) {
    for (const auto& cu : m_dwarf.compilation_units()) {
        for (const auto& die : cu.root()) {
            if (die.has(dwarf::DW_AT::name) && at_name(die) == name) {
                auto low_pc = at_low_pc(die);
                auto entry = get_line_entry_from_pc(low_pc);
                // the DW_AT_low_pc for a function doesn’t point at the start of the user code for that function, it points to the start of the prologue
                ++entry; //skip prologue
                set_breakpoint_at_address(offset_dwarf_address(entry->address));
            }
        }
    }
}

void debugger::step_over_breakpoint() {
    if (m_breakpoints.count(get_pc())) {
        auto& bp = m_breakpoints[get_pc()];
        if (bp.is_enabled()) {
            bp.disable();
            ptrace(PTRACE_SINGLESTEP, m_pid, nullptr, nullptr);
            wait_for_signal();
            bp.enable();
        }
    }
}

void debugger::single_step_instruction() {
    ptrace(PTRACE_SINGLESTEP, m_pid, nullptr, nullptr);
    wait_for_signal();
}

void debugger::single_step_instruction_with_breakpoint_check() {
    //first, check to see if we need to disable and enable a breakpoint
    if (m_breakpoints.count(get_pc())) {
        step_over_breakpoint();
    }
    else {
        single_step_instruction();
    }
}

void debugger::step_in() {
   auto line = get_line_entry_from_pc(get_offset_pc())->line;

   while (get_line_entry_from_pc(get_offset_pc())->line == line) {
      single_step_instruction_with_breakpoint_check();
   }

   auto line_entry = get_line_entry_from_pc(get_offset_pc());
   print_source(line_entry->file->path, line_entry->line);
}

uint64_t debugger::get_offset_pc() {
   return offset_load_address(get_pc());
}

// offset addresses from DWARF info
uint64_t debugger::offset_dwarf_address(uint64_t addr) {
   return addr + m_load_address;
}

void debugger::step_over() {
    auto func = get_function_from_pc(get_offset_pc());
    // at_low_pc and at_high_pc are functions from libelfin
    // which will get us the low and high PC values 
    // for the given function DIE.
    auto func_entry = at_low_pc(func);
    auto func_end = at_high_pc(func);

    auto line = get_line_entry_from_pc(func_entry);
    auto start_line = get_line_entry_from_pc(get_offset_pc());

    // remove any breakpoints we set so that they don’t leak out of our step function
    std::vector<std::intptr_t> to_delete{};

    // set a breakpoint at every line in the current function
    while (line->address < func_end) {
        auto load_address = offset_dwarf_address(line->address);
        if (line->address != start_line->address && !m_breakpoints.count(load_address)) {
            set_breakpoint_at_address(load_address);
            to_delete.push_back(load_address);
        }
        ++line;
    }

    // setting a breakpoint on the return address of the function
    auto frame_pointer = get_register_value(m_pid, reg::rbp);
    auto return_address = read_memory(frame_pointer+8);
    if (!m_breakpoints.count(return_address)) {
        set_breakpoint_at_address(return_address);
        to_delete.push_back(return_address);
    }

    continue_execution();

    for (auto addr : to_delete) {
        remove_breakpoint(addr);
    }
}

void debugger::step_out() {
    // set a breakpoint at the return address of the function and continue
    auto frame_pointer = get_register_value(m_pid, reg::rbp);
    auto return_address = read_memory(frame_pointer+8);

    bool should_remove_breakpoint = false;
    if (!m_breakpoints.count(return_address)) {
        // breakpoint doesn't exist, create it and remove it later
        set_breakpoint_at_address(return_address);
        should_remove_breakpoint = true;
    }

    continue_execution();

    // remove breakpoint if bp is set by step_out
    if (should_remove_breakpoint) {
        remove_breakpoint(return_address);
    }
}

void debugger::remove_breakpoint(std::intptr_t addr) {
    if (m_breakpoints.at(addr).is_enabled()) {
        m_breakpoints.at(addr).disable();
    }
    m_breakpoints.erase(addr);
}

void debugger::wait_for_signal() {
    int wait_status;
    auto options = 0;
    waitpid(m_pid, &wait_status, options);

    auto siginfo = get_signal_info();

    switch (siginfo.si_signo) {
    case SIGTRAP:
        handle_sigtrap(siginfo);
        break;
    case SIGSEGV:
        std::cout << "Yay, segfault. Reason: " << siginfo.si_code << std::endl;
        break;
    default:
        std::cout << "Got signal " << strsignal(siginfo.si_signo) << std::endl;
    }
}

void debugger::handle_sigtrap(siginfo_t info) {
    switch (info.si_code) {
    //one of these will be set if a breakpoint was hit
    case SI_KERNEL:
    case TRAP_BRKPT:
    {
        set_pc(get_pc()-1); //put the pc back where it should be
        std::cout << "Hit breakpoint at address 0x" << std::hex << get_pc() << std::endl;
        auto offset_pc = offset_load_address(get_pc()); //rember to offset the pc for querying DWARF
        auto line_entry = get_line_entry_from_pc(offset_pc);
        print_source(line_entry->file->path, line_entry->line);
        return;
    }
    //this will be set if the signal was sent by single stepping
    case TRAP_TRACE:
        return;
    default:
        std::cout << "Unknown SIGTRAP code " << info.si_code << std::endl;
        return;
    }
}

uint64_t debugger::get_pc() { return get_register_value(m_pid, reg::rip); }

void debugger::set_pc(uint64_t pc) { set_register_value(m_pid, reg::rip, pc); }

uint64_t debugger::read_memory(uint64_t address) {
  return ptrace(PTRACE_PEEKDATA, m_pid, address, nullptr);
}

void debugger::write_memory(uint64_t address, uint64_t value) {
  ptrace(PTRACE_POKEDATA, m_pid, address, value);
}

void debugger::continue_execution() {
  // Use ptrace to tell the process to continue, then waitpid until it’s
  // signalled
  step_over_breakpoint();
  ptrace(PTRACE_CONT, m_pid, nullptr, nullptr);
  wait_for_signal();
}

void debugger::set_breakpoint_at_address(std::intptr_t addr) {
  std::cout << "Set breakpoint at address 0x" << std::hex << addr << std::endl;
  breakpoint bp{m_pid, addr};
  bp.enable();
  m_breakpoints[addr] = bp;
}

dwarf::die debugger::get_function_from_pc(uint64_t pc) {
  for (auto &cu : m_dwarf.compilation_units()) {
    if (die_pc_range(cu.root()).contains(pc)) {
      for (const auto &die : cu.root()) {
        if (die.tag == dwarf::DW_TAG::subprogram) {
          if (die_pc_range(die).contains(pc)) {
            return die;
          }
        }
      }
    }
  }

  throw std::out_of_range{"Cannot find function"};
}

dwarf::line_table::iterator debugger::get_line_entry_from_pc(uint64_t pc) {
  for (auto &cu : m_dwarf.compilation_units()) {
    if (die_pc_range(cu.root()).contains(pc)) {
      auto &lt = cu.get_line_table();
      auto it = lt.find_address(pc);
      if (it == lt.end()) {
        throw std::out_of_range{"Cannot find line entry"};
      } else {
        return it;
      }
    }
  }

  throw std::out_of_range{"Cannot find line entry"};
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
