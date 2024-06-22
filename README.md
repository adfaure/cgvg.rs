# cgvg.rs

**Rewrite in Rust of [cgvg](https://github.com/uzi/cgvg) using [ripgrep](https://github.com/BurntSushi/ripgrep).**

`cgvg.rs` is a set of two command line tools `cg` (code grep) and `vg` (vi grepped) to quickly search for patterns in a file tree and open them with a text editor.

[![asciicast](https://asciinema.org/a/JxqpMDQbCG0XIdmSt3eIr6xPk.svg)](https://asciinema.org/a/JxqpMDQbCG0XIdmSt3eIr6xPk)
*Sorry for the starship warning, I'll fix that asap :)*


## Command `cg`

`cg` is wrapper around the ripgrep utility command. `cg` forwards it command line arguments to rg, and parses its result to save the matching patterns to be oppened later with `vg`.

## Command `vg`

`vg` takes as argument the index of the last research with `cg` and opens it with your `$EDITOR`.
