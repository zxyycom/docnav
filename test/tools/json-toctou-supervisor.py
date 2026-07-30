#!/usr/bin/env python3
"""Deterministically replace a JSON path at the probe's first content read."""

from __future__ import annotations

import argparse
import ctypes
import os
import platform
import signal
import stat
import sys
from collections.abc import Sequence


PTRACE_TRACEME = 0
PTRACE_GETREGS = 12
PTRACE_DETACH = 17
PTRACE_SYSCALL = 24
PTRACE_SETOPTIONS = 0x4200
PTRACE_O_TRACESYSGOOD = 0x00000001
PTRACE_O_EXITKILL = 0x00100000

SYSCALL_STOP = signal.SIGTRAP | 0x80
READ_SYSCALLS = frozenset({0, 17, 19})  # read, pread64, readv on Linux x86_64.
COORDINATION_EXIT = 125


class CoordinationError(RuntimeError):
    """The deterministic ptrace barrier could not be established."""


class UserRegsStruct(ctypes.Structure):
    _fields_ = [
        (name, ctypes.c_ulonglong)
        for name in (
            "r15",
            "r14",
            "r13",
            "r12",
            "rbp",
            "rbx",
            "r11",
            "r10",
            "r9",
            "r8",
            "rax",
            "rcx",
            "rdx",
            "rsi",
            "rdi",
            "orig_rax",
            "rip",
            "cs",
            "eflags",
            "rsp",
            "ss",
            "fs_base",
            "gs_base",
            "ds",
            "es",
            "fs",
            "gs",
        )
    ]


LIBC = ctypes.CDLL(None, use_errno=True)
LIBC.ptrace.restype = ctypes.c_long
LIBC.ptrace.argtypes = [
    ctypes.c_uint,
    ctypes.c_int,
    ctypes.c_void_p,
    ctypes.c_void_p,
]


