# lines_editor

Lines is a minimal text editor.

```
quit sav re,undo del|nrm ins vis hex raw|pasty cvy|wrd,b,end ///cmnt []idnt hjkl
1 # lines_editor
2
3 Lines is a minimal text editor.
4
5
6
7
8
9
10
11
12
13
14
15
16
17
18
19
21
NORMAL 1:1 shorty.txt @0  > ⎕
```


# Abstract
Lines is a minimal terminal text/hex file-editor, written from scratch in vanilla 2024-Rust (with no third party crates or unsafe code), designed for long term reliability and maintainability, adaptable modularity, memory-footprint minimalism, safety, and security. The scope is intentionally constrained to a few essential uses and file operations such as insert, and delete at the character/byte level. Lines is, by design and policy, not a "fully-featured," "feature-rich," "responsive," full IDE competing with Zed, Helix, vsCode, etc.


# Common Use Cases
In 2025 these are all common, yet there is no good way to deal with these.

- opening a data file to see the header-line wrapped to see all the column names

- opening an 8gb data file to inspect the header and top row (through a window). Data files can have hundreds or thousands of columns, are you going to 'head' that in a terminal? (very common in data science)

- looking at the bytes of a file because there is a file encoding issue or file damage problem (very common in data science)

- looking at a plain text file quickly and simply (very common)

- looking at a large plain text file (very common)

- having a lite-weight module for another cli application to be able to simply view a file natively within that application (very common)

- looking at a file where the formatting is such that "rows" are long (or there are no "rows").

- the need to make and save a memo file extremely simply and quickly (the original Lines-editor does this well)


## In Scope:
- modular structure (lines IS a module)
- Opening files (creating new, opening existing)
- Viewing file contents in a "sliding window" (80×24 default, up to 320×96)
- Navigation (hjkl, word boundaries, goto line)
- Basic editing (insert characters, delete characters)
- Line operations (delete line, comment/uncomment)
- Save operations (save)
- File safety (read-copies, timestamped archives, hex-edit in place)
- Visual mode (selection)
- Line numbers (absolute/relative)
~ "Plugin" architecture: modular system for commands
- 'Memo' Mode: (quick-start exists in original Lines)
- open to line
- go to end of file
- Hex editor dual-view
- Byte viewing mode
- Byte editing
- help menu
- source-it (see File Fantastic)
- Pasty: copy(yank) paste
- modular clipboard
- insert file into file with file-path insert
- empty-enter repeat last action
- N-moves
- end of line, end of file, insertions.
- save-as
- Undo (optional, with constrained history buffers)
- Redo

## Future/Probably Scope:
- Multi-cursor/ctrl+d functionality
- Character encoding awareness
- Encoding conversion (write in different encodings)
- Search,
- fuzzy search,
- regex search,
- some Extended goto commands
- Configuration files
- build .rs for --version
- extract line
- extract header
- Select:
--1. ctrl+d superpower
--2. search for selection
--3. maybe crawl-count selection in file (though not all shown...)
- Find/replace
- Syntax highlighting
- File-Fantastic Integration

## ?
- super-mini directory file manager, for if "lines ." open in dir (list file/dir by number, if select file open in lines, if select dir show fiies, option 1 is back, option)

## Out of Scope:
- Mouse support
- Advanced goto features (e.g. Helix has a massive goto suite menu)
- Themes/colors beyond basic highlighting
- Line wrapping toggle
- relative lines



# Questions:

Here are some of the questions and thought behind, below, and around, the Lines-Rust-File-Editor project, or set of experiments:

1. Is it possible to make a use-able, maintain-able, utility with affordable and sustainable development and maintenance resource-cost requirements?

2. Is it possible to make software that is by design and by process following best-practice for soundness, safety, and maintainability?
- Will it have a never ending series of bugs and CVE's?
- Will it take 50 years to develop?
- Can it be easily accessed, understood, inspected, audited, and modified?

3. How long would it take for one not-very-skilled Rust programmer to make a best-practice clone of Vi(Vim)? How long would it take someone who understands workflow and best practice, and who has access to paper, pen, cup of tea, a watch, a few minutes here and there to plan, a few minutes here and there with access to a computer terminal, to adequately manage a best practice process based around checks and standards and audits, process and policy, to brick by brick assemble an MVP?

4. Are there (new or old) truly useful features and functionalities that available test editors and IDE's do not pragmatically have?
- hex-edit
- large file inspect
- large file concatenate
- file-insert
- ctrl+d
- modularity

5. The standard figure for a rough percent of critical vulnerability software exploits that are memory related is 70%. As this figure is not exact, it is even less exact to estimate what percent of that is heap related (such as heap buffer overflow) as opposed to stack. Given that use of heap predominates in much modern software, it is plausible that more than a baseline of 50% of the 70% are heap and while there is attempted process isolation in the heap on some systems, it is still a surface. Question: Is the reduction in attack surface by avoiding dynamic memory allocation significant or is it effectively of no value?


# Results:
- Time to write a working MVP of Vi(Vim) in Rust, greenfield: 1 week

- Time to build MVP-2, adding: hex editor (classic, in-place edit), raw string (showing escaped characters), select including: cursor, w(word forward) e(end of word forward), b(begining of word backwards), toggle standard comment (line or selected lines), toggle rust docstring (line or selected lines), indent/unident (line or selected lines), undo, redo, hex-add byte, hex-remove byte, hex-goto-byte, standardized number column indent (v1), clipboard, cut and paste, paste file from path, boot from(into) existing session(allowing for multiple windows and file-manager file select toggle), byte position display, continuation of cursor from line to line including select, boundaries to keep cursor in text-bounds, no-crash exception handling, save-as (which, strangely, was one of the least-simple to add), Goto (start of line end of line specific line, :boot to line, end of doc, start of doc, specific byte), sync hex-edit cursor and normal edit cursor locations, source-it (command to recreate source-code files ('crate')), delete selection, etc.: 3 weeks
