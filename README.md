# femto

nano? pico? femto!

A terminal based text editor written in rust in 300 LOC and 300 Kb of statically
linked (stripped) binary.

<p align="center">
       <img src="https://raw.githubusercontent.com/frapa/femto/master/femto.gif" alt="femto video">
</p>

The editor is meant to showcase a minimal terminal text editor written
in the fewer number of lines possible. To achieve this goal, it was necessary
to take a couple of hack in some lines, but the rest of code, especially
the program structure, is written to be easily extensible.

## Proof

```bash
$ cloc src
       1 text file.
       1 unique file.
       0 files ignored.

github.com/AlDanial/cloc v 1.74  T=0.01 s (116.8 files/s, 41818.1 lines/s)
-------------------------------------------------------------------------------
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                             1             52              6            300
-------------------------------------------------------------------------------
```

## Features

- Be correct (no bugs, weird UX to save lines, etc)
- Quit editor (ctrl+q).
- Open and save files (ctrl+o, crtl+s).
- Remember saved path.
- Open file passing argument from command line.
- Edit file just like text editor, with unicode support.
- Navigation with arrow keys, home and end.
- Buffers are scollable to reveal full content if larger the screen size.
- Inverted colors status bar with row and col number.
- Report errors in status bar.
- Tabs automatically converted to 4 spaces (finally end the discussion).
- Should be multiplatform, if a compatible terminal is available (in practice tested only on linux).

## Non-features

- Minimum amount of lines and maximum simplicity.
- Ask confirmation when exiting with modification.
- Path autocomplete.
- Configure tabs/spaces.
- Search/replace.
- Syntax highlighting.
- Multiple buffers.
- Autocomplete.

## Install & Try it Out

Linux pre-compiled and stripped binary:

```bash
EXEC='/usr/local/bin/femto' && sudo wget https://raw.githubusercontent.com/frapa/femto/master/femto -O $EXEC && sudo chmod +x $EXEC
```

## Contributing

Please feel free to send a pull request if you see any problem or you see a way to reduce the line count. This is just a challenge to write a simple terminal text editor in the simplest possible way, so we're not looking into adding big new functonality, but bugs should be solved and annoying behavior fixed.
