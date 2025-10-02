#![no_std]
#![no_main]
#![feature(maybe_uninit_array_assume_init)]

use core::{fmt::Write, ptr::copy_nonoverlapping};
use gba::prelude::*;
use mario::{
    assets::{self, BACKGROUND_TILES, COIN_TILE, COIN_TILE_IDX_START},
    keys::FRAME_KEYS,
    level::LevelManager,
    logger,
    player::PlayerManager,
    score::ScoreManager,
    screen::ScreenManager,
    tick::TickContext,
};

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Ok(mut logger) = MgbaBufferedLogger::try_new(MgbaMessageLevel::Fatal) {
        writeln!(logger, "{info}").ok();
    }
    loop {}
}

#[unsafe(link_section = ".iwram")]
extern "C" fn irq_handler(b: IrqBits) {
    if b.vblank() {
        // We'll read the keys during vblank and store it for later.
        FRAME_KEYS.write(KEYINPUT.read());
    }
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
            .with_show_bg1(false)
            .with_show_obj(true),
    );
    BG2CNT.write(
        BackgroundControl::new()
            .with_size(2)
            .with_screenblock(16)
            .with_bpp8(true)
            .with_charblock(0)
            .with_mosaic(true)
            .with_is_affine_wrapping(true),
    );

    assets::reset_data();
    ScreenManager::init();
    PlayerManager::init();
    LevelManager::init();
    PlayerManager::on_start();

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
        let tick_ctx = TickContext {
            tick_count: loop_counter,
        };
        loop_counter = loop_counter.wrapping_add(1);
        // let new_x = orig_x.div(i16fx8::from_bits(val as i16));
        // let new_y = orig_y.div(i16fx8::from_bits(val as i16));

        // otr.set_x(new_x.to_bits() as u16 >> 8);
        // otr.set_y(new_y.to_bits() as u16 >> 8);
        // OBJ_ATTR_ALL.index(0).write(otr);

        VBlankIntrWait();

        // BG2PA.write(i16fx8::from_bits(val as i16));
        // BG2PB.write(i16fx8::default());
        // BG2PC.write(i16fx8::default());
        // BG2PD.write(i16fx8::from_bits(val as i16));
        // AFFINE_PARAM_A.index(0).write(i16fx8::from_bits(val as i16));
        // AFFINE_PARAM_B.index(0).write(i16fx8::from_bits(0));
        // AFFINE_PARAM_C.index(0).write(i16fx8::from_bits(0));
        // AFFINE_PARAM_D.index(0).write(i16fx8::from_bits(val as i16));
        LevelManager::tick(tick_ctx);
        PlayerManager::tick();
        ScoreManager::tick();
        ScreenManager::post_tick();
    }
}
