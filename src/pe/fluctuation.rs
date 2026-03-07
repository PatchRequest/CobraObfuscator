use anyhow::{Context, Result};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::types::PeFile;

const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
const IMAGE_SCN_CNT_CODE: u32 = 0x00000020;
const IMAGE_SCN_MEM_DISCARDABLE: u32 = 0x02000000;

fn align_up(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

// ── Data block layout (104 bytes) ─────────────────────────────────────────
const DATA_XOR_KEY: usize = 0x00;           // 16 bytes
const DATA_TEXT_VA: usize = 0x10;           // u64 — computed at runtime by setup
const DATA_TEXT_SIZE: usize = 0x18;         // u64 — filled at build time
const DATA_TIMER_QUEUE: usize = 0x20;       // u64 — runtime
const DATA_TIMER_HANDLE: usize = 0x28;      // u64 — runtime
const DATA_P_VIRTUALPROTECT: usize = 0x30;  // u64 — runtime
const DATA_P_ADD_VEH: usize = 0x38;         // u64 — runtime
const DATA_P_CREATE_TQ: usize = 0x40;       // u64 — runtime
const DATA_P_CREATE_TQT: usize = 0x48;      // u64 — runtime
const DATA_IS_ENCRYPTED: usize = 0x50;      // u64 — runtime flag
const DATA_OLD_PROTECT: usize = 0x58;       // u64 — scratch for VirtualProtect
const DATA_LOCK: usize = 0x60;              // u32 — spinlock (0=unlocked, 1=locked)
const DATA_BLOCK_SIZE: usize = 0x68;

// ── API name strings ──────────────────────────────────────────────────────
const STR_KERNEL32: &[u8] = b"kernel32.dll\0";
const STR_KERNEL32_W: &[u8] = &[
    b'k', 0, b'e', 0, b'r', 0, b'n', 0, b'e', 0, b'l', 0,
    b'3', 0, b'2', 0, b'.', 0, b'd', 0, b'l', 0, b'l', 0, 0, 0,
];
const STR_VIRTUAL_PROTECT: &[u8] = b"VirtualProtect\0";
const STR_ADD_VEH: &[u8] = b"AddVectoredExceptionHandler\0";
const STR_CREATE_TQ: &[u8] = b"CreateTimerQueue\0";
const STR_CREATE_TQT: &[u8] = b"CreateTimerQueueTimer\0";

/// How we bootstrap API resolution.
#[derive(Debug, Clone)]
enum BootstrapMethod {
    /// LoadLibraryA IAT slot available — call directly.
    LoadLibraryA(u32),
    /// GetModuleHandleW IAT slot — call with wide string, then resolve LoadLibraryA via GetProcAddress.
    GetModuleHandleW(u32),
    /// GetModuleHandleA IAT slot — call with ASCII string, then resolve LoadLibraryA via GetProcAddress.
    GetModuleHandleA(u32),
}

struct BootstrapSlots {
    getproc_iat_rva: u32,
    method: BootstrapMethod,
}

/// Helper for emitting x86-64 machine code with RIP-relative addressing.
struct CodeBuilder {
    code: Vec<u8>,
    base_rva: u32,
}

impl CodeBuilder {
    fn new(base_rva: u32) -> Self {
        Self {
            code: Vec::new(),
            base_rva,
        }
    }

    fn pos(&self) -> usize {
        self.code.len()
    }

    fn current_rva(&self) -> u32 {
        self.base_rva + self.code.len() as u32
    }

    fn emit(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    fn rip_disp(&self, insn_size: u32, target_rva: u32) -> i32 {
        let ip_after = self.current_rva() + insn_size;
        (target_rva as i64 - ip_after as i64) as i32
    }

    /// lea rcx, [rip+disp32] — 7 bytes
    fn emit_lea_rcx_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x48, 0x8D, 0x0D]);
        self.emit(&disp.to_le_bytes());
    }

    /// lea rdx, [rip+disp32] — 7 bytes
    fn emit_lea_rdx_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x48, 0x8D, 0x15]);
        self.emit(&disp.to_le_bytes());
    }

    /// lea r8, [rip+disp32] — 7 bytes
    fn emit_lea_r8_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x4C, 0x8D, 0x05]);
        self.emit(&disp.to_le_bytes());
    }

    /// lea r9, [rip+disp32] — 7 bytes
    fn emit_lea_r9_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x4C, 0x8D, 0x0D]);
        self.emit(&disp.to_le_bytes());
    }

    /// mov rcx, [rip+disp32] — 7 bytes
    fn emit_mov_rcx_qword_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x48, 0x8B, 0x0D]);
        self.emit(&disp.to_le_bytes());
    }

    /// mov rdx, [rip+disp32] — 7 bytes
    fn emit_mov_rdx_qword_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x48, 0x8B, 0x15]);
        self.emit(&disp.to_le_bytes());
    }

    /// mov rax, [rip+disp32] — 7 bytes
    fn emit_mov_rax_qword_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x48, 0x8B, 0x05]);
        self.emit(&disp.to_le_bytes());
    }

    /// mov [rip+disp32], rax — 7 bytes
    fn emit_mov_qword_rip_rax(&mut self, target_rva: u32) {
        let disp = self.rip_disp(7, target_rva);
        self.emit(&[0x48, 0x89, 0x05]);
        self.emit(&disp.to_le_bytes());
    }

    /// mov qword [rip+disp32], imm32 — 11 bytes
    fn emit_mov_qword_rip_imm32(&mut self, target_rva: u32, imm: u32) {
        let disp = self.rip_disp(11, target_rva);
        self.emit(&[0x48, 0xC7, 0x05]);
        self.emit(&disp.to_le_bytes());
        self.emit(&imm.to_le_bytes());
    }

    /// cmp qword [rip+disp32], imm8 — 8 bytes
    fn emit_cmp_qword_rip_imm8(&mut self, target_rva: u32, imm: u8) {
        let disp = self.rip_disp(8, target_rva);
        self.emit(&[0x48, 0x83, 0x3D]);
        self.emit(&disp.to_le_bytes());
        self.emit(&[imm]);
    }

    /// call qword [rip+disp32] — 6 bytes
    fn emit_call_qword_rip(&mut self, target_rva: u32) {
        let disp = self.rip_disp(6, target_rva);
        self.emit(&[0xFF, 0x15]);
        self.emit(&disp.to_le_bytes());
    }

    /// call rel32 — 5 bytes
    fn emit_call_rel32(&mut self, target_rva: u32) {
        let disp = self.rip_disp(5, target_rva);
        self.emit(&[0xE8]);
        self.emit(&disp.to_le_bytes());
    }

    /// jmp rel32 — 5 bytes
    fn emit_jmp_rel32(&mut self, target_rva: u32) {
        let disp = self.rip_disp(5, target_rva);
        self.emit(&[0xE9]);
        self.emit(&disp.to_le_bytes());
    }

    /// call qword [rbx+disp32] — 6 bytes  (for IAT access via image base in rbx)
    fn emit_call_qword_rbx_disp32(&mut self, disp32: u32) {
        self.emit(&[0xFF, 0x93]);
        self.emit(&disp32.to_le_bytes());
    }

    /// Patch a byte at an absolute position in the code buffer.
    fn patch_u8(&mut self, offset: usize, val: u8) {
        self.code[offset] = val;
    }

    /// Emit spinlock acquire: atomically sets dword [rip+lock_rva] from 0→1.
    /// Spins with `pause` until successful. 21 bytes.
    fn emit_spinlock_acquire(&mut self, lock_rva: u32) {
        let spin_start = self.pos();
        self.emit(&[0x31, 0xC0]);                         // xor eax, eax (expected=0)
        self.emit(&[0xB9, 0x01, 0x00, 0x00, 0x00]);       // mov ecx, 1 (desired=1)
        // lock cmpxchg dword [rip+disp32], ecx — 8 bytes
        let disp = self.rip_disp(8, lock_rva);
        self.emit(&[0xF0, 0x0F, 0xB1, 0x0D]);
        self.emit(&disp.to_le_bytes());
        self.emit(&[0x74, 0x04]);                         // je .acquired (+4)
        self.emit(&[0xF3, 0x90]);                         // pause
        let rel = (spin_start as isize - (self.pos() as isize + 2)) as i8;
        self.emit(&[0xEB, rel as u8]);                    // jmp .spin
        // .acquired:
    }

    /// Emit spinlock release: mov dword [rip+lock_rva], 0. 10 bytes.
    fn emit_spinlock_release(&mut self, lock_rva: u32) {
        let disp = self.rip_disp(10, lock_rva);
        self.emit(&[0xC7, 0x05]);
        self.emit(&disp.to_le_bytes());
        self.emit(&[0x00, 0x00, 0x00, 0x00]);
    }
}

