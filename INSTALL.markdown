To install PlotTwist on your system you need Rust and Cargo (not sure which version).

```bash
git clone https://github.com/michcioperz/plottwist-rs
cd plottwist-rs
cargo install --features ncurses
# it will install to ~/.cargo/bin most probably
```

It is advisable to create somewhere in your path a script called `plottwist` with something like this:
```bash
#!/bin/sh
urxvt -e plottwist-series
```
or otherwise alias plottwist to something like this

PlotTwist is best enjoyed with a tiling window manager.
