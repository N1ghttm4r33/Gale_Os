🌪️ GaleOS

GaleOS é um kernel bare-metal escrito em Rust para arquitetura x86_64. O projeto foca na implementação segura de gerenciamento de memória e tratamento de interrupções, utilizando as abstrações modernas do Rust sem depender da biblioteca padrão (no_std).

🚀 Funcionalidades Implementadas

🧠 Gerenciamento de Memória Avançado

Diferente de tutoriais básicos, o GaleOS implementa um Alocador Híbrido (CombinedAllocator) complexo e thread-safe:

Small Block Allocator: Utiliza bitmaps para gerenciar alocações pequenas (≤ 256 bytes) de forma rápida.

Large Block Allocator: Utiliza listas encadeadas de blocos livres para alocações maiores, com suporte a merging (fusão) de blocos adjacentes na desalocação.

Page Caching: Implementa um sistema de cache para reutilização de páginas desalocadas.

Paging: Mapeamento de memória física com LinkedListFrameAllocator.

⚡ Interrupções e Hardware

PIC 8259 (Chained): Gerenciamento de interrupções de hardware (IRQ).

IDT (Interrupt Descriptor Table): Tratamento completo de exceções:

Page Faults: Com despejo de registradores (CR2) e Error Codes.

Double Faults: Com troca de pilha via TSS (Task State Segment) para evitar stack overflow do kernel.

Breakpoints & Invalid Opcodes.

PS/2 Keyboard: Driver de teclado que decodifica scancodes (Set 1) e permite interação direta na tela (movimentação de cursor implementada no handler).

Timer: Interrupções periódicas de hardware.

🖥️ Saída e Debugging

VGA Text Mode: Driver seguro (spin::Mutex + Volatile) para escrita na memória de vídeo 0xb8000. Suporta cores e posicionamento (x,y).

Serial Port (UART 16550): Redirecionamento de logs e saída de testes para o host via porta serial (serial_println!).

🛠️ Pré-requisitos

Você precisa da toolchain Nightly do Rust e ferramentas de compilação cruzada.

Instalar Rust Nightly e Componentes:

rustup install nightly
rustup default nightly
rustup component add rust-src llvm-tools-preview


Adicionar Target:

rustup target add x86_64-unknown-none


Instalar Bootimage:

cargo install bootimage


Emulador (QEMU):

Linux: sudo apt install qemu-system-x86

Windows/macOS: Instalar via site oficial.

▶️ Compilação e Execução

O projeto está configurado para rodar via cargo run, que invoca o bootimage e o QEMU automaticamente.

Rodar Kernel

cargo run


Isso iniciará o QEMU. Você verá logs de inicialização da memória ("BootInfo details", alocações bem-sucedidas) e poderá digitar no teclado.

Rodar Testes

O GaleOS possui um framework de testes customizado que usa a porta serial para reportar status e sair do QEMU (via isa-debug-exit).

cargo test


📂 Estrutura do Código

src/main.rs: Ponto de entrada (_start via entry_point), inicialização do kernel e demonstração de alocação de memória.

src/combined_allocator.rs: A "joia" do sistema. Implementa a lógica de GlobalAlloc manual, gerenciando Pages, Bitmaps e Free Lists.

src/interrupts.rs: Configuração da IDT e handlers de interrupção (Timer, Keyboard, Page Fault).

src/gdt.rs: Configuração da GDT e TSS. Nota: Inclui correção crítica para carregamento de segmentos de dados (DS, SS, ES).

src/vga_buffer.rs: Driver de vídeo com suporte a macros println!.

🐛 Debugging

O Cargo.toml está configurado com argumentos específicos para testes (-device isa-debug-exit...). Para debugging manual, você pode rodar:

qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-gale_sys.bin -s -S


E conectar o GDB na porta :1234.

Este projeto é um kernel experimental para fins de aprendizado em OSDev.
