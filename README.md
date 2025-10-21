# lines_editor

Lines is a minimal text editor.

```
quit save undo del|norm ins vis|wrap raw rlativ byte|wrd,b,end /cmmnt []rpt hjkl
1 # lines_editor
2
3 Lines is a minimal text editor.


















NORMAL 1:3 README.md >
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
- Save operations (save, save-as)
- File safety (read-copies, timestamped archives)
- Line wrapping toggle
- Visual mode (selection)
- Line numbers (absolute/relative)
- Multi-cursor/ctrl+d functionality
~ "Plugin" architecture: modular system for commands
- 'Memo' Mode: (quick-start exists in original Lines)
- open to line
- go to end of file (just iterate, must see line number)


## Future/Probably Scope:
- relative lines
- Character encoding awareness
- extended delete (line array slice)
- Undo (optional, with constrained history buffers)
- Encoding conversion (write in different encodings)
- Hex editor dual-view
- Search,
- fuzzy search,
- regex search,
- some Extended goto commands
- Configuration files
- help menu
- build .rs for --version
- source-it (see File Fantastic)
- super-mini directory file manager, for if "lines ." open in dir (list file/dir by number, if select file open in lines, if select dir show fiies, option 1 is back, option)
- Byte viewing mode
- Byte editing

- Select:
--1. delete a selection
--2. ctrl+d superpower
--3. search for selection
--4. maybe crawl-count selection in file (though not all shown...)

## Out of Scope:
- Redo functionality
- Find/replace
- Syntax highlighting
- Multiple file tabs/buffers
- Clipboard integration beyond terminal
- Mouse support
- Advanced goto features (e.g. Helix has a massive goto suite menu)
- Themes/colors beyond basic highlighting





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
- Time to write a working MVP of Vi(Vim) in Rust: 1 week
