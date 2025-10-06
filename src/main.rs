#![no_std]
#![no_main]
#![feature(maybe_uninit_array_assume_init)]

use core::{fmt::Write, ptr::copy_nonoverlapping};
use gba::prelude::*;
use mario::{
    assets::{
        self, AFFINE2_SCREENBLOCK_START, BACKGROUND_TILES, COIN_TILE, COIN_TILE_IDX_START,
        TEXT_SCREENBLOCK_START,
    },
    color::make_color,
    gba_warning,
    keys::FRAME_KEYS,
    level_manager::LevelManager,
    levels::shared::{PIPE_BODY_LEFT, PIPE_BODY_RIGHT, PIPE_TOP_LEFT, PIPE_TOP_RIGHT},
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
extern "C" fn irq_handler(b: IrqBits) {
    if b.vblank() {
        // We'll read the keys during vblank and store it for later.
        FRAME_KEYS.write(KEYINPUT.read());
    }
}

fn darken_rgb15(color: Color, factor: i32fx8) -> Color {
    let factor = if factor <= i32fx8::from_bits(0) {
        i32fx8::wrapping_from(0)
    } else if factor >= i32fx8::wrapping_from(1) {
        i32fx8::wrapping_from(1)
    } else {
        factor
    };
    // Extract 5-bit channels
    let r = i32fx8::wrapping_from((color.0 & 0x1F) as i32);
    let g = i32fx8::wrapping_from(((color.0 >> 5) & 0x1F) as i32);
    let b = i32fx8::wrapping_from(((color.0 >> 10) & 0x1F) as i32);

    // Scale them down by your darkening factor (e.g., 0.7 = 70% brightness)
    let r = ((r * factor).to_bits() >> 8) as u16;
    let g = ((g * factor).to_bits() >> 8) as u16;
    let b = ((b * factor).to_bits() >> 8) as u16;

    // Repack into RGB15 format
    make_color(r, g, b)
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

    let magic_max = Color(0x127c);
    let magic_1 = darken_rgb15(magic_max, i32fx8::from_bits(230));
    let magic_2 = darken_rgb15(magic_max, i32fx8::from_bits(180));
    let magic_3 = darken_rgb15(magic_max, i32fx8::from_bits(130));
    let mut change_pixel = 0;

    loop {
        let tick_ctx = TickContext {
            tick_count: loop_counter,
        };
        loop_counter = loop_counter.wrapping_add(1);

        if change_pixel == 0 {
            BG_PALETTE.index(1).write(magic_max);
        } else if change_pixel == 8 {
            BG_PALETTE.index(1).write(magic_1);
        } else if change_pixel == 16 {
            BG_PALETTE.index(1).write(magic_2);
        } else if change_pixel == 24 {
            BG_PALETTE.index(1).write(magic_3);
        } else if change_pixel == 32 {
            BG_PALETTE.index(1).write(magic_2);
        } else if change_pixel == 40 {
            BG_PALETTE.index(1).write(magic_1);
        } else if change_pixel == 48 {
            BG_PALETTE.index(1).write(magic_max);
        } else if change_pixel == 64 {
            change_pixel = 0;
        }
        change_pixel += 1;

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
        PlayerManager::tick(tick_ctx);
        TopBarManager::tick(tick_ctx);
        ScreenManager::post_tick();
    }
}
