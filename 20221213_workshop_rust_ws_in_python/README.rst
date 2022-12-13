ASYNCIO PYTHON MODULES IN RUST
==============================

Duration: 40min

Speaker: Ondřej Vostal (Founder & CEO at ChainKeepers, www.chainkeepers.io, https://twitter.com/0xkron)

Slides: https://chainkeepers.github.io/talks/20221213_workshop_rust_ws_in_python/talk

Annotation:

I'll demonstrate how to write a Python module in Rust. What's more, the module will communicate both ways between Python and Rust using asyncio and tokio. After the talk, you'll be able to write an app that asynchronously processes data in Rust without holding the GIL. The main advantage of this approach as opposed to microservices is less boilerplate and saving the IPC overhead.


INSTALL
=======

Rust is easy to setup once you know the tools.

Install rustup

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Install toolchain

    rustup update

Make sure you use rustups versions...

    which cargo

Should point to

    ~/.cargo/bin/cargo

Make rustup use the newest toolchain

    rustup override stable

Now you're ready to build and use.


USAGE
=====

Checkout the repo and change dir to the source root.

Create log dir

    mkdir log

Inspect `run`.  Then

    run 600 20

Later analyze data using

    python analyze/src/main.py
