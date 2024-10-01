use crate::operations::{Operation, Program};

const INTEGER_ARGUMENT_ORDDER: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
const INTEGER_RETURN_ORDER: [&str; 2] = ["rax", "rdx"];


const _PRINT_INT_ASM: &str = "
print_int:
    sub     rsp, 40
    mov     eax, 1
    mov     BYTE [rsp+31], 10
    test    edi, edi
    jns     .L2
    mov     BYTE [rsp+30], 45
    neg     edi
    mov     eax, 2
.L2:
    mov     r9, rsp
    mov     esi, 3435973837
    mov     rdx, r9
    sub     rdx, rax
    add     rdx, 31
.L3:
    mov     eax, edi
    lea     r8, [rsp+32]
    imul    rax, rsi
    sub     r8, rdx
    shr     rax, 35
    lea     ecx, [rax+rax*4]
    add     ecx, ecx
    sub     edi, ecx
    mov     BYTE [rdx], dil
    mov     edi, eax
    mov     rax, rdx
    sub     rdx, 1
    test    edi, edi
    jne     .L3
    lea     rsi, [rsp+32]
    mov     rdx, r8
    mov     edi, 1
    sub     rax, rsi
    lea     rsi, [r9+32+rax]
    mov     rax, 1
    syscall
    add     rsp, 40
    ret
";

const PRINT_INT_ASM: &str ="
print_int:
    mov     r9, -3689348814741910323
    sub     rsp, 40
    mov     BYTE [rsp+31], 10
    lea     rcx, [rsp+30]
.L2:
    mov     rax, rdi
    lea     r8, [rsp+32]
    mul     r9
    mov     rax, rdi
    sub     r8, rcx
    shr     rdx, 3
    lea     rsi, [rdx+rdx*4]
    add     rsi, rsi
    sub     rax, rsi
    add     eax, 48
    mov     BYTE [rcx], al
    mov     rax, rdi
    mov     rdi, rdx
    mov     rdx, rcx
    sub     rcx, 1
    cmp     rax, 9
    ja      .L2
    lea     rax, [rsp+32]
    mov     edi, 1
    sub     rdx, rax
    xor     eax, eax
    lea     rsi, [rsp+32+rdx]
    mov     rdx, r8
    mov     rax, 1
    syscall
    add     rsp, 40
    ret
";


pub struct Compiler {
}

impl Compiler {

  pub fn compile_program(program: Program) -> String {
    let mut output = String::new();
    // executable part
    output.push_str("segment .text\n");
    output.push_str(PRINT_INT_ASM);
    // defined functions
    let functions = Compiler::translate_operations(&program.function_defs);
    output.push_str(&functions);
    // main
    output.push_str("global _start\n");
    output.push_str("_start:\n");
    let main = Compiler::translate_operations(&program.main);
    output.push_str(&main);
    // Safe exit
    output.push_str("    mov rax, 60\n");
    output.push_str("    mov rdi, 0\n");
    output.push_str("    syscall\n");
    output.push('\n');
    // uninitialized data
    output.push_str("segment .bss\n");
    for name in program.vars {
      output.push_str(format!("{}: resb 8\n", name).as_str());
    }
    output
  }

