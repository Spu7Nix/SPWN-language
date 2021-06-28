# SPWN-language

A language for Geometry Dash triggers. An easy way to create levels using code.

SPWN is a programming language that compiles to Geometry Dash levels. What that means is that you can create levels by using not only the visual representation in the GD-editor, but using a "verbal" and abstracted representation as well. This is especially useful for using GD triggers, which (if you want to make complicated stuff) are not really suited for the graphical workflow of the in-game editor.


The documentation for the SPWN language is located [here](https://spu7nix.net/spwn/#/). You can also [contribute to the docs here](https://github.com/Spu7Nix/spwn_docs). If you have questions, comments, need help, or want to share your work, [join the discord server here](https://discord.gg/qKZAhKXqgw).

## Installing - How To Install

###### Windows
1. Download the .msi file from the [latest release](https://github.com/Spu7Nix/SPWN-language/releases).
2. Open the .msi file and follow the install wizard.

###### MacOS
1. Download the .pkg file from the [latest release](https://github.com/Spu7Nix/SPWN-language/releases).
2. Open the .pkg file and follow the install wizard.

 - Note: *If you get a message telling you that you cant open files from unidentified developers, control click the .pkg file and click open*

###### Linux
*Coming soon.*

###### Compiling from source
1. Download source code from this repository.
2. Install rust if you haven't already.
3. Open `spwn-lang` folder in the terminal.
4. Run `cargo build`.
5. Compiled binary is placed in `target/debug`.



## Todo before release:

- [x] Finish mutable variables
- [x] Type annotations for function arguments, variable definitions etc.
- [x] `as` operator for automatically changing type
- [ ] finish documentation
- [x] break/continue statement
- [x] operation order
- [x] escaped characters in string
- [x] fix post-compile optimizations

## Todo at some point

- [x] implement live editor features for windows
- [ ] make it work on linux
- [x] nested comments
- [x] get and edit obj and trigger properties

# Enjoy SPWN!
