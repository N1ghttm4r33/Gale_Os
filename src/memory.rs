use x86_64::{
    structures::paging::PageTable,
    VirtAddr,
    PhysAddr,
    structures::paging::PhysFrame
};

use x86_64::structures::paging::OffsetPageTable;
use x86_64::structures::paging::Size4KiB;

use bootloader::bootinfo::MemoryMap;
use bootloader::bootinfo::MemoryRegionType;

use x86_64::instructions::interrupts;

use crate::println;

//inicia uma nova tabela de nível 4
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    println!("[MEMORY] Starting memory init with offset: {:#x}", physical_memory_offset.as_u64());
    
    let level_4_table = active_level_4_table(physical_memory_offset);
    println!("[MEMORY] Level 4 table obtained successfully");
    
    let mapper = OffsetPageTable::new(level_4_table, physical_memory_offset);
    println!("[MEMORY] OffsetPageTable created successfully");
    
    mapper
}

/// Ativa a página de level4 com verificações de segurança
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    println!("[MEMORY] Starting active_level_4_table");
    
    // Lê CR3 para obter o frame da tabela de nível 4
    let (level_4_table_frame, _) = Cr3::read();
    println!("[MEMORY] CR3 read - frame: {:#x}", level_4_table_frame.start_address().as_u64());

    let phys = level_4_table_frame.start_address();
    println!("[MEMORY] Physical address: {:#x}", phys.as_u64());
    
    println!("[MEMORY] Physical memory offset: {:#x}", physical_memory_offset.as_u64());
    
    // Calcula o endereço virtual
    let virt = physical_memory_offset + phys.as_u64();
    println!("[MEMORY] Virtual address calculated: {:#x}", virt.as_u64());

    // Verifica se o endereço virtual está alinhado
    if virt.as_u64() % 4096 != 0 {
        panic!("Virtual address not page-aligned: {:#x}", virt.as_u64());
    }
    
    println!("[MEMORY] Address is properly aligned");

    // Converte para ponteiro
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    println!("[MEMORY] Pointer created: {:p}", page_table_ptr);

    // Verificação adicional: tente ler o primeiro byte de forma segura
    println!("[MEMORY] Attempting to verify memory access...");
    
    // Tenta acessar de forma conservadora
    let probe_ptr = page_table_ptr as *const u8;
    let first_byte = core::ptr::read_volatile(probe_ptr);
    println!("[MEMORY] First byte read successfully: {:#x}", first_byte);

    // Agora podemos desreferenciar com mais segurança
    println!("[MEMORY] Attempting to dereference...");
    let table_ref = &mut *page_table_ptr;
    println!("[MEMORY] Dereference successful!");

    // Verifica o primeiro entry
    let first_entry = table_ref[0].addr();
    println!("[MEMORY] First page table entry: {:#x}", first_entry.as_u64());

    table_ref
}


//alocador de frame que retorna o memory map

extern crate alloc;
/// Um nó da nossa lista ligada de frames livres.
/// Ele será armazenado no início de cada frame físico livre.
struct ListNode {
    next: Option<&'static mut ListNode>,
}

pub struct LinkedListFrameAllocator {
    head: Option<&'static mut ListNode>,
    physical_memory_offset: u64,
}

impl LinkedListFrameAllocator {
    /// Inicializa o allocator e guarda o offset físico->virtual.
    /// Unsafe porque o caller garante validade do memory_map e offset.
    pub unsafe fn init(physical_memory_offset: u64, memory_map: &'static MemoryMap) -> Self {
        let mut allocator = Self {
            head: None,
            physical_memory_offset,
        };
        println!("2");
        allocator.build_free_list(memory_map);
        allocator
    }

    /// Constrói a lista de frames livres a partir do mapa de memória.
    /// Observação: para debug, estou limitando a adição inicial para evitar sobrescrever estruturas críticas.
    unsafe fn build_free_list(&mut self, memory_map: &'static MemoryMap) {
        println!("build_free_list: start");
        for region in memory_map.iter() {
            println!(" region: type={:?} start={:#x} end={:#x}", region.region_type, region.range.start_addr(), region.range.end_addr());
            if region.region_type != MemoryRegionType::Usable {
                continue;
            }

            let start = region.range.start_addr();
            let end   = region.range.end_addr();

            // itere por endereços físicos de 4KiB
            let mut addr = start;
            let mut added_in_region = 0usize;
            while addr + 0x1000 <= end {
                // DEBUG: adicione apenas alguns frames por região no início para reduzir risco
                // remova o limite (`&& added_in_region < 4096`) depois de testado
                if added_in_region < 1024 {
                    let frame = PhysFrame::containing_address(PhysAddr::new(addr));
                    self.deallocate_frame(frame);
                    added_in_region += 1;
                } else {
                    // se quiser adicionar todos, comente esse break
                    break;
                }
                addr += 0x1000;
            }
            println!(" region done, frames added = {}", added_in_region);
        }
        println!("build_free_list: done");
    }

    /// Converte físico -> ponteiro virtual para T
    fn phys_to_virt_ptr<T>(&self, phys_addr: u64) -> *mut T {
        let virt = phys_addr + self.physical_memory_offset;
        virt as *mut T
    }

    /// Converte ponteiro virtual (ListNode) -> endereço físico
    fn virt_ptr_to_phys(&self, ptr: *const ListNode) -> u64 {
        let virt = ptr as u64;
        virt - self.physical_memory_offset
    }
}

use x86_64::structures::paging::FrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for LinkedListFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        println!("allocate_frame: entering");
        match self.head.take() {
            Some(node) => {
                self.head = node.next.take();

                // node é um ponteiro virtual; converte para endereço físico
                let virt_ptr = node as *const ListNode;
                let phys_addr = self.virt_ptr_to_phys(virt_ptr);
                println!("allocate_frame: returning phys {:#x}", phys_addr);
                Some(PhysFrame::containing_address(PhysAddr::new(phys_addr)))
            }
            None => {
                println!("allocate_frame: none");
                None
            }
        }
    }
}

use x86_64::structures::paging::FrameDeallocator;

impl FrameDeallocator<Size4KiB> for LinkedListFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        println!("deallocate_frame: frame = {:#x}", frame.start_address().as_u64());
        let frame_addr = frame.start_address().as_u64();

        // converte físico -> virtual para obter o ponteiro onde escrever o ListNode
        let new_node_ptr = self.phys_to_virt_ptr::<ListNode>(frame_addr);
        println!("deallocate_frame: new_node_ptr = {:p}", new_node_ptr);

        // inicializa o novo nó (escrever neste endereço virtual seguro)
        // cuidado: estamos escrevendo na própria memória do frame
        (*new_node_ptr).next = self.head.take();

        // atualiza head
        self.head = Some(&mut *new_node_ptr);
    }
}
