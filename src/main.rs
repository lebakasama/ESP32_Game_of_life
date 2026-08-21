#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

mod game_of_life;

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};

use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    spi::master::{Config as SpiConfig, Spi},
    time::Rate,
    main,
};
use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::ColorOrder,
    Builder
};
//use display_interface_spi::SPIInterface;
use log::{error};

const DISPLAY_WIDTH: u16 = 172;
const DISPLAY_HEIGHT: u16 = 320;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}


// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Setup display GPIOS
    let sclk = peripherals.GPIO1;
    let mosi = peripherals.GPIO2;
    let cs = Output::new(peripherals.GPIO14, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO22, Level::High, OutputConfig::default());
    let _bl = Output::new(peripherals.GPIO23, Level::High, OutputConfig::default());

    // Setup SPI (used to transfer commands and data to the display)
    let spi = Spi::new(peripherals.SPI2,
                       SpiConfig::default().with_frequency(Rate::from_mhz(40u32)))
        .unwrap()
        .with_sck(sclk)
        .with_mosi(mosi);

    let mut buffer = [0_u8; 512];
    let spi_dev = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let di = SpiInterface::new(spi_dev, dc, &mut buffer); // SpiDevice, OutputPin

    let mut delay = Delay::new();

    // Display init
    let mut display = Builder::new(ST7789, di)
        .display_size(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .display_offset(34, 0)
        .color_order(ColorOrder::Bgr)
        .reset_pin(rst)
        .init(&mut delay)
        .unwrap();
    
    let mut gol = game_of_life::GameOfLife::new();
    let (num_rows, num_cols) = gol.dimensions();

    delay = Delay::new();
    let cell_size = (DISPLAY_WIDTH as u32 / num_rows)
                        .min(DISPLAY_HEIGHT as u32 / num_cols);
    let cell_size_i32 = cell_size as i32;

    // Draw something
    display.clear(Rgb565::BLACK).unwrap();
    loop {
        // Get only the cells that have changed so we don't need to clear and redraw the whole
        // screen
        let updated = gol.updated();

        for coords in updated {
            let colour = if gol.alive(coords) {Rgb565::RED} else {Rgb565::BLACK};

            Rectangle::new(Point::new(coords.row * cell_size_i32,
                                      coords.col * cell_size_i32),
                            Size::new(cell_size as u32, cell_size as u32))
                    .into_styled(PrimitiveStyle::with_fill(colour))
                    .draw(&mut display)
                    .unwrap();
        }

        delay.delay_millis(100);
        gol.update();
    }
}
