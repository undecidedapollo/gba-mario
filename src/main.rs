#![no_std]
#![no_main]
#![feature(maybe_uninit_array_assume_init)]

use core::{fmt::Write, ptr::copy_nonoverlapping};
use gba::prelude::*;
use mario::{
    assets::{
        AFFINE2_SCREENBLOCK_START, AssetManager, BACKGROUND_TILES, COIN_TILE, COIN_TILE_IDX_START,
        TEXT_SCREENBLOCK_START,
    },
    gba_warning,
    keys::KeysManager,
    level_manager::LevelManager,
    logger,
    player::PlayerManager,
    screen::ScreenManager,
    tick::TickContext,
    topbar::TopBarManager,
};

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Ok(mut logger) = MgbaBufferedLogger::try_new(MgbaMessageLevel::Fatal) {
        writeln!(logger, "{info}").ok();
    }
    loop {}
}

#[unsafe(link_section = ".iwram")]
extern "C" fn irq_handler(_b: IrqBits) {
    // if b.vblank() {
    //     // We'll read the keys during vblank and store it for later.
    //     PREV_KEYS.write(FRAME_KEYS.read());
    //     FRAME_KEYS.write(KEYINPUT.read());
    // }
}

#[unsafe(no_mangle)]
extern "C" fn main() -> ! {
    logger::init_logger();

    RUST_IRQ_HANDLER.write(Some(irq_handler));
    DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));
    IE.write(IrqBits::VBLANK);
    IME.write(true);

    VBlankIntrWait();

    DISPCNT.write(
        DisplayControl::new()
            .with_video_mode(VideoMode::_1)
            .with_obj_vram_1d(true)
            .with_show_bg2(true)
            .with_show_bg1(true)
            .with_show_obj(true),
    );
    BG1CNT.write(
        BackgroundControl::new()
            .with_size(1)
            .with_screenblock(TEXT_SCREENBLOCK_START as u16)
            .with_bpp8(false)
            .with_charblock(1),
    );
    BG2CNT.write(
        BackgroundControl::new()
            .with_size(2)
            .with_screenblock(AFFINE2_SCREENBLOCK_START as u16)
            .with_bpp8(true)
            .with_charblock(0)
            .with_mosaic(true)
            .with_is_affine_wrapping(true),
    );

    AssetManager::on_start();
    ScreenManager::on_start();
    PlayerManager::on_start();
    LevelManager::on_start();

    unsafe {
        copy_nonoverlapping(
            BACKGROUND_TILES.0.as_ptr(),
            CHARBLOCK0_8BPP.index(1).as_usize() as *mut u8,
            BACKGROUND_TILES.0.len(),
        );
        copy_nonoverlapping(
            COIN_TILE.0.as_ptr(),
            OBJ_TILES.index(COIN_TILE_IDX_START * 2).as_usize() as *mut u8,
            COIN_TILE.0.len(),
        );
    }

    let mut loop_counter: u32 = 0;

    loop {
        VBlankIntrWait();
        let keys = KeysManager::on_vblank();
        let tick_ctx = TickContext {
            tick_count: loop_counter,
            keys,
        };
        loop_counter = loop_counter.wrapping_add(1);

        TIMER0_RELOAD.write(0);
        TIMER1_RELOAD.write(0);
        TIMER0_CONTROL.write(
            TimerControl::new()
                .with_enabled(true)
                .with_scale(TimerScale::_1),
        );
        TIMER1_CONTROL.write(TimerControl::new().with_enabled(true).with_cascade(true));

        LevelManager::tick(tick_ctx);
        PlayerManager::tick(tick_ctx);
        TopBarManager::tick(tick_ctx);
        ScreenManager::post_tick();
        AssetManager::post_tick();
        let after0 = TIMER0_COUNT.read();
        let after1 = TIMER1_COUNT.read();
        gba_warning!("TIMER0: {after0}, TIMER1: {after1} TICK: {loop_counter}");
        TIMER0_CONTROL.write(TimerControl::new());
        TIMER1_CONTROL.write(TimerControl::new());
    }
}
