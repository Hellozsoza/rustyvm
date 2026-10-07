# SPDX-License-Identifier: GPL-3.0-only
# Assemble with GNU as --64, link with GNU ld -m elf_x86_64.
# This guest really checks 64-bit arithmetic and a memory address above 4 GiB.
.global _start
.section .text
_start:
    movabs $0x100000001, %rax
    movabs $0x200000002, %rbx
    add %rbx, %rax
    movabs $0x300000003, %r10
    cmp %r10, %rax
    jne fail
    movabs $0x100002000, %r12
    mov %rax, (%r12)
    mov (%r12), %r13
    cmp %r10, %r13
    jne fail
    mov $100, %ecx
    xor %edx, %edx
sum:
    add %rcx, %rdx
    dec %rcx
    jne sum
    cmp $5050, %rdx
    jne fail
    call print
    mov $60, %eax
    xor %edi, %edi
    syscall
print:
    push %rbp
    mov %rsp, %rbp
    mov $1, %eax
    mov $1, %edi
    lea message(%rip), %rsi
    mov $message_end-message, %edx
    syscall
    pop %rbp
    ret
fail:
    mov $60, %eax
    mov $1, %edi
    syscall
.section .rodata
message:
    .ascii "Rusty64: x86-64 code executed by Rust/WASM.\n"
    .ascii "64-bit arithmetic and memory above 4 GiB: OK.\n"
message_end:
.section .note.GNU-stack,"",@progbits
