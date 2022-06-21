//! Driver for Good Display GDEY0213B74 (same as GDEQ0213B74)
//!

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

use defmt_rtt as _; // global logger

use crate::buffer_len;
use crate::color::Color;
use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLut, WaveshareDisplay};

pub(crate) mod command;
use self::command::{
    BorderWaveForm, BorderWaveFormFixLevel, BorderWaveFormGs, BorderWaveFormVbd, Command, DataEntryModeDir, DataEntryModeIncr,
    DeepSleepMode, DisplayUpdateControl2, DriverOutput, GateDrivingVoltage, I32Ext, SourceDrivingVoltage, Vcom,
};

pub(crate) mod constants;
use self::constants::{LUT_FULL_UPDATE, LUT_PARTIAL_UPDATE};

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::Display2in13;

/// Width of the display.
pub const WIDTH: u32 = 122;
/// Height of the display
pub const HEIGHT: u32 = 250;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = false;

/// Good Display 'B74' (e.g. GDEY0213B74, GDEQ0213B74) driver
///
pub struct B74Epd<SPI, CS, BUSY, DC, RST, DELAY> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST, DELAY>,

    sleep_mode: DeepSleepMode,

    /// Background Color
    background_color: Color,
    refresh_mode: RefreshLut,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, CS, BUSY, DC, RST, DELAY> for B74Epd<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // HW reset
        self.interface.reset(delay, 50);
        self.wait_until_idle();
        self.command(spi, Command::SwReset);
        self.wait_until_idle();

        // Use init routine from manufacturer's reference implementation
        let use_reference_init = false;
        let invert_display = false; // not available in reference-init

        if use_reference_init {
            self.cmd_with_data(spi, Command::DriverOutputControl, &[0xf9, 0, 0]);

            self.set_data_entry_mode(spi, DataEntryModeIncr::XIncrYIncr, DataEntryModeDir::XDir)?;
            self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
            self.set_ram_address_counters(spi, 0, 0)?;

            self.cmd_with_data(spi, Command::SetRamXAddressStartEndPosition, &[0, 0x0F]);
            self.cmd_with_data(spi, Command::SetRamYAddressStartEndPosition, &[0xf9, 0, 0, 0]);
            self.cmd_with_data(spi, Command::SetRamXAddressCounter, &[0x00]);
            self.cmd_with_data(spi, Command::SetRamYAddressCounter, &[0xf9, 0x00]);

            self.cmd_with_data(spi, Command::BorderWaveformControl, &[0x05]);
            self.cmd_with_data(spi, Command::DisplayUpdateControl1, &[0x0, 0x80]);
            self.cmd_with_data(spi, Command::TemperatureSensorControl, &[0x80]);
        } else if self.refresh_mode == RefreshLut::Quick {
            panic!("Dont support: .refresh_mode == RefreshLut::Quick ");
            self.set_vcom_register(spi, (-9).vcom())?;
            self.wait_until_idle();

            self.set_lut(spi, Some(self.refresh_mode))?;

            // During partial update, clock/analog are not disabled between 2 updates.
            self.set_display_update_control_2(spi, DisplayUpdateControl2::new().enable_analog().enable_clock())?;
            self.command(spi, Command::MasterActivation)?;
            self.wait_until_idle();

            self.set_border_waveform(
                spi,
                BorderWaveForm {
                    vbd: BorderWaveFormVbd::Gs,
                    fix_level: BorderWaveFormFixLevel::Vss,
                    gs_trans: BorderWaveFormGs::Lut1,
                },
            )?;
        } else {
            self.set_driver_output(
                spi,
                DriverOutput {
                    scan_is_linear: true,
                    scan_g0_is_first: true,
                    scan_dir_incr: true,
                    width: (HEIGHT - 1) as u16,
                },
            )?;

            // Use simple X/Y auto increase
            self.set_data_entry_mode(spi, DataEntryModeIncr::XIncrYIncr, DataEntryModeDir::XDir)?;
            self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
            self.set_ram_address_counters(spi, 0, 0)?;

            self.set_lut(spi, Some(self.refresh_mode))?;

            if invert_display {
                self.set_border_waveform(
                    spi,
                    BorderWaveForm {
                        vbd: BorderWaveFormVbd::Gs,
                        fix_level: BorderWaveFormFixLevel::Vss,
                        gs_trans: BorderWaveFormGs::Lut0,
                    },
                )?;

                self.cmd_with_data(spi, Command::DisplayUpdateControl1, &[0x88, 0x80])?;
            } else {
                self.set_border_waveform(
                    spi,
                    BorderWaveForm {
                        vbd: BorderWaveFormVbd::Gs,
                        fix_level: BorderWaveFormFixLevel::Vss,
                        gs_trans: BorderWaveFormGs::Lut3,
                    },
                )?;

                self.cmd_with_data(spi, Command::DisplayUpdateControl1, &[0, 0x80])?;
            }

            self.cmd_with_data(spi, Command::TemperatureSensorControl, &[0x80]);

            // self.set_vcom_register(spi, (-21).vcom())?;
            // self.set_gate_driving_voltage(spi, 190.gate_driving_decivolt())?;
            // self.set_source_driving_voltage(
            //     spi,
            //     150.source_driving_decivolt(),
            //     50.source_driving_decivolt(),
            //     (-150).source_driving_decivolt(),
            // )?;

            // self.set_lut(spi, Some(RefreshLut::Quick))?;
        }

        self.wait_until_idle();
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, CS, BUSY, DC, RST, DELAY> for B74Epd<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    type DisplayColor = Color;
    fn new(spi: &mut SPI, cs: CS, busy: BUSY, dc: DC, rst: RST, delay: &mut DELAY) -> Result<Self, SPI::Error> {
        let mut epd = B74Epd {
            interface: DisplayInterface::new(cs, busy, dc, rst),
            sleep_mode: DeepSleepMode::Mode1,
            background_color: DEFAULT_BACKGROUND_COLOR,
            refresh_mode: RefreshLut::Full,
        };

        epd.init(spi, delay)?;
        Ok(epd)
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn sleep(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle();

        // All sample code enables and disables analog/clocks...
        // self.set_display_update_control_2(
        //     spi,
        //     DisplayUpdateControl2::new()
        //         .enable_analog()
        //         .enable_clock()
        //         .disable_analog()
        //         .disable_clock(),
        // )?;
        // self.command(spi, Command::MasterActivation)?;

        self.go_to_sleep(spi, self.sleep_mode)
    }

    fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8], _delay: &mut DELAY) -> Result<(), SPI::Error> {
        assert!(buffer.len() == buffer_len(WIDTH as usize, HEIGHT as usize));

        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
        self.set_ram_address_counters(spi, 0, 0)?;

        self.cmd_with_data(spi, Command::WriteRam, buffer)?;

        if self.refresh_mode == RefreshLut::Full {
            // Always keep the base buffer equal to current if not doing partial refresh.
            self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
            self.set_ram_address_counters(spi, 0, 0)?;

            self.cmd_with_data(spi, Command::WriteRamRed, buffer)?;
        }
        Ok(())
    }

    /// Updating only a part of the frame is not supported when using the
    /// partial refresh feature. The function will panic if called when set to
    /// use partial refresh.
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        assert!((width * height / 8) as usize == buffer.len());

        // This should not be used when doing partial refresh. The RAM_RED must
        // be updated with the last buffer having been displayed. Doing partial
        // update directly in RAM makes this update impossible (we can't read
        // RAM content). Using this function will most probably make the actual
        // display incorrect as the controler will compare with something
        // incorrect.
        // assert!(self.refresh_mode == RefreshLut::Full);

        self.set_ram_area(spi, x, y, x + width, y + height)?;
        self.set_ram_address_counters(spi, x, y)?;

        self.cmd_with_data(spi, Command::WriteRam, buffer)?;

        // if self.refresh_mode == RefreshLut::Full {
        //     // Always keep the base buffer equals to current if not doing partial refresh.
        //     self.set_ram_area(spi, x, y, x + width, y + height)?;
        //     self.set_ram_address_counters(spi, x, y)?;

        //     self.cmd_with_data(spi, Command::WriteRamRed, buffer)?;
        // }

        Ok(())
    }

    /// Never use directly this function when using partial refresh, or also
    /// keep the base buffer in syncd using `set_partial_base_buffer` function.
    fn display_frame(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        if self.refresh_mode == RefreshLut::Full {
            let du_ctrl_value = DisplayUpdateControl2::new()
                .load_temp()
                .load_lut()
                .enable_clock()
                .enable_analog()
                .display()
                .disable_analog()
                .disable_clock();

            defmt::info!("display_frame()  with RefreshLut::Full {:x}", du_ctrl_value.as_u8());

            // if partial {
            //     self.set_display_update_control_2(spi, du_ctrl_value.display_mode2())?;
            // } else {
            self.set_display_update_control_2(spi, du_ctrl_value)?;
            // }
            // self.cmd_with_data(spi, Command::DisplayUpdateControl2, &[0xf7])?; // display mode 2
        } else {
            self.set_display_update_control_2(spi, DisplayUpdateControl2::new().display())?;
        }

        self.command(spi, Command::MasterActivation)?;
        self.wait_until_idle();

        Ok(())
    }

    fn update_and_display_frame(&mut self, spi: &mut SPI, buffer: &[u8], delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer, delay)?;
        self.display_frame(spi, delay)?;

        if self.refresh_mode == RefreshLut::Quick {
            self.set_partial_base_buffer(spi, buffer)?;
        }
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        let color = self.background_color.get_byte_value();

        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
        self.set_ram_address_counters(spi, 0, 0)?;

        self.command(spi, Command::WriteRam)?;
        self.interface
            .data_x_times(spi, color, buffer_len(WIDTH as usize, HEIGHT as usize) as u32)?;

        // Always keep the base buffer equals to current if not doing partial refresh.
        if self.refresh_mode == RefreshLut::Full {
            self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
            self.set_ram_address_counters(spi, 0, 0)?;

            self.command(spi, Command::WriteRamRed)?;
            self.interface
                .data_x_times(spi, color, buffer_len(WIDTH as usize, HEIGHT as usize) as u32)?;
        }
        Ok(())
    }

    fn set_background_color(&mut self, background_color: Color) {
        self.background_color = background_color;
    }

    fn background_color(&self) -> &Color {
        &self.background_color
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn set_lut(&mut self, spi: &mut SPI, refresh_rate: Option<RefreshLut>) -> Result<(), SPI::Error> {
        defmt::debug!("set_lut() not implemented for SSD1680");
        // let buffer = match refresh_rate {
        //     Some(RefreshLut::Full) | None => &LUT_FULL_UPDATE,
        //     Some(RefreshLut::Quick) => &LUT_PARTIAL_UPDATE,
        //     // Some(RefreshLut::Full) | None => &constants::LUT_PARTIAL_UPDATEZZ,
        //     // Some(RefreshLut::Quick) => &constants::LUT_PARTIAL_UPDATEZZ,
        // };

        // self.cmd_with_data(spi, Command::WriteLutRegister, buffer);

        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> B74Epd<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    /// When using partial refresh, the controller uses the provided buffer for
    /// comparison with new buffer.
    pub fn set_partial_base_buffer(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        assert!(buffer_len(WIDTH as usize, HEIGHT as usize) == buffer.len());
        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
        self.set_ram_address_counters(spi, 0, 0)?;

        self.cmd_with_data(spi, Command::WriteRamRed, buffer)
    }

    /// Selects which sleep mode will be used when triggering the deep sleep.
    pub fn set_sleep_mode(&mut self, mode: DeepSleepMode) {
        self.sleep_mode = mode;
    }

    /// Sets the refresh mode. When changing mode, the screen will be
    /// re-initialized accordingly.
    pub fn set_refresh_mode(&mut self, spi: &mut SPI, delay: &mut DELAY, refresh_mode: RefreshLut) -> Result<(), SPI::Error> {
        if self.refresh_mode != refresh_mode {
            self.refresh_mode = refresh_mode;
            self.init(spi, delay)
        } else {
            Ok(())
        }
    }

    fn set_gate_scan_start_position(&mut self, spi: &mut SPI, start: u16) -> Result<(), SPI::Error> {
        defmt::debug!("scan start position {}", start);
        assert!(start <= 295);
        self.cmd_with_data(
            spi,
            Command::GateScanStartPosition,
            &[(start & 0xFF) as u8, ((start >> 8) & 0x1) as u8],
        )
    }

    fn set_border_waveform(&mut self, spi: &mut SPI, borderwaveform: BorderWaveForm) -> Result<(), SPI::Error> {
        defmt::debug!("borderwaveform 0x{:x}", borderwaveform.to_u8());
        self.cmd_with_data(spi, Command::BorderWaveformControl, &[borderwaveform.to_u8()])
    }

    fn set_vcom_register(&mut self, spi: &mut SPI, vcom: Vcom) -> Result<(), SPI::Error> {
        defmt::debug!("vcom 0x{:x}", vcom.0);
        self.cmd_with_data(spi, Command::WriteVcomRegister, &[vcom.0])
    }

    fn set_gate_driving_voltage(&mut self, spi: &mut SPI, voltage: GateDrivingVoltage) -> Result<(), SPI::Error> {
        defmt::debug!("driving voltage 0x{:x}", voltage.0);
        self.cmd_with_data(spi, Command::GateDrivingVoltageCtrl, &[voltage.0])
    }

    /// Sets the source driving voltage value
    fn set_source_driving_voltage(
        &mut self,
        spi: &mut SPI,
        vsh1: SourceDrivingVoltage,
        vsh2: SourceDrivingVoltage,
        vsl: SourceDrivingVoltage,
    ) -> Result<(), SPI::Error> {
        defmt::debug!("source drive voltages vsh1: {}, vsh2: {}, vsl: {}", vsh1.0, vsh2.0, vsl.0);
        self.cmd_with_data(spi, Command::SourceDrivingVoltageCtrl, &[vsh1.0, vsh2.0, vsl.0])
    }

    /// Prepare the actions that the next master activation command will
    /// trigger.
    fn set_display_update_control_2(&mut self, spi: &mut SPI, value: DisplayUpdateControl2) -> Result<(), SPI::Error> {
        defmt::debug!("DisplayUpdateControl2 0x{:x}", value.0);
        self.cmd_with_data(spi, Command::DisplayUpdateControl2, &[value.0])
    }

    /// Triggers the deep sleep mode
    fn go_to_sleep(&mut self, spi: &mut SPI, mode: DeepSleepMode) -> Result<(), SPI::Error> {
        defmt::debug!("go_to_sleep 0x{:x}", mode as u8);
        self.wait_until_idle();
        self.cmd_with_data(spi, Command::DeepSleepMode, &[mode as u8])
    }

    fn set_driver_output(&mut self, spi: &mut SPI, output: DriverOutput) -> Result<(), SPI::Error> {
        defmt::debug!("DriverOutputControl {:x}", output.to_bytes());
        self.cmd_with_data(spi, Command::DriverOutputControl, &output.to_bytes())
    }

    /// Sets the data entry mode (ie. how X and Y positions changes when writing
    /// data to RAM)
    fn set_data_entry_mode(
        &mut self,
        spi: &mut SPI,
        counter_incr_mode: DataEntryModeIncr,
        counter_direction: DataEntryModeDir,
    ) -> Result<(), SPI::Error> {
        let mode = counter_incr_mode as u8 | counter_direction as u8;
        self.cmd_with_data(spi, Command::DataEntryModeSetting, &[mode])
    }

    /// Sets both X and Y pixels ranges
    fn set_ram_area(&mut self, spi: &mut SPI, start_x: u32, start_y: u32, end_x: u32, end_y: u32) -> Result<(), SPI::Error> {
        self.cmd_with_data(
            spi,
            Command::SetRamXAddressStartEndPosition, //
            &[(start_x >> 3) as u8, (end_x >> 3) as u8],
        )?;

        self.cmd_with_data(
            spi,
            Command::SetRamYAddressStartEndPosition,
            &[start_y as u8, (start_y >> 8) as u8, end_y as u8, (end_y >> 8) as u8],
        )
    }

    /// Sets both X and Y pixels counters when writing data to RAM
    fn set_ram_address_counters(&mut self, spi: &mut SPI, x: u32, y: u32) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.cmd_with_data(spi, Command::SetRamXAddressCounter, &[(x >> 3) as u8])?;
        self.cmd_with_data(spi, Command::SetRamYAddressCounter, &[y as u8, (y >> 8) as u8])
    }

    /// DisplayInterface delegates
    fn command(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, command)
    }

    fn cmd_with_data(&mut self, spi: &mut SPI, command: Command, data: &[u8]) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, command, data)
    }

    fn wait_until_idle(&mut self) {
        let _ = self.interface.wait_until_idle(IS_BUSY_LOW);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 122);
        assert_eq!(HEIGHT, 250);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
