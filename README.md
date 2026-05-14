# lc3tools-rs

This is a simple reimplementation of the LC3 Simulator with some extra features and prettier Ratatui debugger :).

# Dependencies

- `rust`
- `cargo`

There are pretty much everything you need. It is probably best if you download rust and cargo with rustup from [the rust website](https://rust-lang.org/)

# Installation

```
cargo install https://github.com/MyUsernamee/lc3tools-rs.git
```

# Usage

```
lc3sim [OPTIONS] [OBJ_FILES]...

Arguments:
  [OBJ_FILES]...  Obj files to load. Program counter will be set to the orig of the last obj file

Options:
      --no-repl                  Don't open repl even on execeptions
  -b, --breakpoint <BREAKPOINT>  Unless no-repl is provided, add a breakpoint at the given address, open a repl when
 hit, otherwise does nothing
      --verbose                  
  -h, --help                     Print help
```