def ptrace(request: int, pid: int, address: int = 0, data: int = 0) -> int:
    ctypes.set_errno(0)
    result = LIBC.ptrace(
        request,
        pid,
        ctypes.c_void_p(address),
        ctypes.c_void_p(data),
    )
    if result == -1:
        error_number = ctypes.get_errno()
        raise CoordinationError(
            f"ptrace request {request} for pid {pid} failed: "
            f"{os.strerror(error_number)}"
        )
    return int(result)


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--docnav-bin", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--replacement", required=True)
    parser.add_argument("command_args", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if args.command_args[:1] == ["--"]:
        args.command_args = args.command_args[1:]
    if not args.command_args:
        parser.error("docnav command arguments are required after --")
    return args


def regular_file(path: str, label: str) -> os.stat_result:
    try:
        result = os.stat(path)
    except OSError as error:
        raise CoordinationError(f"{label} cannot be inspected: {error}") from error
    if not stat.S_ISREG(result.st_mode):
        raise CoordinationError(f"{label} is not a regular file: {path}")
    return result


def validate_platform_and_inputs(
    docnav_bin: str,
    target: str,
    replacement: str,
) -> tuple[os.stat_result, os.stat_result]:
    if sys.platform != "linux" or platform.machine() != "x86_64":
        raise CoordinationError("ptrace barrier requires Linux x86_64")
    binary_stat = regular_file(docnav_bin, "docnav binary")
    if binary_stat.st_mode & 0o111 == 0:
        raise CoordinationError(f"docnav binary is not executable: {docnav_bin}")
    target_stat = regular_file(target, "target")
    replacement_stat = regular_file(replacement, "replacement")
    if target_stat.st_dev != replacement_stat.st_dev:
        raise CoordinationError("target and replacement must share a filesystem")
    if (target_stat.st_dev, target_stat.st_ino) == (
        replacement_stat.st_dev,
        replacement_stat.st_ino,
    ):
        raise CoordinationError("target and replacement must be different inodes")
    return target_stat, replacement_stat


def child_trace_and_exec(docnav_bin: str, command_args: Sequence[str]) -> None:
    try:
        ptrace(PTRACE_TRACEME, 0)
        os.kill(os.getpid(), signal.SIGSTOP)
        os.execve(
            docnav_bin,
            [docnav_bin, *command_args],
            os.environ.copy(),
        )
    except BaseException as error:
        message = f"json-toctou-supervisor coordination error: {error}\n"
        os.write(2, message.encode("utf-8", errors="replace"))
        os._exit(COORDINATION_EXIT)


def get_registers(pid: int) -> UserRegsStruct:
    registers = UserRegsStruct()
    ptrace(PTRACE_GETREGS, pid, data=ctypes.addressof(registers))
    return registers


def fd_identity(pid: int, fd: int) -> tuple[int, int] | None:
    try:
        result = os.stat(f"/proc/{pid}/fd/{fd}")
    except OSError:
        return None
    return result.st_dev, result.st_ino


def replace_at_barrier(
    pid: int,
    fd: int,
    target: str,
    replacement: str,
    target_stat: os.stat_result,
    replacement_stat: os.stat_result,
) -> None:
    expected_target = target_stat.st_dev, target_stat.st_ino
    expected_replacement = replacement_stat.st_dev, replacement_stat.st_ino
    if fd_identity(pid, fd) != expected_target:
        raise CoordinationError("target descriptor changed before atomic replacement")
    try:
        os.replace(replacement, target)
    except OSError as error:
        raise CoordinationError(f"atomic replacement failed: {error}") from error
    replaced_target = os.stat(target)
    if (replaced_target.st_dev, replaced_target.st_ino) != expected_replacement:
        raise CoordinationError("target path does not identify the replacement inode")
    if fd_identity(pid, fd) != expected_target:
        raise CoordinationError("probe descriptor no longer identifies the valid inode")


def child_exit_code(status: int) -> int:
    if os.WIFEXITED(status):
        return os.WEXITSTATUS(status)
    if os.WIFSIGNALED(status):
        return 128 + os.WTERMSIG(status)
    raise CoordinationError(f"unexpected child wait status after detach: {status}")


def wait_for_exit(pid: int) -> int:
    waited_pid, status = os.waitpid(pid, 0)
    if waited_pid != pid:
        raise CoordinationError(f"waited for unexpected pid {waited_pid}")
    return child_exit_code(status)


def trace_until_target_read(
    pid: int,
    target: str,
    replacement: str,
    target_stat: os.stat_result,
    replacement_stat: os.stat_result,
) -> int:
    waited_pid, status = os.waitpid(pid, 0)
    if waited_pid != pid or not os.WIFSTOPPED(status):
        raise CoordinationError("child exited before the initial ptrace stop")
    if os.WSTOPSIG(status) != signal.SIGSTOP:
        raise CoordinationError(
            f"child stopped with signal {os.WSTOPSIG(status)} before tracing"
        )

    ptrace(
        PTRACE_SETOPTIONS,
        pid,
        data=PTRACE_O_TRACESYSGOOD | PTRACE_O_EXITKILL,
    )
    ptrace(PTRACE_SYSCALL, pid)
    entering_syscall = True
    expected_target = target_stat.st_dev, target_stat.st_ino

    while True:
        waited_pid, status = os.waitpid(pid, 0)
        if waited_pid != pid:
            raise CoordinationError(f"waited for unexpected pid {waited_pid}")
        if os.WIFEXITED(status) or os.WIFSIGNALED(status):
            raise CoordinationError(
                f"docnav exited with status {child_exit_code(status)} "
                "before the target read barrier"
            )
        if not os.WIFSTOPPED(status):
            raise CoordinationError(f"unexpected traced child status: {status}")

        stop_signal = os.WSTOPSIG(status)
        if stop_signal == SYSCALL_STOP:
            if entering_syscall:
                registers = get_registers(pid)
                syscall_number = int(registers.orig_rax)
                fd = int(registers.rdi)
                if (
                    syscall_number in READ_SYSCALLS
                    and int(registers.rdx) > 0
                    and fd_identity(pid, fd) == expected_target
                ):
                    replace_at_barrier(
                        pid,
                        fd,
                        target,
                        replacement,
                        target_stat,
                        replacement_stat,
                    )
                    ptrace(PTRACE_DETACH, pid)
                    return wait_for_exit(pid)
            entering_syscall = not entering_syscall
            ptrace(PTRACE_SYSCALL, pid)
            continue

        delivered_signal = 0 if stop_signal == signal.SIGTRAP else stop_signal
        ptrace(PTRACE_SYSCALL, pid, data=delivered_signal)


def kill_and_reap(pid: int) -> None:
    try:
        os.kill(pid, signal.SIGKILL)
    except ProcessLookupError:
        return
    try:
        os.waitpid(pid, 0)
    except ChildProcessError:
        pass


def run(argv: Sequence[str]) -> int:
    args = parse_args(argv)
    docnav_bin = os.path.abspath(args.docnav_bin)
    target = os.path.abspath(args.target)
    replacement = os.path.abspath(args.replacement)
    target_stat, replacement_stat = validate_platform_and_inputs(
        docnav_bin,
        target,
        replacement,
    )

    pid = os.fork()
    if pid == 0:
        child_trace_and_exec(docnav_bin, args.command_args)
        raise AssertionError("exec returned")
    try:
        return trace_until_target_read(
            pid,
            target,
            replacement,
            target_stat,
            replacement_stat,
        )
    except BaseException:
        kill_and_reap(pid)
        raise


def main() -> int:
    try:
        return run(sys.argv[1:])
    except CoordinationError as error:
        print(
            f"json-toctou-supervisor coordination error: {error}",
            file=sys.stderr,
        )
        return COORDINATION_EXIT


if __name__ == "__main__":
    raise SystemExit(main())
