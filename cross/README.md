# cross

This crate depends on the `logic` crate,
which contains the shared business logic.
The split allows `logic` to be tested natively
on the host while this crate handles all
hardware-specific concerns such as peripherals,
interrupts, and chip initialization.
