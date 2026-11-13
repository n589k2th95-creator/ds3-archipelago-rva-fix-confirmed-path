use std::time::Duration;

use anyhow::{Context, Result};
use darksouls3::{
    sprj::{MapItemMan, MenuMan, SprjTaskGroupIndex, SprjTaskImp},
    util::system::wait_for_system_init,
};
use fromsoftware_shared::{FromStatic, Program, SharedTaskImpExt};
use log::info;

pub struct DarkSoulsIII;

impl shared::Game for DarkSoulsIII {
    type Core = crate::core::Core;
    type GraphicsHooks = hudhook::hooks::dx11::ImguiDx11Hooks;
    type InputBlocker = DS3InputBlocker;
    const TYPE: shared::GameType = shared::GameType::DarkSoulsIII;
    const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

    fn run_recurring_task(mut task: impl FnMut() + 'static + Send) -> Result<()> {
        // Do not use SprjTaskImp::wait_for_instance here. On some Wine/Proton
        // environments that combined helper can fail while translating one of
        // the task-runner RVAs, producing the otherwise-unhelpful
        // "Could not translate RVA to VA" initialization error.
        //
        // Follow the explicit initialization sequence used by the later
        // upstream DS3 client: wait until the game has populated its system
        // reflection data, then resolve the task-runner singleton separately.
        info!("Waiting for Dark Souls III system initialization...");
        wait_for_system_init(&Program::current(), Duration::MAX)
            .context("Waiting for Dark Souls III system initialization failed")?;

        info!("DS3 system initialization completed; resolving SprjTaskImp...");
        let task_manager = unsafe { SprjTaskImp::instance() }.context(
            "Resolving the DS3 SprjTaskImp singleton after system initialization failed",
        )?;

        info!("SprjTaskImp resolved; registering Archipelago frame task...");
        task_manager.run_recurring(move |_: &'_ usize| task(), SprjTaskGroupIndex::FrameBegin);
        info!("Archipelago frame task registered.");
        Ok(())
    }

    unsafe fn is_main_menu() -> bool {
        // If MapItemMan isn't available, that usually means we're on the
        // main menu. There's probably a better way to detect that but we
        // don't know it yet.
        unsafe { MapItemMan::instance() }.is_err()
    }

    unsafe fn force_cursor_visible() {
        if let Ok(man) = unsafe { MenuMan::instance_mut() } {
            man.set_menu_mode(true);
        }
    }

    unsafe fn is_menu_open() -> bool {
        unsafe { Self::is_main_menu() || MenuMan::instance().is_ok_and(|mm| mm.is_menu_mode()) }
    }
}

pub struct DS3InputBlocker(pub &'static darksouls3_extra::input::InputBlocker);

impl shared::InputBlocker for DS3InputBlocker {
    fn block_only(&self, input_flags: shared::InputFlags) {
        self.0
            .block_only(darksouls3_extra::input::InputFlags::from_bits(input_flags.bits()).unwrap())
    }
}
