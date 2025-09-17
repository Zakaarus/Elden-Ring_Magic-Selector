use std::env::current_exe;
use std::ffi::c_void;
use std::time;
use std::thread;
use std::time::Duration;
use eldenring::cs::CSTaskGroupIndex;
use eldenring::cs::CSTaskImp;
use eldenring::cs::WorldChrMan;
use eldenring::fd4::FD4TaskData;
use eldenring_util::program::Program;
use eldenring_util::singleton::get_instance;
use eldenring_util::system::wait_for_system_init;
use eldenring_util::task::CSTaskImpExt;
use eldenring_util::*;
use libmem::*;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::DisableThreadLibraryCalls;
use windows::Win32::System::Threading::QueueUserWorkItem;
use windows::Win32::System::Threading::WT_EXECUTEDEFAULT;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn DllMain(hmodule: HMODULE, reason: u32) -> bool
{
    if reason != 1 {return true;}
    let _ = unsafe { DisableThreadLibraryCalls(hmodule) };
    let _ = unsafe { QueueUserWorkItem(Some(dll_thread), Some(std::ptr::null_mut()), WT_EXECUTEDEFAULT) };
    return true;
}

//WARNING THIS CODE IS A MESS, IM JUST THROWING IT OUT SO I HAVE SOMETHING THAT WORKS. I WILL FIX IT EVENTUALLY.

struct TaskState
{
    this_module:Module,
    this_process:Process,
    aob: &'static str,
    pointer: usize,
    read_result: usize,
    first_offset:usize,
    other_offsets:[usize;2],
}

unsafe extern "system" fn dll_thread(_:*mut c_void) -> u32
{
    
    wait_for_system_init(&Program::current(), Duration::MAX)
        .expect("Timeout waiting for system init");

    let mut frame_begin_state = TaskState{
        this_module : find_module(&current_exe().unwrap().into_os_string().into_string().unwrap()).unwrap(),
        this_process : find_process(&current_exe().unwrap().into_os_string().into_string().unwrap()).unwrap(),
        aob : "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 05 48 8B 40 58 C3 C3",
        pointer : 0,
        read_result : 0,
        first_offset:0x08,
        other_offsets:[0x530,0x80],
    };

    unsafe
    {   //trust me you don't want to know. *I* don't want to know. Don't judge this is literally a rushed prototype
        let my_address:Address = sig_scan(frame_begin_state.aob, frame_begin_state.this_module.base, frame_begin_state.this_module.size).unwrap();
        let my_inst = disassemble(my_address).unwrap();
        println!("{:#X?}", my_inst);
        let mut _testing2:String = my_inst.op_str;
        println!("{}",_testing2);
        let _testing3 = _testing2.find("0x").unwrap();
        _testing2.drain(0.._testing3+2);
        _testing2.pop(); 
        let _testing5 = usize::from_str_radix(&_testing2, 16).unwrap();
        let mut test_read:usize = 0;
        while  !(test_read > 0)
        {
            thread::sleep(time::Duration::from_millis(5000));
            test_read = read_memory(_testing5+my_address+my_inst.bytes.len());
        }
        frame_begin_state.read_result = test_read;

    }

    // Retrieve games task runner and register a task at frame begin.
    let cs_task = unsafe { get_instance::<CSTaskImp>().unwrap().unwrap() };
    cs_task.run_recurring(
        move|_: &FD4TaskData| {frame_begin(&mut frame_begin_state);},
        CSTaskGroupIndex::FrameBegin,
    );
    return 1;
}

fn change_memory(frame_begin_state:&mut TaskState, replacement:u8)
{
    unsafe
    {
        if frame_begin_state.read_result > 0
        {
            frame_begin_state.pointer = resolve_pointer_chain(&frame_begin_state.this_process,frame_begin_state.read_result+frame_begin_state.first_offset,&frame_begin_state.other_offsets).unwrap();
            if 0x7FFFFFFFFFFF > frame_begin_state.pointer && frame_begin_state.pointer > 0x1000
            {
                set_memory(frame_begin_state.pointer,replacement,1);
            }
        }
    }
}


fn frame_begin(frame_begin_state:&mut TaskState){
    {
        // Retrieve WorldChrMan
        let Some(world_chr_man) = unsafe { get_instance::<WorldChrMan>() }.unwrap() else {
            return;
        };

        // Retrieve main player
        let Some(ref mut main_player) = world_chr_man.main_player else {
            return;
        };

        for n in 0..9{
            if input::is_key_pressed(0x31+n) {
                change_memory(frame_begin_state, n as u8);
            }
        }
    }
}

pub fn resolve_pointer_chain(
    process: &Process,
    mut addr: usize,
    offsets: &[usize],
) -> Option<usize> {
    for (_, offset) in offsets.iter().enumerate() {
        let ptr = libmem::read_memory_ex::<usize>(process, addr)?;
        // leaving this here for debugging purposes
        // println!(
        //     "Step {}: Read 0x{:X} -> 0x{:X} + 0x{:X}",
        //     i, addr, ptr, offset
        // );
        addr = ptr + offset;
    }
    Some(addr)
}
