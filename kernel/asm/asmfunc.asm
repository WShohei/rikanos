; asmfunc.asm
;
; System V AMD64 Calling Convention
; Registers: RDI, RSI, RDX, RCX, R8, R9

bits 64

extern kernel_main_new_stack

section .bss align=16
  KERNEL_MAIN_STACK:
  resb 1024 * 1024

section .text

global IoOut32 ; void IoOut32(uint16_t addr, uint32_t data);
IoOut32:
  mov dx, di ; dx = addr
  mov eax, esi ; eax = data
  out dx, eax
  ret

global IoIn32 ; uint32_t IoIn32(uint16_t addr);
IoIn32:
  mov dx, di ; dx = addr
  in eax, dx ; eax = data
  ret

global kernel_main
kernel_main:
  mov rsp, KERNEL_MAIN_STACK + 1024 * 1024
  call kernel_main_new_stack
.fin:
  hlt
  jmp .fin

global load_gdt ; fn load_gdt(limit: u16, offset: u64) -> ()
load_gdt:
  push rbp
  mov rbp, rsp
  sub rsp, 10
  mov [rsp], di ; limit
  mov [rsp + 2], rsi ; offset
  lgdt [rsp]
  mov rsp, rbp
  pop rbp
  ret

global set_dsall ; fn set_dsall(value: u16) -> ()
set_dsall:
  mov ds, di
  mov es, di
  mov fs, di
  mov gs, di
  ret

global set_csss ; fn set_csss(cs: u16, ss: u16) -> ()
set_csss:
  push rbp
  mov rbp, rsp
  mov ss, si
  mov rax, .next
  push rdi ; cs
  push rax ; RIP
  o64 retf
.next:
  mov rsp, rbp
  pop rbp
  ret

global set_cr3 ; fn set_cr3(value: u64) -> ()
set_cr3:
  mov cr3, rdi
  ret
