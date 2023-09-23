#include "minilog.h"

int main() {
    int my_variable = 42;
    MINILOG_P(my_variable);

    minilog::log_trace("below is the color show :)");
#define _PATTERN(name) minilog::log_##name(#name);
    MINILOG_FOR_EACH(_PATTERN)
#undef _PATTERN
}
