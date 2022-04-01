# SPWN Language

![spwnnew](https://user-images.githubusercontent.com/85206419/127884996-92251ba7-4c28-4bf0-bb40-d363d5e31ccb.png)

<a href="https://github.com/Spu7Nix/SPWN-language/blob/master/LICENSE">
    <img alt="GitHub license" src="https://img.shields.io/github/license/Spu7Nix/SPWN-language">
</a>
<a href="https://github.com/Spu7Nix/SPWN-language/stargazers">
    <img alt="GitHub stars" src="https://img.shields.io/github/stars/Spu7Nix/SPWN-language">
</a>
<a href="https://github.com/Spu7Nix/SPWN-language">
    <img alt="GitHub all releases" src="https://img.shields.io/github/downloads/spu7nix/SPWN-language/total">
</a>
<a href="https://discord.gg/qKZAhKXqgw">
    <img alt="Discord" src="https://img.shields.io/discord/791323294301290546?label=Discord%20Chat">
</a>
<a href="https://github.com/Spu7Nix/SPWN-language/releases">
    <img alt="GitHub tag (latest by date)" src="https://img.shields.io/github/v/tag/spu7nix/spwn-language?label=Version">
</a>
<a href="https://github.com/Spu7Nix/SPWN-language/issues">
    <img alt="Open issues" src="https://shields.io/github/issues/Spu7nix/SPWN-language">
</a>
<a href="https://github.com/Spu7Nix/SPWN-language/issues">
    <img alt="Pull requests" src="https://shields.io/github/issues-pr/Spu7nix/SPWN-language">
</a>

A language for Geometry Dash triggers. An easy way to create levels using code.

SPWN is a programming language that compiles to Geometry Dash levels. What that means is that you can create levels by using not only the visual representation in the GD-editor, but using a "verbal" and abstracted representation as well. This is especially useful for using GD triggers, which (if you want to make complicated stuff) are not really suited for the graphical workflow of the in-game editor.

| Useful links
|-
| [Official SPWN documentation](https://spu7nix.net/spwn/#/)
| [Documentation Repository](https://github.com/Spu7Nix/spwn_docs)
| [Official SPWN Discord server](https://discord.gg/qKZAhKXqgw)
| [SPWN Playground](https://spu7nix.net/spwn/try/)

## Installing - How To Install

You can either use the installers for your operating system, or build SPWN from source. Please note that building from source will give you access to newer features and bug fixes, but **may be unstable**.

### Universal

#### this method is outdated; use a method described below instead

<s>With Rust installed:
- Run `cargo install spwn`.
- Wait for it to compile.
- Download the [libraries](https://github.com/Spu7Nix/SPWN-language/tree/v0.6-beta/libraries) folder.
- Copy the libraries folder to `$HOME/.cargo/~~bin/spwn`.
- Make something awesome!</s>

If you would like a pre-compiled package, look below.

### Windows

1. Download the .msi file from the [latest release](https://github.com/Spu7Nix/SPWN-language/releases).
2. Open the .msi file and follow the install wizard.

> **Note:** If you get a message telling you that SmartScreen protected your PC, press more info, then run anyways.

### MacOS

1. Download the .pkg file from the [latest release](https://github.com/Spu7Nix/SPWN-language/releases).
2. Open the .pkg file and follow the install wizard.

> **Note:** If you get a message telling you that you cant open files from unidentified developers, open 'System Preferences' then click 'Security & Privacy' and click 'Open Anyway' on the 'General' menu.

### Linux

### Generic (any distro)
- Download `spwn-0.0.8-x86_64.tar.gz` from the [latest release](https://github.com/Spu7Nix/SPWN-language/releases).
- Extract the tarball using `tar -xvf spwn-0.0.8-x86_64.tar.gz`.
- Run SPWN using `./spwn`.
- Optional: add SPWN to path by adding `PATH=$PATH:<where you extracted spwn>` to your `.bashrc` or `.zshrc`.

#### Debian based (Ubuntu, Mint, Elementary, ...)

**No v0.7 or v0.8 installers for Debian based distros have been built yet.**

You can either:

- Download the .deb file from the [latest release](https://github.com/Spu7Nix/SPWN-language/releases) and install it using dpkg with `sudo dpkg -i spwn_0.0.6-0_amd64.deb`.
- Use a one-liner to do this faster: `curl -sLO https://github.com/Spu7Nix/SPWN-language/releases/download/v0.6-beta/spwn_0.6.0-0_amd64.deb && sudo dpkg -i spwn_0.6.0-0_amd64.deb`.
- Use the generic install method.

#### Arch based (Manjaro, Artix, ...)

- Install the [arch package](https://github.com/Spu7Nix/SPWN-language/releases) using pacman:

```cmd
pacman -U spwn-0.0.8-x86_64-linux_arch.pkg.tar.zst
```

- Alternatively, you can install the SPWN binary from the AUR (replace yay with your helper of choice):

```cmd
yay -S spwn-bin
```

#### Android

##### Requirements

- Rooted device
- Termux, or something similar
- Rust

Once you have these, run:

```cmd
cargo install spwn
```

Let it compile and you're good to go.

### Compiling from source

1. [Download](https://github.com/Spu7Nix/SPWN-language/archive/refs/heads/master.zip) the source code from this repository.
2. Unzip the .zip file.
3. Install rust (if you haven't already).
4. Open the unzipped folder in the terminal by
- Windows: Right click the folder and press `Open command window here`.
- Mac: Open `Terminal.app` in the `Utilities` folder in the `Applications` folder, then drag and drop the folder onto the terminal window and press enter.
- Linux: Right click the folder and press `Open in Terminal`.
6. Run `cargo build --release`.
7. Compiled binary is placed in the `target/release` directory.

## Using SPWN

### SPWN Playground

SPWN Playground is a SPWN compiler built into your browser. You can use it to [try out SPWN](https://spu7nix.net/spwn/try/) before you decide to install it.

### Setup

Alright, enough talk, how do we actually use SPWN?

SPWN code can be programmed in any code editor, but the ones that have had SPWN extensions or plugins written for them are [Visual Studio Code](https://code.visualstudio.com/), [Sublime Text](https://www.sublimetext.com/) and [Vim](https://www.vim.org/).

#### VSCode

Navigate to [VSCode SPWN language support](https://marketplace.visualstudio.com/items?itemName=Spu7Nix.spwn-language-support) and hit install. In VSCode, hit enable and then create a new file with the extension .spwn.

> **Note:** Make sure to have the file in the same directory as the libraries folder.

VSCode should automatically change the language syntax to SPWN, but if it doesn't, navigate to the bottom right of the screen and click on `select language mode`, then select SPWN.

#### Sublime Text

Open Sublime Text and open the Command Palette... by selecting Command Palette from the Tools pull-down menu. In the menu that opens type install which will result in the Install Package Control option being presented. Hit Enter or left click the entry to install Package Control. Open the Command Palette again, and type 'install'. When `Package Control: Install Package` is highlighted press 'Enter' then type 'SPWN Language' and press 'Enter' when `SPWN Language` is highlighted.

#### Vim

Go to [spwn().vim!](https://gitlab.com/verticallity/spwn-vim) and follow to instructions on that page.

#### Other Editors

For any other editor with syntax highlighting, most C type syntax highlighting schemes work fine.

### How to run SPWN Code

Head to the [docs](https://spu7nix.net/spwn/#/) to create a simple program, such as the one below

```spwn
test = 5g
-> test.move(5, 100, 0.25)
test.move(10, -10, 2)
```

Save the file, then open a terminal and type in `spwn build YOURFILENAME.spwn`. Make sure to have GD closed during this process. After running this command, open GD, and the levels content will be modified. Head over to the docs to learn how to program in SPWN.

> **Note:** SPWN generates triggers near the top of your level, so you might not see any difference.

### Command Line Reference

Here is a list of SPWN command line subcommands and flags. This information can be found by typing `spwn help` in the command line as well.

### Subcommands

```
build    
    Runs/builds a given file [aliases: b]

doc      
    Generates documentation for a SPWN library, in the form of a markdown file

eval     
    Runs/builds the input given in stdin/the console as SPWN code [aliases: b]

help  
    Print this message or the help of the given subcommand(s)
```

### Build Flags

```
-a, --allow <allow>...
    Allow the use of a builtin

-c, --console-output
    Makes the script print the created level into the console instead of writing it to your
    save file

-d, --deny <deny>...
    Deny the use of a builtin

-e, --live-editor
    Instead of writing the level to the save file, the script will use a live editor library
    if it's installed (Currently works only for MacOS)

-h, --help
    Print help information

-i, --include-path <include-path>...
    Adds a search path to look for librariesAdds a search path to look for libraries

-l, --no-level
    Only compiles the script, no level creation at all

-n, --level-name <NAME>...
    Targets a specific level

-o, --no-optimize
    Removes post-optimization of triggers, making the output more readable, while also using
    a lot more objects and groups

-s, --save-file <FILE>...
    Chooses a specific save file to write to
```

### Examples

`spwn build addition.spwn --level-name add`
Build a file called addition.spwn and write it to the level named add.

`spwn build subtraction.spwn --no-level`
Build a file called subtraction.spwn but don't write to a level.

`spwn build AI.spwn -c`
Build a file called AI.spwn and output the level string to the console.

## Todo before release

- [x] Finish mutable variables
- [x] Type annotations for function arguments, variable definitions etc.
- [x] `as` operator for automatically changing type
- [ ] Finish documentation
- [x] Break/continue statement
- [x] Operation order
- [x] Escaped characters in string
- [x] Fix post-compile optimizations

| Enjoy SPWN!
|-
