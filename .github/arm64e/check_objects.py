"""Checks arm64e code for what pointer authentication misses: every object in the given rlibs or
archives, or, with `--linked`, linked Mach-O binaries.

Fails on:
- a function that saves its return address (x30) without signing it with `pacibsp`;
- an indirect call with a plain `blr`;
- an indirect branch with a plain `br`, unless it is a jump-table dispatch;
- in an object, a data relocation that stores a function's address unsigned;
- in a linked binary, a plain rebase (an unsigned pointer) to an address in `__text`.

Prints, per archive or binary, how many plain `br` it accepted as jump-table dispatches, and the
instructions around each branch it rejects.

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


def register(operand):
    """`w9` and `x9` are one register."""
    operand = operand.strip()
    return "x" + operand[1:] if re.fullmatch(r"w\d+", operand) else operand


def operands(ops):
    """The operands before any memory operand: `ldp x9, x10, [sp]` gives `x9` and `x10`."""
    return [register(o) for o in ops.split("[")[0].split(",") if o.strip()]


# Instructions that write no register named in their operands.
NO_DESTINATION = re.compile(
    r"^(b|b\..*|bl|blr.*|br.*|cbn?z|tbn?z|ret.*|eret.*|cmp|cmn|tst|f?ccmp|ccmn|fcmpe?|nop|"
    r"prfu?m|dmb|dsb|isb|hint|brk|udf|msr|sys|bti|clrex|yield|wf[ei]|sevl?|csdb)$"
)
STORE = re.compile(r"^st(?!l?xr|l?xp)")  # store-exclusives write their status register
WRITEBACK = re.compile(r"\[(x\d+|sp)(?:,[^\]]*)?\]!|\[(x\d+|sp)\],")
# A call may leave anything in the caller-saved registers.
CALL = re.compile(r"^(bl|blr.*)$")
CALL_CLOBBERED = {f"x{n}" for n in range(19)} | {"x30"}
NO_FALLTHROUGH = re.compile(r"^(b|br.*|ret.*|brk|udf)$")


def writes(mnemonic, ops):
    """The registers an instruction writes."""
    if CALL.match(mnemonic):
        return CALL_CLOBBERED
    written = set()
    if m := WRITEBACK.search(ops):
        written.add(m.group(1) or m.group(2))
    if NO_DESTINATION.match(mnemonic) or STORE.match(mnemonic):
        return written
    names = operands(ops)
    count = 2 if mnemonic in ("ldp", "ldnp", "ldpsw", "ldxp", "ldaxp") else 1
    return written | set(names[:count])


def is_jump_table_dispatch(insns, i):
    """Whether `br xR` at `insns[i]` dispatches through a jump table, as LLVM emits it: xR was last
    written by `add xR, xA, xB`, where xA is an `adr` anchor and xB a table entry, loaded within the
    two instructions before the `add`. The plain form is `adr xR; ldr xB; add xR, xR, xB`, the
    hardened one `ldrsw x16; adr x17; add x16, x17, x16`."""
    (target,) = operands(insns[i][1])
    for k in range(i - 1, -1, -1):
        mnemonic, ops = insns[k]
        if target in writes(mnemonic, ops):
            break
        if NO_FALLTHROUGH.match(mnemonic):
            return False
    else:
        return False
    names = operands(ops)
    if mnemonic != "add" or len(names) < 3 or k < 2:
        return False
    anchor, entry = names[1], names[2]
    window = insns[k - 2 : k]
    anchored = any(m == "adr" and operands(o)[:1] == [anchor] for m, o in window)
    loaded = any(m.startswith("ldr") and operands(o)[:1] == [entry] for m, o in window)
    return anchored and loaded


def context(lines, i):
    """The instructions around `lines[i]`, which is marked."""
    around = range(max(0, i - 12), min(len(lines), i + 3))
    return "".join(f"\n    {'>' if j == i else ' '} {lines[j]}" for j in around)


def check_code(objdump, obj, problems):
    """Returns the number of functions, and of plain `br` accepted as jump-table dispatches."""
    functions = []
    for line in run(objdump, "-d", "--no-show-raw-insn", obj).splitlines():
        if m := FUNCTION_HEADER.match(line):
            functions.append((m.group(1), [], []))
        elif (m := INSTRUCTION.match(line)) and functions:
            functions[-1][1].append((m.group(1), m.group(2)))
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
            elif m == "br" and is_jump_table_dispatch(insns, i):
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
