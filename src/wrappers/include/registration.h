#pragma once

#include <iostream>
#include <cstdint>

using HandlerFn = void (*)(uint32_t);

void register_message_handler(HandlerFn handler) {
    if (handler) {
        handler(42);
    } else {
        std::cerr << "Error: handler is null" << std::endl;
    }
}