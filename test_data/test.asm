section .text
global  _start

_start:
	MOV rax, 60
	MOV rdi, 0
	SYSCALL
