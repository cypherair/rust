"""Checks arm64e code for what pointer authentication misses: every object in the given rlibs or
archives, or, with `--linked`, linked Mach-O binaries.

Fails on:
- a function that saves its return address (x30) without signing it with `pacibsp`;
- an indirect call with a plain `blr`;
- an indirect branch with a plain `br`, unless it ends LLVM's hardened jump-table dispatch;
- in an object, a data relocation that stores a function's address unsigned;
- in a linked binary, a plain rebase (an unsigned pointer) to an address in `__text`.

Prints, per archive or binary, how many plain `br` it accepted as hardened jump-table dispatches,
and the instructions around each branch it rejects.

Usage: check_objects.py LLVM_BIN_DIR ARCHIVE...
       check_objects.py LLVM_BIN_DIR --linked BINARY...
"""

import pathlib
import re
import subprocess
import sys
import tempfile

# Sections whose relocations to code are not function pointers that code calls through.
UNSIGNED_OK_SECTIONS = {
    "__text",
    "__compact_unwind",
    "__eh_frame",
    "__gcc_except_tab",
    "__mod_init_func",  # left unsigned on purpose, like clang's own constructors
    "__mod_term_func",
    "__init_offsets",
    "__thread_vars",  # TLV descriptors name `_tlv_bootstrap`; dyld binds them
    "__got",  # code loads a raw address from the GOT and signs it, as clang does
}

FUNCTION_HEADER = re.compile(r"^[0-9a-f]+ <(.+)>:$")
INSTRUCTION = re.compile(r"^\s*[0-9a-f]+:\s+(\S+)\s*(.*)$")
SECTION_HEADER = re.compile(r"^RELOCATION RECORDS FOR \[(.+)\]:$")
RELOCATION = re.compile(r"^[0-9a-f]+\s+(\S+)\s+(\S+)$")
SYMBOL = re.compile(r"^[0-9a-f]+\s+[lg ]\s+F\s+__TEXT,__text\s+(\S+)$")
TEXT_SECTION = re.compile(r"^\s*\d+\s+__text\s+([0-9a-f]+)\s+([0-9a-f]+)\s")
# `dyld_info -fixups`: segment, section, address, kind, target.
FIXUP = re.compile(r"^\s*(\S+)\s+(\S+)\s+0x[0-9A-Fa-f]+\s+(\S+)\s+(\S+)")


def run(tool, *args):
    command = [str(tool), *map(str, args)]
    return subprocess.run(command, check=True, capture_output=True, text=True).stdout


def function_symbols(objdump, obj):
    lines = run(objdump, "-t", obj).splitlines()
    return {m.group(1) for line in lines if (m := SYMBOL.match(line))}


# The hardened jump-table dispatch, as LLVM emits it (`LowerHardenedBRJumpTable`) before its
# `br x16`, after the bounds check: the index is clamped again, and the table address, the entry and
# the target never leave x16 and x17.
HARDENED_DISPATCH = (
    ("csel", re.compile(r"x16, x16, xzr, ls")),
    ("adrp", re.compile(r"x17, .*")),
    ("add", re.compile(r"x17, x17, #.*")),
    ("ldrsw", re.compile(r"x16, \[x17, x16, lsl #2\]")),
    ("adr", re.compile(r"x17, .*")),
    ("add", re.compile(r"x16, x17, x16")),
)
IMMEDIATE = r"#(0x)?[0-9a-f]+"


def is_hardened_dispatch(insns, i):
    """Whether the plain `br` at `insns[i]` ends a hardened jump-table dispatch."""
    n = len(HARDENED_DISPATCH)
    if insns[i][1] != "x16" or i < n + 1:
        return False
    for (mnemonic, ops), (want, pattern) in zip(insns[i - n : i], HARDENED_DISPATCH):
        if mnemonic != want or not pattern.fullmatch(ops):
            return False
    mnemonic, ops = insns[i - n - 1]
    if mnemonic == "cmp" and re.fullmatch(rf"x16, {IMMEDIATE}", ops):
        return True
    # A table too large for `cmp`'s immediate: its size goes into x17 with `mov` and `movk`.
    if mnemonic != "cmp" or ops != "x16, x17":
        return False
    j = i - n - 2
    while j >= 0 and insns[j][0] == "movk" and insns[j][1].startswith("x17, "):
        j -= 1
    return j >= 0 and insns[j][0] == "mov" and re.fullmatch(rf"x17, {IMMEDIATE}", insns[j][1])


def context(lines, i):
    """The instructions around `lines[i]`, which is marked."""
    around = range(max(0, i - 12), min(len(lines), i + 3))
    return "".join(f"\n    {'>' if j == i else ' '} {lines[j]}" for j in around)


