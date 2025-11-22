use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use core::ptr::addr_of;

use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(addr_of!(STACK) );
            let stack_end = stack_start + STACK_SIZE.try_into().unwrap();
            stack_end
        };
        tss
    };
}

// Struct de seletores atualizada para incluir o seletor de dados
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector, // <-- ADICIONADO
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let data_selector = gdt.append(Descriptor::kernel_data_segment()); // <-- ADICIONADO: Cria o segmento de dados
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, data_selector, tss_selector }) // <-- ADICIONADO: Salva o seletor de dados
    };
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    // Importa CS e SS e os outros para podermos carregá-los
    use x86_64::instructions::segmentation::{CS, SS, DS, ES, FS, GS, Segment}; // <-- MODIFICADO

    GDT.0.load();
    unsafe {
        // Carrega os segmentos de código e TSS como antes
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);

        // --- SEÇÃO CRÍTICA ADICIONADA ---
        // Carrega o seletor de dados em todos os outros registradores de segmento.
        // O mais importante é o SS (Stack Segment).
        SS::set_reg(GDT.1.data_selector); // <-- CRÍTICO: Carrega um SS válido
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        FS::set_reg(GDT.1.data_selector);
        GS::set_reg(GDT.1.data_selector);
    }
}