// ── XOR crypt subroutine (34 bytes) ──────────────────────────────────────
//
// void xor_crypt(void* data, size_t size, void* key16)
// rcx = data, rdx = size, r8 = key_ptr (16 bytes)
// Byte-by-byte XOR with 16-byte repeating key.
//
// 48 85 D2          test rdx, rdx
// 74 1C             jz .done
// 45 31 C9          xor r9d, r9d      ; data offset
// 49 39 D1          cmp r9, rdx       ; .loop:
// 7D 14             jge .done
// 4C 89 C8          mov rax, r9
// 83 E0 0F          and eax, 0xF
// 41 0F B6 04 00    movzx eax, byte [r8+rax]
// 42 30 04 09       xor byte [rcx+r9], al
// 49 FF C1          inc r9
// EB E7             jmp .loop
// C3                ret
const XOR_CRYPT_CODE: [u8; 34] = [
    0x48, 0x85, 0xD2,                         // test rdx, rdx
    0x74, 0x1C,                               // jz .done (+28)
    0x45, 0x31, 0xC9,                         // xor r9d, r9d
    // .loop:
    0x49, 0x39, 0xD1,                         // cmp r9, rdx
    0x7D, 0x14,                               // jge .done (+20)
    0x4C, 0x89, 0xC8,                         // mov rax, r9
    0x83, 0xE0, 0x0F,                         // and eax, 0xF
    0x41, 0x0F, 0xB6, 0x04, 0x00,             // movzx eax, byte [r8+rax]
    0x42, 0x30, 0x04, 0x09,                   // xor byte [rcx+r9], al
    0x49, 0xFF, 0xC1,                         // inc r9
    0xEB, 0xE7,                               // jmp .loop (-25)
    // .done:
    0xC3,                                     // ret
];