def check_code(objdump, obj, problems):
    """Returns the number of functions, and of plain `br` accepted as hardened dispatches."""
    functions = []
    for line in run(objdump, "-d", "--no-show-raw-insn", obj).splitlines():
        if m := FUNCTION_HEADER.match(line):
            functions.append((m.group(1), [], []))
        elif (m := INSTRUCTION.match(line)) and functions:
            # Without objdump's comment: `mov x17, #0x1387 ; =4999`.
            functions[-1][1].append((m.group(1), m.group(2).split(";")[0].strip()))
            functions[-1][2].append(" ".join(line.split()))
    jump_tables = 0
    for name, insns, lines in functions:
        mnemonics = [m for m, _ in insns]
        # The registers stored, not the address: `stp x29, x30, [sp, #-0x10]!`.
        stored = [ops.split("[")[0] for m, ops in insns if m in ("stp", "str", "stur")]
        saves_lr = any("x30" in registers for registers in stored)
        signs_lr = "pacibsp" in mnemonics
        if saves_lr and not signs_lr:
            problems.append(f"{obj.name}: {name} saves x30 without pacibsp")
        for i, (m, ops) in enumerate(insns):
            if m == "blr":
                problems.append(
                    f"{obj.name}: {name} calls through a plain blr {ops}{context(lines, i)}"
                )
            elif m == "br" and is_hardened_dispatch(insns, i):
                jump_tables += 1
            elif m == "br":
                problems.append(
                    f"{obj.name}: {name} branches through a plain br {ops}{context(lines, i)}"
                )
    return len(functions), jump_tables


def check_data(objdump, obj, functions, problems):
    authenticated = 0
    section = None
    for line in run(objdump, "-r", obj).splitlines():
        if m := SECTION_HEADER.match(line):
            section = m.group(1)
        elif (m := RELOCATION.match(line)) and section is not None:
            kind, target = m.groups()
            if kind == "ARM64_RELOC_AUTHENTICATED_POINTER":
                authenticated += 1
            elif (
                kind == "ARM64_RELOC_UNSIGNED"
                and section not in UNSIGNED_OK_SECTIONS
                and not section.startswith(("__debug", "__apple"))  # DWARF names code addresses
            ):
                if target in functions or target == "__text":
                    problems.append(f"{obj.name}: [{section}] stores {target} unsigned")
    return authenticated


def text_range(objdump, binary):
    for line in run(objdump, "-h", binary).splitlines():
        if m := TEXT_SECTION.match(line):
            size, start = int(m.group(1), 16), int(m.group(2), 16)
            return start, start + size
    sys.exit(f"{binary}: no __text section")


def check_linked(objdump, binary, problems):
    """Checks a linked binary's code, and dyld's fixups for unsigned pointers to its code."""
    n_functions, jump_tables = check_code(objdump, binary, problems)
    print(f"{binary}: {n_functions} functions, {jump_tables} jump-table branches")
    start, end = text_range(objdump, binary)
    authenticated = 0
    for line in run("dyld_info", "-fixups", binary).splitlines():
        if not (m := FIXUP.match(line)):
            continue
        segment, section, kind, target = m.groups()
        if kind.startswith("auth-"):
            authenticated += 1
        elif kind == "rebase" and section not in UNSIGNED_OK_SECTIONS:
            address = int(target, 16)
            if start <= address < end:
                problems.append(f"{binary.name}: [{segment},{section}] stores {target} unsigned")
    return n_functions, authenticated


def main_linked(objdump, binaries):
    problems = []
    n_functions = n_auth = 0
    for binary in binaries:
        functions, authenticated = check_linked(objdump, binary, problems)
        n_functions += functions
        n_auth += authenticated
    print(f"{len(binaries)} binaries, {n_functions} functions, "
          f"{n_auth} authenticated fixups, {len(problems)} problems")
    for problem in problems[:200]:
        print(problem)
    if not n_functions or not n_auth or problems:
        sys.exit(1)


def main():
    llvm_bin = pathlib.Path(sys.argv[1])
    if sys.argv[2:3] == ["--linked"]:
        main_linked(llvm_bin / "llvm-objdump", [pathlib.Path(b) for b in sys.argv[3:]])
        return
    archives = [pathlib.Path(a) for a in sys.argv[2:]]
    objdump, ar = llvm_bin / "llvm-objdump", llvm_bin / "llvm-ar"
    if not archives:
        sys.exit("no archives given")
    problems = []
    with tempfile.TemporaryDirectory() as tmp:
        members = []
        for archive in archives:
            out = pathlib.Path(tmp) / archive.stem
            out.mkdir()
            run(ar, "x", f"--output={out}", archive)
            members.append((archive, sorted(out.glob("*.o"))))
        objects = [obj for _, objs in members for obj in objs]
        functions = set().union(*(function_symbols(objdump, o) for o in objects))
        n_functions = 0
        for archive, objs in members:
            counts = [check_code(objdump, o, problems) for o in objs]
            archive_functions = sum(c[0] for c in counts)
            print(f"{archive.name}: {len(objs)} objects, {archive_functions} functions, "
                  f"{sum(c[1] for c in counts)} jump-table branches")
            n_functions += archive_functions
        n_auth = sum(check_data(objdump, o, functions, problems) for o in objects)
    print(f"{len(archives)} archives, {len(objects)} objects, {n_functions} functions, "
          f"{n_auth} authenticated pointers in data, {len(problems)} problems")
    for problem in problems[:200]:
        print(problem)
    # The data check needs the function symbols to recognise a relocation to a function.
    if not objects or not functions or not n_functions or not n_auth or problems:
        sys.exit(1)


if __name__ == "__main__":
    main()
