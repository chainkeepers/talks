===

Install rustup

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Install toolchain

    rustup update

Make sure you use rustups versions...

    which cargo

Should point to

    ~/.cargo/bin/cargo

Switch to project directory

    cd PREFIX

Make rustup use the newest toolchain

    rustup override stable

Install dependencies

    make prepare

Build it

    make build
