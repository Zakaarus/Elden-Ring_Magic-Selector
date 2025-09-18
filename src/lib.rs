use std::ffi::c_void;
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

unsafe extern "system" fn dll_thread(_:*mut c_void) -> u32
{
    
    wait_for_system_init(&Program::current(), Duration::MAX)
        .expect("Timeout waiting for system init");

    // Retrieve games task runner and register a task at frame begin.
    let cs_task = unsafe { get_instance::<CSTaskImp>().unwrap().unwrap() };
    cs_task.run_recurring(
        move|_: &FD4TaskData| {frame_begin();},
        CSTaskGroupIndex::FrameBegin,
    );
    return 1;
}



fn frame_begin(){
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
                main_player.player_game_data.equipment.equip_magic_data.selected_slot = n;
            }
        }
    }
}
