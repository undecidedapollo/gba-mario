#![no_std]
#![no_main]
#![feature(maybe_uninit_array_assume_init)]

use core::{fmt::Write, ptr::copy_nonoverlapping};
use gba::prelude::*;
use mario::{
    assets::{self, BACKGROUND_TILES, zero_screenblock},
    keys::FRAME_KEYS,
    logger,
    score::ScoreManager,
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
            .with_show_obj(false),
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

    zero_screenblock(0);
    zero_screenblock(1);
    zero_screenblock(2);
    zero_screenblock(3);
    zero_screenblock(4);
    zero_screenblock(5);
    zero_screenblock(6);
    zero_screenblock(7);
    zero_screenblock(8);

    unsafe {
        copy_nonoverlapping(
            BACKGROUND_TILES.0.as_ptr(),
            CHARBLOCK0_8BPP.index(1).as_usize() as *mut u8,
            BACKGROUND_TILES.0.len(),
        );
    }

    for i in 0..2 {
        let base = i * 2;
        AFFINE2_SCREENBLOCKS
            .get_frame(16)
            .unwrap()
            .index(0, base)
            .write(u8x2::default().with_high(1).with_low(2));
        AFFINE2_SCREENBLOCKS
            .get_frame(16)
            .unwrap()
            .index(0, base + 1)
            .write(u8x2::default().with_high(17).with_low(18));

        for i in 0..7 {
            AFFINE2_SCREENBLOCKS
                .get_frame(16)
                .unwrap()
                .index(1 + i, base)
                .write(u8x2::default().with_high(3).with_low(4));
            AFFINE2_SCREENBLOCKS
                .get_frame(16)
                .unwrap()
                .index(1 + i, base + 1)
                .write(u8x2::default().with_high(19).with_low(20));
        }
    }

    let mut loop_counter: u16 = 0;

    let min = 1 << 7;
    let max = 1 << 9;

    let mut dir = true;

    let mut val: u16 = min;

    loop {
        VBlankIntrWait();
        loop_counter = loop_counter.wrapping_add(1);
        if dir {
            val += 3;
            if val >= max {
                val = max;
                dir = !dir;
                for _ in 0..60 {
                    VBlankIntrWait();
                }
            }
        } else {
            val -= 3;
            if val <= min {
                val = min;
                dir = !dir;
                for _ in 0..60 {
                    VBlankIntrWait();
                }
            }
        }
        BG2PA.write(i16fx8::from_bits(val as i16));
        BG2PB.write(i16fx8::default());
        BG2PC.write(i16fx8::default());
        BG2PD.write(i16fx8::from_bits(val as i16));
        ScoreManager::tick();
    }
}
