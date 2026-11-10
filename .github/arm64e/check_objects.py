"""Checks every object in the given rlibs or archives for code that pointer authentication misses.

Fails on:
- a function that saves its return address (x30) without signing it with `pacibsp`;
- an indirect call with a plain `blr`;
- an indirect branch with a plain `br`, unless it is a jump-table dispatch;
- a data relocation that stores a function's address unsigned.

Usage: check_objects.py LLVM_BIN_DIR ARCHIVE...
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
}

FUNCTION_HEADER = re.compile(r"^[0-9a-f]+ <(.+)>:$")
INSTRUCTION = re.compile(r"^\s+[0-9a-f]+:\s+(\S+)\s*(.*)$")
SECTION_HEADER = re.compile(r"^RELOCATION RECORDS FOR \[(.+)\]:$")
RELOCATION = re.compile(r"^[0-9a-f]+\s+(\S+)\s+(\S+)$")
SYMBOL = re.compile(r"^[0-9a-f]+\s+[lg ]\s+F\s+__TEXT,__text\s+(\S+)$")


def run(tool, *args):
    command = [str(tool), *map(str, args)]
    return subprocess.run(command, check=True, capture_output=True, text=True).stdout


def function_symbols(objdump, obj):
    lines = run(objdump, "-t", obj).splitlines()
    return {m.group(1) for line in lines if (m := SYMBOL.match(line))}


def check_code(objdump, obj, problems):
    functions = []
    for line in run(objdump, "-d", "--no-show-raw-insn", obj).splitlines():
        if m := FUNCTION_HEADER.match(line):
            functions.append((m.group(1), []))
        elif (m := INSTRUCTION.match(line)) and functions:
            functions[-1][1].append((m.group(1), m.group(2)))
    for name, insns in functions:
        mnemonics = [m for m, _ in insns]
        # The registers stored, not the address: `stp x29, x30, [sp, #-0x10]!`.
        stored = [ops.split("[")[0] for m, ops in insns if m in ("stp", "str", "stur")]
        saves_lr = any("x30" in registers for registers in stored)
        signs_lr = "pacibsp" in mnemonics
        if saves_lr and not signs_lr:
            problems.append(f"{obj.name}: {name} saves x30 without pacibsp")
        for i, (m, ops) in enumerate(insns):
            if m == "blr":
                problems.append(f"{obj.name}: {name} calls through a plain blr {ops}")
            elif m == "br" and "adr" not in mnemonics[max(0, i - 6) : i]:
                problems.append(f"{obj.name}: {name} branches through a plain br {ops}")
    return len(functions)


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


def main():
    llvm_bin = pathlib.Path(sys.argv[1])
    archives = [pathlib.Path(a) for a in sys.argv[2:]]
    objdump, ar = llvm_bin / "llvm-objdump", llvm_bin / "llvm-ar"
    if not archives:
        sys.exit("no archives given")
    problems = []
    with tempfile.TemporaryDirectory() as tmp:
        objects = []
        for archive in archives:
            out = pathlib.Path(tmp) / archive.stem
            out.mkdir()
            run(ar, "x", f"--output={out}", archive)
            objects += sorted(out.glob("*.o"))
        functions = set().union(*(function_symbols(objdump, o) for o in objects))
        n_functions = sum(check_code(objdump, o, problems) for o in objects)
        n_auth = sum(check_data(objdump, o, functions, problems) for o in objects)
    print(f"{len(archives)} archives, {len(objects)} objects, {n_functions} functions, "
          f"{n_auth} authenticated pointers in data, {len(problems)} problems")
    for problem in problems[:200]:
        print(problem)
    if not objects or not n_functions or not n_auth or problems:
        sys.exit(1)


if __name__ == "__main__":
    main()