/// Generate the timer callback stub.
///
/// Called by Windows timer thread pool:
///   VOID CALLBACK TimerCallback(PVOID param, BOOLEAN fired)
///
/// 1. Acquire spinlock
/// 2. is_encrypted = 1  (set BEFORE removing execute — VEH will see this and spin on lock)
/// 3. VirtualProtect(.text, size, PAGE_READWRITE, &old)
/// 4. xor_crypt(.text, size, &key)
/// 5. VirtualProtect(.text, size, PAGE_READONLY, &old)
/// 6. Release spinlock
fn generate_timer_callback(
    callback_rva: u32,
    data_rva: u32,
    xor_crypt_rva: u32,
) -> Vec<u8> {
    let mut cb = CodeBuilder::new(callback_rva);

    // Prologue — Windows ABI: save non-volatile regs, align stack
    cb.emit(&[0x53]);                     // push rbx
    cb.emit(&[0x55]);                     // push rbp
    cb.emit(&[0x48, 0x89, 0xE5]);         // mov rbp, rsp
    cb.emit(&[0x48, 0x83, 0xE4, 0xF0]);   // and rsp, -16
    cb.emit(&[0x48, 0x83, 0xEC, 0x30]);   // sub rsp, 48

    // Acquire spinlock
    cb.emit_spinlock_acquire(data_rva + DATA_LOCK as u32);

    // is_encrypted = 1 (set before VirtualProtect so VEH knows to wait)
    cb.emit_mov_qword_rip_imm32(data_rva + DATA_IS_ENCRYPTED as u32, 1);

    // VirtualProtect(text_va, text_size, PAGE_READWRITE=4, &old_protect)
    cb.emit_mov_rcx_qword_rip(data_rva + DATA_TEXT_VA as u32);
    cb.emit_mov_rdx_qword_rip(data_rva + DATA_TEXT_SIZE as u32);
    cb.emit(&[0x41, 0xB8, 0x04, 0x00, 0x00, 0x00]); // mov r8d, 4 (PAGE_READWRITE)
    cb.emit_lea_r9_rip(data_rva + DATA_OLD_PROTECT as u32);
    cb.emit_call_qword_rip(data_rva + DATA_P_VIRTUALPROTECT as u32);

    // xor_crypt(text_va, text_size, &xor_key)
    cb.emit_mov_rcx_qword_rip(data_rva + DATA_TEXT_VA as u32);
    cb.emit_mov_rdx_qword_rip(data_rva + DATA_TEXT_SIZE as u32);
    cb.emit_lea_r8_rip(data_rva + DATA_XOR_KEY as u32);
    cb.emit_call_rel32(xor_crypt_rva);

    // VirtualProtect(text_va, text_size, PAGE_READONLY=2, &old_protect)
    cb.emit_mov_rcx_qword_rip(data_rva + DATA_TEXT_VA as u32);
    cb.emit_mov_rdx_qword_rip(data_rva + DATA_TEXT_SIZE as u32);
    cb.emit(&[0x41, 0xB8, 0x02, 0x00, 0x00, 0x00]); // mov r8d, 2 (PAGE_READONLY)
    cb.emit_lea_r9_rip(data_rva + DATA_OLD_PROTECT as u32);
    cb.emit_call_qword_rip(data_rva + DATA_P_VIRTUALPROTECT as u32);

    // Release spinlock
    cb.emit_spinlock_release(data_rva + DATA_LOCK as u32);

    // Epilogue
    cb.emit(&[0x48, 0x89, 0xEC]);         // mov rsp, rbp
    cb.emit(&[0x5D]);                     // pop rbp
    cb.emit(&[0x5B]);                     // pop rbx
    cb.emit(&[0xC3]);                     // ret

    cb.code
}

