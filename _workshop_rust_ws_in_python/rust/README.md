===

Install rustup

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