  fn translate_operations(operations: &Vec<Operation>) -> String {
    let mut output: String = String::new();
    for step in operations {
      match step {
        Operation::PushInt(s) => {
          output.push_str(&format!("    push {s}\n"));
        },
        Operation::AddInt => {
          output.push_str("    pop rbx\n");
          output.push_str("    pop rax\n");
          output.push_str("    add rax, rbx\n");
          output.push_str("    push rax\n");
        },
        Operation::MultInt => {
          output.push_str("    pop rbx\n");
          output.push_str("    pop rax\n");
          output.push_str("    mul rax, rbx\n");
          output.push_str("    push rax\n");
        },
        Operation::MinusInt => {
          output.push_str("    pop rbx\n");
          output.push_str("    pop rax\n");
          output.push_str("    sub rax, rbx\n");
          output.push_str("    push rax\n");
        },
        Operation::DivInt => {
          output.push_str("    pop rbx\n");
          output.push_str("    pop rax\n");
          output.push_str("    div rax, rbx\n");
          output.push_str("    push rax\n");
        },
        Operation::EqualInt => {
          output.push_str("    mov r12, 0\n");
          output.push_str("    mov r13, 1\n");
          output.push_str("    pop rbx\n");
          output.push_str("    pop rax\n");
          output.push_str("    cmp rax, rbx\n");
          output.push_str("    cmove r12, r13\n");
          output.push_str("    push r12\n");
        }
        Operation::GreaterInt => {
          output.push_str("    mov r12, 0\n");
          output.push_str("    mov r13, 1\n");
          output.push_str("    pop rbx\n");
          output.push_str("    pop rax\n");
          output.push_str("    cmp rax, rbx\n");
          output.push_str("    cmovg r12, r13\n");
          output.push_str("    push r12\n");
        }
        Operation::LessInt => {
          output.push_str("    mov r12, 0\n");
          output.push_str("    mov r13, 1\n");
          output.push_str("    pop rbx\n");
          output.push_str("    pop rax\n");
          output.push_str("    cmp rax, rbx\n");
          output.push_str("    cmovl r12, r13\n");
          output.push_str("    push r12\n");
        }
        Operation::PushFloat(_) => todo!(),
        Operation::AddFloat => todo!(),
        Operation::MultFloat => todo!(),
        Operation::MinusFloat => todo!(),
        Operation::DivFloat => todo!(),
        Operation::PrintInt => {
          output.push_str("    pop rdi\n");
          output.push_str("    call print_int\n");
        },
        Operation::LoadInt(addr) => {
          output.push_str(format!("    mov rax, [{}]\n", addr).as_str());
          output.push_str("    push rax\n");
        },
        Operation::StoreInt(addr) => {
          output.push_str("    pop rax\n");
          output.push_str(format!("    mov [{}], rax\n", addr).as_str());
        },
        Operation::If(n) => {
          output.push_str("    pop rax\n");
          output.push_str("    cmp rax, 0\n");
          output.push_str(format!("    je ELSE_{}\n", n).as_str());
        }
        Operation::Else(n) => {
          output.push_str(format!("    jmp END_IF_{}\n", n).as_str());
          output.push_str(format!("ELSE_{}:\n", n).as_str());
        }
        Operation::EndIF(n) => {
          output.push_str(format!("END_IF_{}:\n", n).as_str());
        }
        Operation::While(n) => {
          output.push_str(format!("WHILE_{}:\n", n).as_str());
        }
        Operation::CondWhile(n) => {
          output.push_str("    pop rax\n");
          output.push_str("    cmp rax, 0\n");
          output.push_str(format!("    je END_WHILE_{}\n", n).as_str());
        }
        Operation::EndWhile(n) => {
          output.push_str(format!("    jmp WHILE_{}\n", n).as_str());
          output.push_str(format!("END_WHILE_{}:\n", n).as_str());
        }
        Operation::PopStack => {
          output.push_str("    pop rax\n");
        }
        Operation::BeginFunction(name) => {
          output.push_str(format!("{}:\n", name).as_str());
          output.push_str("    push rbp\n");
          output.push_str("    mov rbp, rsp\n");
        }
        Operation::EndFunction() => {
          output.push_str("    mov rsp, rbp\n");
          output.push_str("    pop rbp\n");
          output.push_str("    ret\n");
        }
        Operation::FunctionCall(name, _) => {
          output.push_str(format!("    call {}\n", name).as_str());
        }
        Operation::ParameterIntegerStore(n) => {
          if n < &INTEGER_ARGUMENT_ORDDER.len() {
            output.push_str("    pop rax\n");
            output.push_str(format!("    mov QWORD [rbp - 8*{}], rax\n", n + 1).as_str());
          }
          else {
            todo!()
          }
        },
        Operation::ParameterIntegerLoad(n) => {
          if n < &6 {
            output.push_str(format!("    push QWORD [rbp - 8*{}]\n", n + 1).as_str());
          }
          else {
            todo!()
          }
        },
        Operation::ArgumentIntegerStore(n) => {
          if n < &6 {
            let reg = INTEGER_ARGUMENT_ORDDER[*n];
            output.push_str(format!("    pop {reg}\n").as_str());
          }
          else {
            todo!()
          }
        }
        Operation::ArgumentIntegerLoad(n) => {
          if n < &6 {
            let reg = INTEGER_ARGUMENT_ORDDER[*n];
            output.push_str(format!("    mov QWORD [rbp - 8*{}], {reg}\n", n + 1).as_str());
          }
          else {
            todo!()
          }
        }
        Operation::ReserveParameters(n) => {
            output.push_str(format!("    sub rsp, {}\n", (n + 1)*8).as_str());
        }
      }
    }
    output
  }
}