/// Generate the VEH handler stub.
///
/// LONG CALLBACK VehHandler(PEXCEPTION_POINTERS info)
///   rcx = PEXCEPTION_POINTERS
///
/// 1. Check ExceptionCode == ACCESS_VIOLATION (0xC0000005)
/// 2. Check ExceptionAddress in [text_va, text_va+text_size)
/// 3. Check is_encrypted == 1
/// 4. Acquire spinlock (waits for timer callback to finish if in progress)
/// 5. Decrypt .text, mark PAGE_EXECUTE_READ, is_encrypted = 0
/// 6. Release spinlock
/// 7. Schedule re-encrypt timer
/// 8. Return EXCEPTION_CONTINUE_EXECUTION (-1)
fn generate_veh_handler(
    handler_rva: u32,
    data_rva: u32,
    xor_crypt_rva: u32,
    timer_callback_rva: u32,
    delay_ms: u32,
) -> Vec<u8> {
    let mut cb = CodeBuilder::new(handler_rva);

    // Prologue
    cb.emit(&[0x53]);                     // push rbx
    cb.emit(&[0x55]);                     // push rbp
    cb.emit(&[0x56]);                     // push rsi
    cb.emit(&[0x48, 0x89, 0xE5]);         // mov rbp, rsp
    cb.emit(&[0x48, 0x83, 0xE4, 0xF0]);   // and rsp, -16
    cb.emit(&[0x48, 0x83, 0xEC, 0x60]);   // sub rsp, 96

    // rsi = ExceptionInfo
    cb.emit(&[0x48, 0x89, 0xCE]);         // mov rsi, rcx

    // rax = pExceptionRecord = [rsi]
    cb.emit(&[0x48, 0x8B, 0x06]);         // mov rax, [rsi]

    // Check ExceptionCode == 0xC0000005
    cb.emit(&[0x81, 0x38, 0x05, 0x00, 0x00, 0xC0]); // cmp dword [rax], 0xC0000005
    let jne_search_1 = cb.pos();
    cb.emit(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne .search (placeholder)

    // rdx = ExceptionAddress = [rax+0x10]
    cb.emit(&[0x48, 0x8B, 0x50, 0x10]);   // mov rdx, [rax+0x10]

    // Check rdx >= text_va
    cb.emit_mov_rcx_qword_rip(data_rva + DATA_TEXT_VA as u32);
    cb.emit(&[0x48, 0x39, 0xCA]);         // cmp rdx, rcx
    let jb_search = cb.pos();
    cb.emit(&[0x0F, 0x82, 0x00, 0x00, 0x00, 0x00]); // jb .search (placeholder)

    // Check rdx < text_va + text_size
    cb.emit(&[0x48, 0x03, 0x0D]);         // add rcx, [rip+disp32] (text_size)
    let disp = cb.rip_disp(7, data_rva + DATA_TEXT_SIZE as u32);
    cb.emit(&disp.to_le_bytes());
    cb.emit(&[0x48, 0x39, 0xCA]);         // cmp rdx, rcx
    let jae_search = cb.pos();
    cb.emit(&[0x0F, 0x83, 0x00, 0x00, 0x00, 0x00]); // jae .search (placeholder)

    // Check is_encrypted == 1
    cb.emit_cmp_qword_rip_imm8(data_rva + DATA_IS_ENCRYPTED as u32, 1);
    let jne_search_2 = cb.pos();
    cb.emit(&[0x0F, 0x85, 0x00, 0x00, 0x00, 0x00]); // jne .search (placeholder)

    // ── Acquire spinlock (waits for timer to finish encrypting) ──
    cb.emit_spinlock_acquire(data_rva + DATA_LOCK as u32);

    // ── Decrypt .text ──

    // VirtualProtect(text_va, text_size, PAGE_READWRITE=4, &old)
    cb.emit_mov_rcx_qword_rip(data_rva + DATA_TEXT_VA as u32);
    cb.emit_mov_rdx_qword_rip(data_rva + DATA_TEXT_SIZE as u32);
    cb.emit(&[0x41, 0xB8, 0x04, 0x00, 0x00, 0x00]); // mov r8d, 4
    cb.emit_lea_r9_rip(data_rva + DATA_OLD_PROTECT as u32);
    cb.emit_call_qword_rip(data_rva + DATA_P_VIRTUALPROTECT as u32);

    // xor_crypt(text_va, text_size, &key)
    cb.emit_mov_rcx_qword_rip(data_rva + DATA_TEXT_VA as u32);
    cb.emit_mov_rdx_qword_rip(data_rva + DATA_TEXT_SIZE as u32);
    cb.emit_lea_r8_rip(data_rva + DATA_XOR_KEY as u32);
    cb.emit_call_rel32(xor_crypt_rva);

    // VirtualProtect(text_va, text_size, PAGE_EXECUTE_READ=0x20, &old)
    cb.emit_mov_rcx_qword_rip(data_rva + DATA_TEXT_VA as u32);
    cb.emit_mov_rdx_qword_rip(data_rva + DATA_TEXT_SIZE as u32);
    cb.emit(&[0x41, 0xB8, 0x20, 0x00, 0x00, 0x00]); // mov r8d, 0x20
    cb.emit_lea_r9_rip(data_rva + DATA_OLD_PROTECT as u32);
    cb.emit_call_qword_rip(data_rva + DATA_P_VIRTUALPROTECT as u32);

    // is_encrypted = 0
    cb.emit_mov_qword_rip_imm32(data_rva + DATA_IS_ENCRYPTED as u32, 0);

    // ── Release spinlock ──
    cb.emit_spinlock_release(data_rva + DATA_LOCK as u32);

    // Schedule re-encrypt timer (AFTER releasing lock):
    // CreateTimerQueueTimer(&timer_handle, timer_queue, timer_callback, NULL, delay_ms, 0, 0)
    cb.emit_lea_rcx_rip(data_rva + DATA_TIMER_HANDLE as u32);  // &timer_handle
    cb.emit_mov_rdx_qword_rip(data_rva + DATA_TIMER_QUEUE as u32); // timer_queue
    cb.emit_lea_r8_rip(timer_callback_rva);                       // callback
    cb.emit(&[0x45, 0x31, 0xC9]);         // xor r9d, r9d (NULL parameter)
    // Stack args: [rsp+0x20]=DueTime, [rsp+0x28]=Period=0, [rsp+0x30]=Flags=0
    cb.emit(&[0xC7, 0x44, 0x24, 0x20]);   // mov dword [rsp+0x20], delay_ms
    cb.emit(&delay_ms.to_le_bytes());
    cb.emit(&[0xC7, 0x44, 0x24, 0x28, 0x00, 0x00, 0x00, 0x00]); // mov dword [rsp+0x28], 0
    cb.emit(&[0xC7, 0x44, 0x24, 0x30, 0x00, 0x00, 0x00, 0x00]); // mov dword [rsp+0x30], 0
    cb.emit_call_qword_rip(data_rva + DATA_P_CREATE_TQT as u32);

    // Return EXCEPTION_CONTINUE_EXECUTION = -1
    cb.emit(&[0xB8, 0xFF, 0xFF, 0xFF, 0xFF]); // mov eax, -1
    let jmp_epilog = cb.pos();
    cb.emit(&[0xEB, 0x00]);               // jmp .epilog (placeholder)

    // .search:
    let search_offset = cb.pos();
    cb.emit(&[0x31, 0xC0]);               // xor eax, eax (return 0)

    // .epilog:
    let epilog_offset = cb.pos();
    cb.emit(&[0x48, 0x89, 0xEC]);         // mov rsp, rbp
    cb.emit(&[0x5E]);                     // pop rsi
    cb.emit(&[0x5D]);                     // pop rbp
    cb.emit(&[0x5B]);                     // pop rbx
    cb.emit(&[0xC3]);                     // ret

    // Patch jump targets
    let search_rel = (search_offset as i32) - (jne_search_1 as i32 + 6);
    cb.code[jne_search_1 + 2..jne_search_1 + 6].copy_from_slice(&search_rel.to_le_bytes());

    let search_rel2 = (search_offset as i32) - (jb_search as i32 + 6);
    cb.code[jb_search + 2..jb_search + 6].copy_from_slice(&search_rel2.to_le_bytes());

    let search_rel3 = (search_offset as i32) - (jae_search as i32 + 6);
    cb.code[jae_search + 2..jae_search + 6].copy_from_slice(&search_rel3.to_le_bytes());

    let search_rel4 = (search_offset as i32) - (jne_search_2 as i32 + 6);
    cb.code[jne_search_2 + 2..jne_search_2 + 6].copy_from_slice(&search_rel4.to_le_bytes());

    let epilog_rel = (epilog_offset as u8).wrapping_sub(jmp_epilog as u8 + 2);
    cb.patch_u8(jmp_epilog + 1, epilog_rel);

    cb.code
}

/// Generate the setup stub that runs before the original entry point.
///
/// 1. Compute image base from RIP
/// 2. Get kernel32 handle (via LoadLibraryA or GetModuleHandle)
/// 3. Resolve APIs via GetProcAddress
/// 4. Store text_va = image_base + text_rva
/// 5. Install VEH
/// 6. Create timer queue + schedule initial encrypt
/// 7. Jump to next entry point
fn generate_setup_stub(
    setup_rva: u32,
    data_rva: u32,
    str_kernel32_rva: u32,
    str_kernel32_w_rva: Option<u32>,
    _str_load_library_a_rva: Option<u32>,
    str_virtualprotect_rva: u32,
    str_addveh_rva: u32,
    str_createtq_rva: u32,
    str_createtqt_rva: u32,
    veh_handler_rva: u32,
    timer_callback_rva: u32,
    bootstrap: &BootstrapMethod,
    getproc_iat_rva: u32,
    text_rva: u32,
    delay_ms: u32,
    next_entry_rva: u32,
) -> Vec<u8> {
    let mut cb = CodeBuilder::new(setup_rva);

    // Prologue — save all volatile + callee-saved regs we use
    cb.emit(&[0x51]);                     // push rcx
    cb.emit(&[0x52]);                     // push rdx
    cb.emit(&[0x41, 0x50]);               // push r8
    cb.emit(&[0x41, 0x51]);               // push r9
    cb.emit(&[0x53]);                     // push rbx
    cb.emit(&[0x56]);                     // push rsi
    cb.emit(&[0x55]);                     // push rbp
    cb.emit(&[0x48, 0x89, 0xE5]);         // mov rbp, rsp
    cb.emit(&[0x48, 0x83, 0xE4, 0xF0]);   // and rsp, -16
    cb.emit(&[0x48, 0x83, 0xEC, 0x60]);   // sub rsp, 96

    // Compute image base: lea rbx, [rip+0]; sub rbx, <rva_of_next_insn>
    cb.emit(&[0x48, 0x8D, 0x1D, 0x00, 0x00, 0x00, 0x00]); // lea rbx, [rip+0]
    let next_insn_rva = cb.current_rva();
    cb.emit(&[0x48, 0x81, 0xEB]);
    cb.emit(&next_insn_rva.to_le_bytes()); // rbx = image_base

    // Compute text_va = rbx + text_rva, store it
    cb.emit(&[0x48, 0x8D, 0x83]);         // lea rax, [rbx+disp32]
    cb.emit(&text_rva.to_le_bytes());
    cb.emit_mov_qword_rip_rax(data_rva + DATA_TEXT_VA as u32);

    // ── Get kernel32 handle into rsi ──
    match bootstrap {
        BootstrapMethod::LoadLibraryA(loadlib_iat_rva) => {
            // LoadLibraryA("kernel32.dll") via IAT
            cb.emit_lea_rcx_rip(str_kernel32_rva);
            cb.emit_call_qword_rbx_disp32(*loadlib_iat_rva);
            cb.emit(&[0x48, 0x89, 0xC6]);     // mov rsi, rax
        }
        BootstrapMethod::GetModuleHandleW(getmod_iat_rva) => {
            // GetModuleHandleW(L"kernel32.dll") via IAT
            cb.emit_lea_rcx_rip(str_kernel32_w_rva.unwrap());
            cb.emit_call_qword_rbx_disp32(*getmod_iat_rva);
            cb.emit(&[0x48, 0x89, 0xC6]);     // mov rsi, rax
        }
        BootstrapMethod::GetModuleHandleA(getmod_iat_rva) => {
            // GetModuleHandleA("kernel32.dll") via IAT
            cb.emit_lea_rcx_rip(str_kernel32_rva);
            cb.emit_call_qword_rbx_disp32(*getmod_iat_rva);
            cb.emit(&[0x48, 0x89, 0xC6]);     // mov rsi, rax
        }
    }

    // ── Resolve APIs via GetProcAddress ──

    // GetProcAddress(kernel32, "VirtualProtect")
    cb.emit(&[0x48, 0x89, 0xF1]);         // mov rcx, rsi
    cb.emit_lea_rdx_rip(str_virtualprotect_rva);
    cb.emit_call_qword_rbx_disp32(getproc_iat_rva);
    cb.emit_mov_qword_rip_rax(data_rva + DATA_P_VIRTUALPROTECT as u32);

    // GetProcAddress(kernel32, "AddVectoredExceptionHandler")
    cb.emit(&[0x48, 0x89, 0xF1]);         // mov rcx, rsi
    cb.emit_lea_rdx_rip(str_addveh_rva);
    cb.emit_call_qword_rbx_disp32(getproc_iat_rva);
    cb.emit_mov_qword_rip_rax(data_rva + DATA_P_ADD_VEH as u32);

    // GetProcAddress(kernel32, "CreateTimerQueue")
    cb.emit(&[0x48, 0x89, 0xF1]);         // mov rcx, rsi
    cb.emit_lea_rdx_rip(str_createtq_rva);
    cb.emit_call_qword_rbx_disp32(getproc_iat_rva);
    cb.emit_mov_qword_rip_rax(data_rva + DATA_P_CREATE_TQ as u32);

    // GetProcAddress(kernel32, "CreateTimerQueueTimer")
    cb.emit(&[0x48, 0x89, 0xF1]);         // mov rcx, rsi
    cb.emit_lea_rdx_rip(str_createtqt_rva);
    cb.emit_call_qword_rbx_disp32(getproc_iat_rva);
    cb.emit_mov_qword_rip_rax(data_rva + DATA_P_CREATE_TQT as u32);

    // ── Install VEH ──
    cb.emit(&[0xB9, 0x01, 0x00, 0x00, 0x00]); // mov ecx, 1
    cb.emit_lea_rdx_rip(veh_handler_rva);
    cb.emit_call_qword_rip(data_rva + DATA_P_ADD_VEH as u32);

    // ── Create timer queue ──
    cb.emit_call_qword_rip(data_rva + DATA_P_CREATE_TQ as u32);
    cb.emit_mov_qword_rip_rax(data_rva + DATA_TIMER_QUEUE as u32);

    // ── Schedule initial encrypt timer ──
    cb.emit_lea_rcx_rip(data_rva + DATA_TIMER_HANDLE as u32);
    cb.emit(&[0x48, 0x89, 0xC2]);         // mov rdx, rax (timer_queue from above)
    cb.emit_lea_r8_rip(timer_callback_rva);
    cb.emit(&[0x45, 0x31, 0xC9]);         // xor r9d, r9d
    cb.emit(&[0xC7, 0x44, 0x24, 0x20]);   // mov dword [rsp+0x20], delay_ms
    cb.emit(&delay_ms.to_le_bytes());
    cb.emit(&[0xC7, 0x44, 0x24, 0x28, 0x00, 0x00, 0x00, 0x00]); // Period = 0
    cb.emit(&[0xC7, 0x44, 0x24, 0x30, 0x00, 0x00, 0x00, 0x00]); // Flags = 0
    cb.emit_call_qword_rip(data_rva + DATA_P_CREATE_TQT as u32);

    // ── Epilogue ──
    cb.emit(&[0x48, 0x89, 0xEC]);         // mov rsp, rbp
    cb.emit(&[0x5D]);                     // pop rbp
    cb.emit(&[0x5E]);                     // pop rsi
    cb.emit(&[0x5B]);                     // pop rbx
    cb.emit(&[0x41, 0x59]);               // pop r9
    cb.emit(&[0x41, 0x58]);               // pop r8
    cb.emit(&[0x5A]);                     // pop rdx
    cb.emit(&[0x59]);                     // pop rcx
    cb.emit_jmp_rel32(next_entry_rva);

    cb.code
}

/// Find bootstrap IAT slots by parsing imports.
///
/// Looks for GetProcAddress (required) and one of:
///   LoadLibraryA, GetModuleHandleW, GetModuleHandleA (in order of preference).
fn find_bootstrap_iat_slots(data: &[u8], pe: &PeFile) -> Result<BootstrapSlots> {
    let dir_offset = pe.data_directory_offset + 1 * 8; // import directory
    if dir_offset + 8 > data.len() {
        anyhow::bail!("No import directory");
    }
    let import_rva = u32::from_le_bytes(data[dir_offset..dir_offset + 4].try_into().unwrap());
    let import_size = u32::from_le_bytes(data[dir_offset + 4..dir_offset + 8].try_into().unwrap());
    if import_rva == 0 || import_size == 0 {
        anyhow::bail!("Empty import directory");
    }

    let import_file_offset = pe.sections.iter().find_map(|s| s.rva_to_offset(import_rva))
        .context("Cannot find import directory")?;

    let mut loadlib_a = None;
    let mut getproc = None;
    let mut getmod_w = None;
    let mut getmod_a = None;

    let mut pos = import_file_offset;
    loop {
        if pos + 20 > data.len() { break; }
        let ilt_rva = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        let name_rva = u32::from_le_bytes(data[pos + 12..pos + 16].try_into().unwrap());
        let iat_rva = u32::from_le_bytes(data[pos + 16..pos + 20].try_into().unwrap());
        if ilt_rva == 0 && name_rva == 0 && iat_rva == 0 { break; }

        let lookup_rva = if ilt_rva != 0 { ilt_rva } else { iat_rva };
        let lookup_offset = pe.sections.iter().find_map(|s| s.rva_to_offset(lookup_rva))
            .context("Cannot find ILT/IAT")?;

        let mut idx = 0u32;
        loop {
            let entry_off = lookup_offset + (idx as usize) * 8;
            if entry_off + 8 > data.len() { break; }
            let entry = u64::from_le_bytes(data[entry_off..entry_off + 8].try_into().unwrap());
            if entry == 0 { break; }

            if entry & (1u64 << 63) == 0 {
                let hint_rva = (entry & 0x7FFFFFFF) as u32;
                if let Some(hint_off) = pe.sections.iter().find_map(|s| s.rva_to_offset(hint_rva)) {
                    let mut end = hint_off + 2;
                    while end < data.len() && data[end] != 0 { end += 1; }
                    let name = std::str::from_utf8(&data[hint_off + 2..end]).unwrap_or("");
                    let slot_rva = iat_rva + idx * 8;
                    match name {
                        "LoadLibraryA" => loadlib_a = Some(slot_rva),
                        "GetProcAddress" => getproc = Some(slot_rva),
                        "GetModuleHandleW" => getmod_w = Some(slot_rva),
                        "GetModuleHandleA" => getmod_a = Some(slot_rva),
                        _ => {}
                    }
                }
            }
            idx += 1;
        }
        pos += 20;
    }

    let getproc_iat_rva = getproc.context("Binary does not import GetProcAddress")?;

    let method = if let Some(rva) = loadlib_a {
        BootstrapMethod::LoadLibraryA(rva)
    } else if let Some(rva) = getmod_w {
        log::info!("No LoadLibraryA — using GetModuleHandleW for bootstrap");
        BootstrapMethod::GetModuleHandleW(rva)
    } else if let Some(rva) = getmod_a {
        log::info!("No LoadLibraryA — using GetModuleHandleA for bootstrap");
        BootstrapMethod::GetModuleHandleA(rva)
    } else {
        anyhow::bail!(
            "Binary does not import LoadLibraryA, GetModuleHandleW, or GetModuleHandleA"
        );
    };

    Ok(BootstrapSlots {
        getproc_iat_rva,
        method,
    })
}

/// Apply PE Fluctuation to an already-written PE output buffer.
///
/// Installs a VEH + timer-based system that encrypts .text when idle
/// and decrypts on-demand when code executes. At runtime:
///   1. Setup resolves APIs, installs VEH, creates encrypt timer
///   2. After `delay_ms`, timer fires: XOR-encrypts .text, marks PAGE_READONLY
///   3. Code execution in .text faults → VEH decrypts, marks PAGE_EXECUTE_READ
///   4. VEH schedules another encrypt timer → cycle repeats
///
/// Returns true if fluctuation was applied.
pub fn apply_pe_fluctuation(
    output: &mut Vec<u8>,
    pe: &PeFile,
    seed: u64,
    delay_ms: u32,
) -> Result<bool> {
    // Find .text section
    let text_section = pe.sections.iter()
        .find(|s| s.name == ".text")
        .context("No .text section found")?;
    let text_rva = text_section.virtual_address;
    let text_size = text_section.raw_size as u64;

    if text_size == 0 {
        log::info!("Empty .text section — skipping PE fluctuation");
        return Ok(false);
    }

    // Find bootstrap IAT slots
    let bootstrap = find_bootstrap_iat_slots(output, pe)?;
    log::info!(
        "Fluctuation bootstrap: {:?}, GetProcAddress=0x{:x}",
        bootstrap.method, bootstrap.getproc_iat_rva,
    );

    // Generate XOR key
    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(0xF1C7_A7E0));
    let mut xor_key = [0u8; 16];
    rng.fill(&mut xor_key);
    // Ensure no zero bytes in key (would leave bytes unencrypted)
    for b in &mut xor_key {
        if *b == 0 { *b = rng.gen_range(1..=255u8); }
    }

    // ── Layout computation ──
    let last = &pe.sections[pe.sections.len() - 1];
    let sh = last.header_offset;
    let current_raw_size = u32::from_le_bytes(output[sh + 16..sh + 20].try_into().unwrap());
    let current_virtual_size = u32::from_le_bytes(output[sh + 8..sh + 12].try_into().unwrap());
    let current_chars = u32::from_le_bytes(output[sh + 36..sh + 40].try_into().unwrap());

    let base_rva = align_up(last.virtual_address + current_raw_size, 16);
    let padding_before = base_rva - (last.virtual_address + current_raw_size);

    // Layout: [data_block][strings][xor_crypt][timer_callback][veh_handler][setup_stub]
    let data_rva = base_rva;

    let strings_rva = data_rva + DATA_BLOCK_SIZE as u32;
    let mut strings_blob = Vec::new();
    let str_kernel32_off = strings_blob.len();
    strings_blob.extend_from_slice(STR_KERNEL32);
    // Wide kernel32 string (for GetModuleHandleW fallback)
    let str_kernel32_w_off = if matches!(bootstrap.method, BootstrapMethod::GetModuleHandleW(_)) {
        let off = strings_blob.len();
        strings_blob.extend_from_slice(STR_KERNEL32_W);
        Some(off)
    } else {
        None
    };
    // "LoadLibraryA" string (for GetModuleHandle fallback — not used in fluctuation but reserved)
    let str_load_library_a_off: Option<usize> = None;
    let str_vp_off = strings_blob.len();
    strings_blob.extend_from_slice(STR_VIRTUAL_PROTECT);
    let str_aveh_off = strings_blob.len();
    strings_blob.extend_from_slice(STR_ADD_VEH);
    let str_ctq_off = strings_blob.len();
    strings_blob.extend_from_slice(STR_CREATE_TQ);
    let str_ctqt_off = strings_blob.len();
    strings_blob.extend_from_slice(STR_CREATE_TQT);
    // Pad to 8-byte alignment
    while strings_blob.len() % 8 != 0 {
        strings_blob.push(0);
    }

    let xor_crypt_rva = strings_rva + strings_blob.len() as u32;

    let timer_cb_rva = xor_crypt_rva + XOR_CRYPT_CODE.len() as u32;
    let timer_cb_code = generate_timer_callback(timer_cb_rva, data_rva, xor_crypt_rva);

    let veh_rva = timer_cb_rva + timer_cb_code.len() as u32;
    let veh_code = generate_veh_handler(veh_rva, data_rva, xor_crypt_rva, timer_cb_rva, delay_ms);

    let setup_rva = veh_rva + veh_code.len() as u32;

    // Read current entry point from the output buffer (may have been patched by string enc / import hiding)
    let entry_point_file_offset = pe.data_directory_offset - 96;
    let current_entry_rva = u32::from_le_bytes(
        output[entry_point_file_offset..entry_point_file_offset + 4].try_into().unwrap(),
    );

    let setup_code = generate_setup_stub(
        setup_rva,
        data_rva,
        strings_rva + str_kernel32_off as u32,
        str_kernel32_w_off.map(|off| strings_rva + off as u32),
        str_load_library_a_off.map(|off| strings_rva + off as u32),
        strings_rva + str_vp_off as u32,
        strings_rva + str_aveh_off as u32,
        strings_rva + str_ctq_off as u32,
        strings_rva + str_ctqt_off as u32,
        veh_rva,
        timer_cb_rva,
        &bootstrap.method,
        bootstrap.getproc_iat_rva,
        text_rva,
        delay_ms,
        current_entry_rva,
    );

    let total_code_size = DATA_BLOCK_SIZE as u32
        + strings_blob.len() as u32
        + XOR_CRYPT_CODE.len() as u32
        + timer_cb_code.len() as u32
        + veh_code.len() as u32
        + setup_code.len() as u32;

    let total_append = padding_before + total_code_size;

    // ── Extend last section ──
    let new_raw_size = align_up(current_raw_size + total_append, pe.file_alignment);
    let new_virtual_size = std::cmp::max(current_virtual_size, new_raw_size);
    let append_file_offset = (last.raw_offset + current_raw_size) as usize;
    let extension_bytes = (new_raw_size - current_raw_size) as usize;

    if append_file_offset >= output.len() {
        output.resize(append_file_offset + extension_bytes, 0);
    } else {
        let tail = output[append_file_offset..].to_vec();
        output.truncate(append_file_offset);
        output.resize(append_file_offset + extension_bytes, 0);
        output.extend_from_slice(&tail);
    }

    // Write data block
    let data_file_offset = append_file_offset + padding_before as usize;
    output[data_file_offset + DATA_XOR_KEY..data_file_offset + DATA_XOR_KEY + 16]
        .copy_from_slice(&xor_key);
    // text_va is computed at runtime; text_size is set here
    output[data_file_offset + DATA_TEXT_SIZE..data_file_offset + DATA_TEXT_SIZE + 8]
        .copy_from_slice(&text_size.to_le_bytes());

    // Write strings
    let strings_file_offset = data_file_offset + DATA_BLOCK_SIZE;
    output[strings_file_offset..strings_file_offset + strings_blob.len()]
        .copy_from_slice(&strings_blob);

    // Write xor_crypt
    let xor_crypt_file_offset = strings_file_offset + strings_blob.len();
    output[xor_crypt_file_offset..xor_crypt_file_offset + XOR_CRYPT_CODE.len()]
        .copy_from_slice(&XOR_CRYPT_CODE);

    // Write timer callback
    let timer_cb_file_offset = xor_crypt_file_offset + XOR_CRYPT_CODE.len();
    output[timer_cb_file_offset..timer_cb_file_offset + timer_cb_code.len()]
        .copy_from_slice(&timer_cb_code);

    // Write VEH handler
    let veh_file_offset = timer_cb_file_offset + timer_cb_code.len();
    output[veh_file_offset..veh_file_offset + veh_code.len()]
        .copy_from_slice(&veh_code);

    // Write setup stub
    let setup_file_offset = veh_file_offset + veh_code.len();
    output[setup_file_offset..setup_file_offset + setup_code.len()]
        .copy_from_slice(&setup_code);

    // ── Update section headers ──
    output[sh + 8..sh + 12].copy_from_slice(&new_virtual_size.to_le_bytes());
    output[sh + 16..sh + 20].copy_from_slice(&new_raw_size.to_le_bytes());
    let new_chars = (current_chars & !IMAGE_SCN_MEM_DISCARDABLE)
        | IMAGE_SCN_MEM_EXECUTE
        | IMAGE_SCN_MEM_WRITE
        | IMAGE_SCN_CNT_CODE;
    output[sh + 36..sh + 40].copy_from_slice(&new_chars.to_le_bytes());

    // Update SizeOfImage
    let new_size_of_image = align_up(last.virtual_address + new_virtual_size, pe.section_alignment);
    output[pe.size_of_image_offset..pe.size_of_image_offset + 4]
        .copy_from_slice(&new_size_of_image.to_le_bytes());

    // Update SizeOfCode
    let old_size_of_code = u32::from_le_bytes(
        output[pe.size_of_code_offset..pe.size_of_code_offset + 4].try_into().unwrap(),
    );
    let new_size_of_code = old_size_of_code + (new_raw_size - current_raw_size);
    output[pe.size_of_code_offset..pe.size_of_code_offset + 4]
        .copy_from_slice(&new_size_of_code.to_le_bytes());

    // Hook entry point to setup stub
    output[entry_point_file_offset..entry_point_file_offset + 4]
        .copy_from_slice(&setup_rva.to_le_bytes());

    // Zero checksum
    output[pe.checksum_offset..pe.checksum_offset + 4].fill(0);

    log::info!(
        "PE fluctuation: .text RVA=0x{:x} size=0x{:x}, setup at 0x{:x}, entry 0x{:x} -> 0x{:x}, delay={}ms",
        text_rva, text_size, setup_rva, current_entry_rva, setup_rva, delay_ms,
    );

    Ok(true)
}
